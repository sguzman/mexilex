//! Deterministic, Git-friendly corpus publication.
//!
//! The upstream dump and derived SQLite database are *build inputs*;
//! the normalized JSONL shards are the versioned lexical source in Git.
//! Shard paths derive from stable entry-key prefixes, rather than source-line
//! positions, so reordering the dump does not rewrite unrelated shards.
use rusqlite::Connection;
use serde_json::{json, Value};
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct CorpusStats {
    entries: usize,
    senses: usize,
    mexican_senses: usize,
    shards: usize,
}

fn parse_json(s: String) -> Result<Value> {
    Ok(serde_json::from_str(&s)?)
}

pub(crate) fn export_corpus(connection: &Connection, destination: &Path) -> Result<CorpusStats> {
    if destination.exists() && fs::read_dir(destination)?.next().is_some() {
        return Err(format!(
            "corpus output directory {} must be empty; export into a fresh directory",
            destination.display()
        )
        .into());
    }
    fs::create_dir_all(destination)?;

    let mut entries = connection.prepare(
        "SELECT key, word, pos, etymology_number, forms_json, tags_json, source_edition
         FROM entries ORDER BY key",
    )?;
    let mut senses = connection.prepare(
        "SELECT key, sense_index, definition_en, glosses_json, tags_json,
                raw_tags_json, examples_json, mexican_usage
         FROM senses WHERE entry_key = ?1 ORDER BY sense_index, key",
    )?;
    let mut rows = entries.query([])?;
    let mut current_shard = String::new();
    let mut output: Option<BufWriter<File>> = None;
    let mut counts = CorpusStats::default();

    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        if key.len() < 3 || !key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!("invalid hashed entry key: {key}").into());
        }
        // 4096 possible shards, each in a 16-file directory. Ordering by hash
        // makes output deterministic and avoids a large number of open files.
        let shard = &key[..3];
        if current_shard != shard {
            if let Some(ref mut writer) = output {
                writer.flush()?;
            }
            let directory = destination.join(&shard[..2]);
            fs::create_dir_all(&directory)?;
            output = Some(BufWriter::new(File::create(
                directory.join(format!("{}.jsonl", &shard[2..])),
            )?));
            current_shard = shard.to_owned();
            counts.shards += 1;
        }
        let mut record_senses = Vec::new();
        let mut sense_rows = senses.query([&key])?;
        while let Some(sense) = sense_rows.next()? {
            let mexican: i64 = sense.get(7)?;
            record_senses.push(json!({
                "id": sense.get::<_, String>(0)?,
                "sense_index": sense.get::<_, i64>(1)?,
                "definition_en": sense.get::<_, String>(2)?,
                "glosses": parse_json(sense.get(3)?)?,
                "tags": parse_json(sense.get(4)?)?,
                "raw_tags": parse_json(sense.get(5)?)?,
                "examples": parse_json(sense.get(6)?)?,
                "mexican_usage": mexican != 0,
            }));
            counts.senses += 1;
            if mexican != 0 {
                counts.mexican_senses += 1;
            }
        }
        let record = json!({
            "id": key,
            "language": "es-MX",
            "definition_language": "en",
            "word": row.get::<_, String>(1)?,
            "part_of_speech": row.get::<_, String>(2)?,
            "etymology_number": row.get::<_, Option<i64>>(3)?,
            "forms": parse_json(row.get(4)?)?,
            "tags": parse_json(row.get(5)?)?,
            "source_edition": row.get::<_, String>(6)?,
            "senses": record_senses
        });
        writeln!(output.as_mut().expect("output initialized"), "{record}")?;
        counts.entries += 1;
    }
    if let Some(ref mut writer) = output {
        writer.flush()?;
    }
    let manifest = json!({
        "format_version": 1,
        "lexical_language": "es-MX",
        "definition_language": "en",
        "source_edition": "enwiktionary",
        "source_url": "https://kaikki.org/dictionary/Spanish/",
        "attribution": "English Wiktionary contributors via Kaikki/Wiktextract",
        "license_note": "Wiktionary-derived data retains upstream CC BY-SA and GFDL obligations; the MIT code license does not apply to the corpus.",
        "license_urls": [
            "https://en.wiktionary.org/wiki/Wiktionary:Copyrights",
            "https://creativecommons.org/licenses/by-sa/4.0/"
        ],
        "entry_count": counts.entries,
        "sense_count": counts.senses,
        "mexico_labeled_sense_count": counts.mexican_senses,
        "shard_count": counts.shards
    });
    fs::write(
        destination.join("manifest.json"),
        format!("{}\n", serde_json::to_string_pretty(&manifest)?),
    )?;
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn export_is_repeatable_and_region_metadata_is_per_sense() {
        let mut db = Connection::open_in_memory().unwrap();
        crate::import_reader(
            &mut db,
            Cursor::new(include_str!("../tests/fixtures/sample.jsonl")),
        )
        .unwrap();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let first = std::env::temp_dir().join(format!("mexilex-{}-{unique}-1", std::process::id()));
        let second = std::env::temp_dir().join(format!("mexilex-{}-{unique}-2", std::process::id()));
        let stats = export_corpus(&db, &first).unwrap();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.senses, 3);
        assert_eq!(stats.mexican_senses, 1);
        assert!(export_corpus(&db, &first).is_err());
        assert_eq!(stats, export_corpus(&db, &second).unwrap());

        let first_manifest = fs::read_to_string(first.join("manifest.json")).unwrap();
        let second_manifest = fs::read_to_string(second.join("manifest.json")).unwrap();
        assert_eq!(first_manifest, second_manifest);
        let manifest: Value = serde_json::from_str(&first_manifest).unwrap();
        assert_eq!(manifest["lexical_language"], "es-MX");
        assert_eq!(manifest["definition_language"], "en");
        assert_eq!(manifest["mexico_labeled_sense_count"], 1);
        let mut parsed = Vec::new();
        for entry in fs::read_dir(&first).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                for shard in fs::read_dir(entry.path()).unwrap() {
                    let shard = shard.unwrap();
                    let data = fs::read_to_string(shard.path()).unwrap();
                    let other = second.join(entry.file_name()).join(shard.file_name());
                    assert_eq!(data, fs::read_to_string(other).unwrap());
                    for line in data.lines() {
                        parsed.push(serde_json::from_str::<Value>(line).unwrap());
                    }
                }
            }
        }
        assert_eq!(parsed.len(), 2);
        let banqueta = parsed.iter().find(|value| value["word"] == "banqueta").unwrap();
        assert_eq!(banqueta["senses"][0]["mexican_usage"], true);
        assert_eq!(banqueta["senses"][1]["mexican_usage"], false);
        fs::remove_dir_all(first).unwrap();
        fs::remove_dir_all(second).unwrap();
    }
}
