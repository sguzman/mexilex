//! Mexican film-dialogue word-frequency *evidence*, not a dialect-label dictionary.
//!
//! ACTIV-ES v0.2 uses subtitle-derived counts; source data is separately
//! GPL-2.0 licensed. Never infer a "Mexico-only" sense from a frequency.
use crate::{corpus_load, lexical};
use csv::StringRecord;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub(crate) const SOURCE_COMMIT: &str = "e42124efbd0c6c30fc8b6a93e5d3a1e9bc4c7e08";
pub(crate) const SOURCE_PATH: &str = "activ-es-v.02/wordlists/plain/aes1grams.csv";

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct EvidenceStats {
    pub(crate) source_rows: usize,
    pub(crate) matched_words: usize,
    pub(crate) attested_mexico: usize,
    pub(crate) discarded_non_headwords: usize,
}

fn exact_lower(word: &str) -> String {
    word.trim().to_lowercase()
}

fn field<'a>(record: &'a StringRecord, index: usize, label: &str) -> Result<&'a str> {
    record
        .get(index)
        .ok_or_else(|| format!("missing {label}").into())
}
fn rate(record: &StringRecord, index: usize, label: &str) -> Result<f64> {
    let n: f64 = field(record, index, label)?.parse()?;
    if !n.is_finite() || n < 0.0 {
        return Err(format!("{label} must be a finite nonnegative number").into());
    }
    Ok(n)
}

fn known_headwords(corpus: &Path) -> Result<HashSet<String>> {
    let mut words = HashSet::new();
    for path in corpus_load::shards(corpus)? {
        for line in BufReader::new(File::open(path)?).lines() {
            let record: Value = serde_json::from_str(&line?)?;
            let senses = record
                .get("senses")
                .and_then(Value::as_array)
                .ok_or("corpus record missing senses")?;
            if !lexical::is_form(senses) {
                let word = record
                    .get("word")
                    .and_then(Value::as_str)
                    .ok_or("corpus record missing word")?;
                words.insert(exact_lower(word));
            }
        }
    }
    Ok(words)
}

pub(crate) fn parse_evidence<R: Read, W: Write>(
    source: R,
    mut output: W,
    headwords: &HashSet<String>,
) -> Result<EvidenceStats> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(false)
        .from_reader(source);
    let headers = reader.headers()?.clone();
    let index = |label: &str| -> Result<usize> {
        headers
            .iter()
            .position(|header| header == label)
            .ok_or_else(|| format!("missing ACTIV-ES column {label}").into())
    };
    let word_col = index("word")?;
    let mx_col = index("mx_orf")?;
    let es_col = index("es_orf")?;
    let ar_col = index("ar_orf")?;
    let dispersion_col = index("mx_ord")?;
    let mut rows: BTreeMap<String, [f64; 4]> = BTreeMap::new();
    let mut stats = EvidenceStats::default();
    for result in reader.records() {
        let row = result?;
        stats.source_rows += 1;
        let word = exact_lower(field(&row, word_col, "word")?);
        if word.is_empty() || !headwords.contains(&word) {
            stats.discarded_non_headwords += 1;
            continue;
        }
        let values = [
            rate(&row, mx_col, "mx_orf")?,
            rate(&row, es_col, "es_orf")?,
            rate(&row, ar_col, "ar_orf")?,
            rate(&row, dispersion_col, "mx_ord")?,
        ];
        if values[3] > 10.0 {
            return Err("Mexico dispersion cannot exceed 10 documents".into());
        }
        if rows.insert(word.clone(), values).is_some() {
            return Err(format!("duplicate ACTIV-ES word: {word}").into());
        }
    }
    for (word, [mx, es, ar, dispersion]) in rows {
        if mx == 0.0 {
            continue;
        }
        serde_json::to_writer(&mut output, &json!([word, mx, es, ar, dispersion]))?;
        output.write_all(b"\n")?;
        stats.attested_mexico += 1;
    }
    stats.matched_words = stats.source_rows - stats.discarded_non_headwords;
    Ok(stats)
}

