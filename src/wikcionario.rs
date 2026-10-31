//! Independent word-level Mexican regional evidence from Spanish Wiktionary.
//! This cannot be joined to an English-language *sense*: only spelling matches
//! are computed. Do not infer a specific English gloss is Mexico-only.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub(crate) const SOURCE_URL: &str = "https://kaikki.org/eswiktionary/Espa%C3%B1ol/tags/p9/Mexico/kaikki.org-dictionary-Espa%C3%B1ol-tag-Mexico.jsonl";

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct SourceStats {
    pub(crate) source_records: usize,
    pub(crate) source_senses: usize,
    pub(crate) distinct_spellings: usize,
    pub(crate) distinct_word_pos: usize,
    pub(crate) matched_word_pos: usize,
    pub(crate) matched_spellings: usize,
}

fn canonical(word: &str) -> String {
    word.trim().to_lowercase()
}

pub(crate) fn parse_records<R: BufRead, W: Write>(
    source: R,
    mut output: W,
    headwords: &HashSet<String>,
) -> Result<SourceStats> {
    let mut rows: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut source_records = 0;
    let mut source_senses = 0;
    for (offset, line) in source.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: Value = serde_json::from_str(&line)
            .map_err(|error| format!("invalid Spanish Wiktionary line {}: {error}", offset + 1))?;
        if record.get("lang_code").and_then(Value::as_str) != Some("es") {
            return Err(format!("non-Spanish source record at line {}", offset + 1).into());
        }
        let word = record
            .get("word")
            .and_then(Value::as_str)
            .ok_or("missing spelling")?;
        let pos = record
            .get("pos")
            .and_then(Value::as_str)
            .ok_or("missing POS")?;
        if word.trim().is_empty() || pos.trim().is_empty() {
            return Err("empty spelling or POS in Spanish Wiktionary feed".into());
        }
        let senses = record
            .get("senses")
            .and_then(Value::as_array)
            .ok_or("missing senses")?;
        let mexican = senses
            .iter()
            .filter(|sense| {
                sense
                    .get("tags")
                    .and_then(Value::as_array)
                    .is_some_and(|tags| tags.iter().any(|tag| tag.as_str() == Some("Mexico")))
            })
            .count();
        if mexican == 0 {
            return Err(format!(
                "record without explicit Mexico tag at source line {}",
                offset + 1
            )
            .into());
        }
        source_records += 1;
        source_senses += mexican;
        *rows.entry((word.to_owned(), pos.to_owned())).or_default() += mexican;
    }
    if rows.is_empty() {
        return Err("empty Mexico-tagged Spanish Wiktionary feed".into());
    }
    let mut spellings = HashSet::new();
    let mut matched_spellings = HashSet::new();
    let mut matched_word_pos = 0;
    for ((word, pos), senses) in &rows {
        let lower = canonical(word);
        spellings.insert(lower.clone());
        let matched = headwords.contains(&lower);
        if matched {
            matched_word_pos += 1;
            matched_spellings.insert(lower);
        }
        serde_json::to_writer(&mut output, &json!([word, pos, senses, matched]))?;
        output.write_all(b"\n")?;
    }
    Ok(SourceStats {
        source_records,
        source_senses,
        distinct_spellings: spellings.len(),
        distinct_word_pos: rows.len(),
        matched_word_pos,
        matched_spellings: matched_spellings.len(),
    })
}

pub(crate) fn import_file(source: &Path, corpus: &Path, destination: &Path) -> Result<SourceStats> {
    if destination.exists() || destination.with_extension("manifest.json").exists() {
        return Err("refusing to overwrite existing Wikcionario evidence".into());
    }
    let headwords = crate::evidence::known_headwords(corpus)?;
    if headwords.is_empty() {
        return Err("empty normalized English-Wiktionary headword corpus".into());
    }
    if let Some(parent) = destination.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let result = (|| -> Result<SourceStats> {
        let bytes = fs::read(source)?;
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        let mut output = BufWriter::new(
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(destination)?,
        );
        let stats = parse_records(BufReader::new(bytes.as_slice()), &mut output, &headwords)?;
        output.flush()?;
        let manifest = json!({
            "format_version": 1,
            "source": "Spanish Wiktionary via Kaikki; explicitly Mexico-tagged Español senses",
            "source_edition": "eswiktionary",
            "source_language": "es",
            "lexical_language": "es-MX",
            "source_url": SOURCE_URL,
            "source_sha256": sha256,
            "license": "Wiktionary CC BY-SA/GFDL; see WIKCIONARIO-SOURCE.md",
            "source_records": stats.source_records,
            "source_senses": stats.source_senses,
            "distinct_spellings": stats.distinct_spellings,
            "distinct_word_pos": stats.distinct_word_pos,
            "matched_word_pos": stats.matched_word_pos,
            "matched_spellings": stats.matched_spellings,
            "matching": "case-insensitive exact spelling of a non-inflection-only headword in English Wiktionary; never a sense-level match"
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
    fn independent_mx_tagging_never_attaches_to_an_english_sense() {
        let data = [
            r#"{"lang_code":"es","word":"banqueta","pos":"noun","senses":[{"tags":["Mexico"],"glosses":["acera"]},{"tags":["Spain"],"glosses":["taburete"]}]}"#,
            r#"{"lang_code":"es","word":"chamba","pos":"noun","senses":[{"tags":["Mexico"],"glosses":["trabajo"]}]}"#,
            r#"{"lang_code":"es","word":"chamba","pos":"verb","senses":[{"tags":["Mexico"],"glosses":["otro uso"]}]}"#
        ].join("\n");
        let headwords = ["banqueta".to_owned()].into_iter().collect();
        let mut out = Vec::new();
        let stats = parse_records(Cursor::new(data), &mut out, &headwords).unwrap();
        assert_eq!(stats.source_records, 3);
        assert_eq!(stats.source_senses, 3);
        assert_eq!(stats.distinct_spellings, 2);
        assert_eq!(stats.distinct_word_pos, 3);
        assert_eq!(stats.matched_word_pos, 1);
        assert_eq!(stats.matched_spellings, 1);
        let rows: Vec<Value> = String::from_utf8(out)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(rows[0], json!(["banqueta", "noun", 1, true]));
        assert_eq!(rows[1], json!(["chamba", "noun", 1, false]));
        assert_eq!(rows[2], json!(["chamba", "verb", 1, false]));
    }

    #[test]
    fn ignores_neither_unknown_regions_nor_malformed_feeds() {
        let headwords = HashSet::new();
        let bad = r#"{"lang_code":"es","word":"banqueta","pos":"noun","senses":[{"tags":["New-Mexico"]}]}"#;
        assert!(parse_records(Cursor::new(bad), Vec::new(), &headwords).is_err());
        let other_language =
            r#"{"lang_code":"en","word":"banqueta","pos":"noun","senses":[{"tags":["Mexico"]}]}"#;
        assert!(parse_records(Cursor::new(other_language), Vec::new(), &headwords).is_err());
    }
}
