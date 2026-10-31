//! Versioned, stateless study-entry export for external spaced-repetition trainers.
//! No personal progress or SRS scheduling data is stored by Mexilex.
use crate::{corpus_load, lexical};
use serde_json::{json, Value};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct StudyStats {
    scanned: usize,
    exported: usize,
    excluded_inflections: usize,
    excluded_unlabeled: usize,
    mexico_labeled: usize,
}

pub(crate) fn export_study(
    corpus: &Path,
    destination: &Path,
    mexico_only: bool,
    include_forms: bool,
) -> Result<StudyStats> {
    if destination.exists() {
        return Err(format!("refusing to overwrite {}", destination.display()).into());
    }
    let manifest: Value = serde_json::from_reader(File::open(corpus.join("manifest.json"))?)?;
    if manifest.get("format_version").and_then(Value::as_u64) != Some(1)
        || manifest.get("lexical_language").and_then(Value::as_str) != Some("es-MX")
        || manifest.get("definition_language").and_then(Value::as_str) != Some("en")
    {
        return Err("unsupported input corpus or language pair".into());
    }
    if let Some(parent) = destination.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    // Create-new guarantees this export cannot silently replace an existing deck.
    // On failure, remove the partial export rather than leaving it looking complete.
    let result = (|| -> Result<StudyStats> {
        let mut output = BufWriter::new(
            OpenOptions::new().write(true).create_new(true).open(destination)?,
        );
        let mut stats = StudyStats::default();
        for path in corpus_load::shards(corpus)? {
            for (line_number, line) in BufReader::new(File::open(&path)?).lines().enumerate() {
                let line = line?;
                let record: Value = serde_json::from_str(&line).map_err(|error| {
                    format!("{}:{}: {error}", path.display(), line_number + 1)
                })?;
                stats.scanned += 1;
                let senses = record.get("senses").and_then(Value::as_array)
                    .ok_or("lexical record missing senses")?;
                let only_forms = lexical::is_form(senses);
                if only_forms && !include_forms {
                    stats.excluded_inflections += 1;
                    continue;
                }
                let mex = senses.iter().any(|sense| {
                    sense.get("mexican_usage").and_then(Value::as_bool) == Some(true)
                });
                if mexico_only && !mex {
                    stats.excluded_unlabeled += 1;
                    continue;
                }
                if mex {
                    stats.mexico_labeled += 1;
                }
                let row = json!({
                    "format_version": 1,
                    "source": "mexilex",
                    "source_edition": record["source_edition"],
                    "lexical_language": "es-MX",
                    "definition_language": "en",
                    "entry_id": record["id"],
                    "word": record["word"],
                    "part_of_speech": record["part_of_speech"],
                    "etymology_number": record["etymology_number"],
                    "forms": record["forms"],
                    "senses": senses,
                    "mexico_labeled": mex,
                    "inflection_only": only_forms,
                });
                serde_json::to_writer(&mut output, &row)?;
                output.write_all(b"\n")?;
                stats.exported += 1;
            }
        }
        let expected = manifest.get("entry_count").and_then(Value::as_u64)
            .ok_or("manifest lacks entry count")?;
        if stats.scanned as u64 != expected {
            return Err(format!("incomplete corpus: expected {expected}, read {}", stats.scanned).into());
        }
        output.flush()?;
        Ok(stats)
    })();
    if result.is_err() {
        let _ = fs::remove_file(destination);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn export_preserves_meanings_and_requires_no_user_history() {
        let mut db = Connection::open_in_memory().unwrap();
        crate::import_reader(
            &mut db,
            Cursor::new(include_str!("../tests/fixtures/sample.jsonl")),
        ).unwrap();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("mexilex-study-{}-{unique}",std::process::id()));
        let input = dir.join("corpus");
        crate::corpus::export_corpus(&db, &input).unwrap();
        let output = dir.join("study.jsonl");
        let stats = export_study(&input, &output, true, false).unwrap();
        assert_eq!(stats.scanned, 2);
        assert_eq!(stats.exported, 1);
        assert_eq!(stats.mexico_labeled, 1);
        let line = fs::read_to_string(&output).unwrap();
        let entry: Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(entry["word"], "banqueta");
        assert_eq!(entry["senses"].as_array().unwrap().len(), 2);
        assert_eq!(entry["senses"][0]["definition_en"], "sidewalk");
        assert_eq!(entry["senses"][1]["definition_en"], "footstool");
        assert_eq!(entry["lexical_language"], "es-MX");
        assert_eq!(entry["definition_language"], "en");
        assert!(entry.get("review_history").is_none());
        assert!(export_study(&input, &output, true, false).is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn form_only_entries_are_suppressed_by_default() {
        let mut db = Connection::open_in_memory().unwrap();
        let data = [
            r#"{"lang_code":"es","word":"caminar","pos":"verb","senses":[{"glosses":["to walk"],"tags":[]}]}"#,
            r#"{"lang_code":"es","word":"caminé","pos":"verb","senses":[{"glosses":["first-person singular preterite of caminar"],"tags":["form-of"]}]}"#,
        ].join("\n");
        crate::import_reader(&mut db, Cursor::new(data)).unwrap();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("mexilex-forms-{}-{unique}",std::process::id()));
        let input = dir.join("corpus");
        crate::corpus::export_corpus(&db, &input).unwrap();
        let a = export_study(&input, &dir.join("lemmas.jsonl"), false, false).unwrap();
        assert_eq!(a.exported, 1);
        assert_eq!(a.excluded_inflections, 1);
        let b = export_study(&input, &dir.join("all.jsonl"), false, true).unwrap();
        assert_eq!(b.exported, 2);
        fs::remove_dir_all(dir).unwrap();
    }
}