pub(crate) fn import_evidence(
    source: &Path,
    corpus: &Path,
    destination: &Path,
) -> Result<EvidenceStats> {
    if destination.exists() {
        return Err("refusing to overwrite frequency evidence".into());
    }
    let words = known_headwords(corpus)?;
    if words.is_empty() {
        return Err("empty tracked lexical corpus".into());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let result = (|| -> Result<EvidenceStats> {
        let mut checksum = Sha256::new();
        let mut input = File::open(source)?;
        let mut block = [0u8; 65536];
        loop {
            let n = input.read(&mut block)?;
            if n == 0 {
                break;
            }
            checksum.update(&block[..n]);
        }
        let writer = BufWriter::new(
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(destination)?,
        );
        let stats = parse_evidence(BufReader::new(File::open(source)?), writer, &words)?;
        let manifest = json!({
            "format_version": 1,
            "source": "ACTIV-ES v0.2, one-gram plain surface forms",
            "source_repo": "francojc/activ-es",
            "source_commit": SOURCE_COMMIT,
            "source_path": SOURCE_PATH,
            "source_sha256": format!("{:x}", checksum.finalize()),
            "license": "GPL-2.0-only; see ACTIV-ES-LICENSE",
            "measure": "observed relative frequency per 100,000 words of film dialogue",
            "dispersion": "Mexico document dispersion per 10 films",
            "comparison": ["Mexico", "Spain", "Argentina"],
            "matching": "exact case-insensitive surface spelling of Git-tracked non-form-only headwords; no POS or sense inference",
            "source_rows": stats.source_rows,
            "matched_words": stats.matched_words,
            "attested_mexico": stats.attested_mexico
        });
        fs::write(
            destination.with_extension("manifest.json"),
            format!("{}\n", serde_json::to_string_pretty(&manifest)?),
        )?;
        Ok(stats)
    })();
    if result.is_err() {
        let _ = fs::remove_file(destination);
        let _ = fs::remove_file(destination.with_extension("manifest.json"));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn preserves_mexican_attestation_without_claiming_exclusivity() {
        let csv = "\"\",\"word\",\"ar_orf\",\"es_orf\",\"mx_orf\",\"aes_orf\",\"ar_ord\",\"es_ord\",\"mx_ord\",\"aes_ord\"\n\
            \"1\",\"banqueta\",2,1,7,3,1,1,6,2\n\
            \"2\",\"casa\",40,30,70,46,9,9,10,9\n\
            \"3\",\"caminé\",5,5,5,5,2,2,2,2\n\
            \"4\",\"chamba\",0.2,0,2,1,1,0,3,1\n\
            \"5\",\"raro\",0,0,0,0,0,0,0,0\n";
        let words = ["banqueta", "casa", "chamba", "raro"]
            .iter()
            .map(|w| w.to_string())
            .collect();
        let mut buffer = Vec::new();
        let counts = parse_evidence(Cursor::new(csv), &mut buffer, &words).unwrap();
        assert_eq!(counts.source_rows, 5);
        assert_eq!(counts.matched_words, 4);
        assert_eq!(counts.attested_mexico, 3);
        let rows: Vec<Value> = String::from_utf8(buffer)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        let sidewalk = rows.iter().find(|r| r[0] == "banqueta").unwrap();
        assert_eq!(sidewalk[1], 7.0);
        assert_eq!(sidewalk[2], 1.0);
        assert_eq!(sidewalk[3], 2.0);
        assert_eq!(sidewalk[4], 6.0);
        assert!(!rows.iter().any(|r| r[0] == "caminé" || r[0] == "raro"));
    }

    #[test]
    fn rejects_bad_columns_and_non_finite_rates() {
        let words = ["casa".to_string()].into_iter().collect();
        let mut output = Vec::new();
        assert!(parse_evidence(Cursor::new("word,mx_orf\ncasa,2\n"), &mut output, &words).is_err());
        let header = "word,ar_orf,es_orf,mx_orf,mx_ord\n";
        assert!(parse_evidence(
            Cursor::new(format!("{header}casa,1,2,NaN,3\n")),
            &mut output,
            &words
        )
        .is_err());
        assert!(parse_evidence(
            Cursor::new(format!("{header}casa,1,2,3,11\n")),
            &mut output,
            &words
        )
        .is_err());
    }
}
