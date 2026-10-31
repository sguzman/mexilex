//! Reconstitute a searchable database from the normalized corpus checked into Git.
use rusqlite::{params, Connection};
use serde_json::Value;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Loaded {
    entries: usize,
    senses: usize,
    mexico_labeled_senses: usize,
    shards: usize,
}

fn required<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {key}").into())
}

fn json_field(value: &Value, field: &str) -> Result<String> {
    let data = value
        .get(field)
        .ok_or_else(|| format!("missing field {field}"))?;
    if !data.is_array() {
        return Err(format!("{field} must be an array").into());
    }
    Ok(data.to_string())
}

fn exact_count(manifest: &Value, key: &str, count: usize) -> Result<()> {
    let expected = manifest
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("manifest missing numeric {key}"))?;
    if expected != count as u64 {
        return Err(format!("{key} mismatch: manifest={expected}, actual={count}").into());
    }
    Ok(())
}

fn shards(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for item in fs::read_dir(directory)? {
        let item = item?;
        if !item.file_type()?.is_dir() {
            continue;
        }
        let prefix = item.file_name().to_string_lossy().into_owned();
        if prefix.len() != 2 || !prefix.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("unexpected directory in corpus: {prefix}").into());
        }
        for candidate in fs::read_dir(item.path())? {
            let candidate = candidate?;
            if !candidate.file_type()?.is_file() {
                return Err(format!("unexpected non-file: {}", candidate.path().display()).into());
            }
            let name = candidate.file_name().to_string_lossy().into_owned();
            if name.len() != 7
                || !name.ends_with(".jsonl")
                || !name.as_bytes()[0].is_ascii_hexdigit()
            {
                return Err(format!("unexpected shard name: {name}").into());
            }
            paths.push(candidate.path());
        }
    }
    paths.sort();
    Ok(paths)
}

pub(crate) fn load_corpus(db: &mut Connection, directory: &Path) -> Result<Loaded> {
    let manifest: Value = serde_json::from_reader(File::open(directory.join("manifest.json"))?)?;
    if manifest.get("format_version").and_then(Value::as_u64) != Some(1) {
        return Err("unsupported corpus format version".into());
    }
    if required(&manifest, "lexical_language")? != "es-MX"
        || required(&manifest, "definition_language")? != "en"
    {
        return Err("unexpected language metadata in corpus".into());
    }
    let paths = shards(directory)?;
    db.execute_batch(include_str!("schema.sql"))?;
    let transaction = db.transaction()?;
    transaction.execute("DELETE FROM entries", [])?;
    transaction.execute("DELETE FROM metadata", [])?;
    for (key, value) in [
        ("source_edition", required(&manifest, "source_edition")?),
        ("source_url", required(&manifest, "source_url")?),
        ("definition_language", "en"),
        ("lexical_language", "es-MX"),
    ] {
        transaction.execute(
            "INSERT INTO metadata (key,value) VALUES (?1,?2)",
            params![key, value],
        )?;
    }
    let mut stats = Loaded::default();
    {
        let mut write_entry = transaction.prepare_cached(
            "INSERT INTO entries
             (key, word, pos, etymology_number, forms_json, tags_json, source_edition, source_line)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        let mut write_sense = transaction.prepare_cached(
            "INSERT INTO senses (key, entry_key, sense_index, definition_en,
             glosses_json, tags_json, raw_tags_json, examples_json, mexican_usage)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;
        for path in paths {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            for (line_index, line) in reader.lines().enumerate() {
                let line = line?;
                let record: Value = serde_json::from_str(&line)
                    .map_err(|err| format!("{}:{}: {err}", path.display(), line_index + 1))?;
                let key = required(&record, "id")?;
                let word = required(&record, "word")?;
                let pos = required(&record, "part_of_speech")?;
                if required(&record, "language")? != "es-MX"
                    || required(&record, "definition_language")? != "en"
                {
                    return Err(format!("{}: language mismatch", path.display()).into());
                }
                let key_prefix = &key.get(..3).ok_or("invalid entry key")?;
                let stem = path
                    .file_stem()
                    .ok_or("invalid shard filename")?
                    .to_string_lossy();
                let parent = path
                    .parent()
                    .ok_or("invalid shard path")?
                    .file_name()
                    .ok_or("invalid shard directory")?
                    .to_string_lossy();
                if key_prefix != format!("{parent}{stem}") {
                    return Err(format!("{}: misplaced entry {key}", path.display()).into());
                }
                let etym = record.get("etymology_number").and_then(Value::as_i64);
                write_entry.execute(params![
                    key,
                    word,
                    pos,
                    etym,
                    json_field(&record, "forms")?,
                    json_field(&record, "tags")?,
                    required(&record, "source_edition")?,
                    0i64,
                ])?;
                stats.entries += 1;
                let senses = record
                    .get("senses")
                    .and_then(Value::as_array)
                    .ok_or("missing senses array")?;
                for sense in senses {
                    let mexico = sense
                        .get("mexican_usage")
                        .and_then(Value::as_bool)
                        .ok_or("invalid Mexican usage flag")?;
                    let index = sense
                        .get("sense_index")
                        .and_then(Value::as_i64)
                        .ok_or("missing sense index")?;
                    write_sense.execute(params![
                        required(sense, "id")?,
                        key,
                        index,
                        required(sense, "definition_en")?,
                        json_field(sense, "glosses")?,
                        json_field(sense, "tags")?,
                        json_field(sense, "raw_tags")?,
                        json_field(sense, "examples")?,
                        i64::from(mexico),
                    ])?;
                    stats.senses += 1;
                    if mexico {
                        stats.mexico_labeled_senses += 1;
                    }
                }
            }
            stats.shards += 1;
        }
    }
    exact_count(&manifest, "entry_count", stats.entries)?;
    exact_count(&manifest, "sense_count", stats.senses)?;
    exact_count(
        &manifest,
        "mexico_labeled_sense_count",
        stats.mexico_labeled_senses,
    )?;
    exact_count(&manifest, "shard_count", stats.shards)?;
    transaction.commit()?;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn round_trip_from_git_corpus() {
        let mut source = Connection::open_in_memory().unwrap();
        crate::import_reader(
            &mut source,
            Cursor::new(include_str!("../tests/fixtures/sample.jsonl")),
        )
        .unwrap();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("mexilex-roundtrip-{}-{unique}", std::process::id()));
        crate::corpus::export_corpus(&source, &directory).unwrap();
        let mut target = Connection::open_in_memory().unwrap();
        let counts = load_corpus(&mut target, &directory).unwrap();
        assert_eq!(counts.entries, 2);
        assert_eq!(counts.senses, 3);
        assert_eq!(counts.mexico_labeled_senses, 1);
        let source_records: i64 = source
            .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))
            .unwrap();
        let target_records: i64 = target
            .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))
            .unwrap();
        assert_eq!(source_records, target_records);
        let mut cards = Vec::new();
        crate::export_tsv(&target, &mut cards, false).unwrap();
        let cards = String::from_utf8(cards).unwrap();
        assert!(cards.contains("banqueta"));
        assert!(cards.contains("sidewalk"));
        assert!(cards.contains("footstool"));
        fs::remove_dir_all(directory).unwrap();
    }
}
