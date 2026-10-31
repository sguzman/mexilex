use flate2::read::MultiGzDecoder;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const EDITION: &str = "enwiktionary";
const SOURCE_URL: &str = "https://kaikki.org/dictionary/Spanish/";

#[derive(Default, Debug, PartialEq, Eq)]
struct Counts {
    lines: usize,
    spanish_records: usize,
    entries: usize,
    senses: usize,
    mexican_senses: usize,
    skipped_without_definition: usize,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("mexilex: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or_else(usage)?;
    match command.as_str() {
        "import" => {
            let source = args.next().ok_or_else(usage)?;
            let target = args.next().ok_or_else(usage)?;
            if args.next().is_some() {
                return Err(usage().into());
            }
            if let Some(parent) = Path::new(&target).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let reader = open_source(&source)?;
            let mut connection = Connection::open(target)?;
            let counts = import_reader(&mut connection, reader)?;
            println!("{counts:#?}");
        }
        "stats" => {
            let db = args.next().ok_or_else(usage)?;
            if args.next().is_some() {
                return Err(usage().into());
            }
            let connection =
                Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            let entries: i64 =
                connection.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))?;
            let senses: i64 =
                connection.query_row("SELECT COUNT(*) FROM senses", [], |row| row.get(0))?;
            let mexican: i64 = connection.query_row(
                "SELECT COUNT(*) FROM senses WHERE mexican_usage = 1",
                [],
                |row| row.get(0),
            )?;
            println!("entries: {entries}\nsenses: {senses}\nMexico-labeled senses: {mexican}");
        }
        "lookup" => {
            let db = args.next().ok_or_else(usage)?;
            let word = args.next().ok_or_else(usage)?;
            if args.next().is_some() {
                return Err(usage().into());
            }
            let connection =
                Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            lookup(&connection, &word)?;
        }
        "export-tsv" => {
            let db = args.next().ok_or_else(usage)?;
            let output = args.next().ok_or_else(usage)?;
            let filter = args.next();
            if filter
                .as_deref()
                .is_some_and(|flag| flag != "--mexican-only")
                || args.next().is_some()
            {
                return Err(usage().into());
            }
            let connection =
                Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            let out = BufWriter::new(File::create(output)?);
            export_tsv(&connection, out, filter.is_some())?;
        }
        _ => return Err(usage().into()),
    }
    Ok(())
}

fn usage() -> String {
    "usage:\n  mexilex import <enwiktionary.jsonl[.gz]> <catalog.sqlite>\n  mexilex stats <catalog.sqlite>\n  mexilex lookup <catalog.sqlite> <spanish-word>\n  mexilex export-tsv <catalog.sqlite> <cards.tsv> [--mexican-only]".to_owned()
}

fn open_source(path: &str) -> Result<Box<dyn BufRead>> {
    let file = File::open(path)?;
    if path.ends_with(".gz") {
        Ok(Box::new(BufReader::new(MultiGzDecoder::new(file))))
    } else {
        Ok(Box::new(BufReader::new(file)))
    }
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|item| !item.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

fn encoded(value: Option<&Value>) -> String {
    value.cloned().unwrap_or_else(|| json!([])).to_string()
}

fn tagged_mexican(sense: &Value) -> bool {
    ["tags", "raw_tags", "categories"].iter().any(|name| {
        strings(sense.get(*name)).iter().any(|tag| {
            matches!(
                tag.trim().to_ascii_lowercase().as_str(),
                "mexico"
                    | "mexican"
                    | "mexican spanish"
                    | "mexico city"
                    | "mexican-spanish"
                    | "mexico city spanish"
            )
        })
    })
}

fn fingerprint(fields: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

// Import is atomic: an invalid or interrupted dataset does not leave a half-written catalogue.
// The input must be an English-Wiktionary Wiktextract/Kaikki JSONL feed.
// We intentionally do not translate entries from the Spanish Wiktionary edition.
fn import_reader<R: BufRead>(connection: &mut Connection, reader: R) -> Result<Counts> {
    connection.execute_batch(include_str!("schema.sql"))?;
    let transaction = connection.transaction()?;
    transaction.execute("DELETE FROM entries", [])?;
    transaction.execute("DELETE FROM metadata", [])?;
    transaction.execute(
        "INSERT INTO metadata (key, value) VALUES ('source_edition', ?1)",
        [EDITION],
    )?;
    transaction.execute(
        "INSERT INTO metadata (key, value) VALUES ('source_url', ?1)",
        [SOURCE_URL],
    )?;
    transaction.execute(
        "INSERT INTO metadata (key, value) VALUES ('definition_language', 'en')",
        [],
    )?;
    transaction.execute(
        "INSERT INTO metadata (key, value) VALUES ('lexical_language', 'es-MX')",
        [],
    )?;
    let mut counts = Counts::default();

    {
        let mut insert_entry = transaction.prepare_cached(
            "INSERT OR IGNORE INTO entries
             (key, word, pos, etymology_number, forms_json, tags_json, source_edition, source_line)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        let mut insert_sense = transaction.prepare_cached(
            "INSERT OR IGNORE INTO senses
             (key, entry_key, sense_index, definition_en, glosses_json,
              tags_json, raw_tags_json, examples_json, mexican_usage)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;
        for (offset, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            counts.lines += 1;
            let record: Value = serde_json::from_str(&line)
                .map_err(|error| format!("invalid JSONL at source line {}: {error}", offset + 1))?;
            if record.get("lang_code").and_then(Value::as_str) != Some("es") {
                continue;
            }
            counts.spanish_records += 1;
            let Some(word) = record.get("word").and_then(Value::as_str) else {
                counts.skipped_without_definition += 1;
                continue;
            };
            if word.trim().is_empty() {
                counts.skipped_without_definition += 1;
                continue;
            }
            let pos = record
                .get("pos")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let etymology = record.get("etymology_number").and_then(Value::as_i64);
            let ety_text = etymology.map(|n| n.to_string()).unwrap_or_default();
            let Some(senses) = record.get("senses").and_then(Value::as_array) else {
                counts.skipped_without_definition += 1;
                continue;
            };
            let useful: Vec<(usize, &Value, Vec<String>)> = senses
                .iter()
                .enumerate()
                .filter_map(|(index, sense)| {
                    let glosses = strings(sense.get("glosses"));
                    if glosses.is_empty() {
                        None
                    } else {
                        Some((index, sense, glosses))
                    }
                })
                .collect();
            if useful.is_empty() {
                counts.skipped_without_definition += 1;
                continue;
            }
            // Source-stable when the lexical identity and first gloss remain unchanged.
            // Future synchronization must reconcile changed identities before migrating reviews.
            let entry_key = fingerprint(&[EDITION, "es", word, pos, &ety_text, &useful[0].2[0]]);
            let inserted = insert_entry.execute(params![
                entry_key,
                word,
                pos,
                etymology,
                encoded(record.get("forms")),
                encoded(record.get("tags")),
                EDITION,
                (offset + 1) as i64,
            ])?;
            if inserted != 0 {
                counts.entries += 1;
            }
            for (index, sense, glosses) in useful {
                let primary_gloss = glosses.join("; ");
                let sense_key = sense
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| {
                        fingerprint(&[&entry_key, &index.to_string(), &primary_gloss])
                    });
                let mexican = tagged_mexican(sense);
                let inserted = insert_sense.execute(params![
                    sense_key,
                    entry_key,
                    index as i64,
                    primary_gloss,
                    json!(glosses).to_string(),
                    encoded(sense.get("tags")),
                    encoded(sense.get("raw_tags")),
                    encoded(sense.get("examples")),
                    i64::from(mexican),
                ])?;
                if inserted != 0 {
                    counts.senses += 1;
                    if mexican {
                        counts.mexican_senses += 1;
                    }
                }
            }
        }
    }
    transaction.commit()?;
    Ok(counts)
}

fn lookup(connection: &Connection, word: &str) -> Result<()> {
    let mut statement = connection.prepare(
        "SELECT e.word, e.pos, s.definition_en, s.mexican_usage
         FROM entries e JOIN senses s ON s.entry_key = e.key
         WHERE e.word = ?1 ORDER BY e.pos, s.sense_index",
    )?;
    let results = statement.query_map([word], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)? != 0,
        ))
    })?;
    let mut found = false;
    for result in results {
        let (word, pos, definition, mexican) = result?;
        println!(
            "{word} [{pos}]{}: {definition}",
            if mexican { " [Mexico]" } else { "" }
        );
        found = true;
    }
    if !found {
        println!("No matching entries.");
    }
    Ok(())
}

fn cell(value: &str) -> String {
    value.replace('\t', " ").replace(['\r', '\n'], " ")
}

// The trainer owns review state. This export is stateless and may be regenerated.
fn export_tsv<W: Write>(connection: &Connection, mut output: W, mexican_only: bool) -> Result<()> {
    writeln!(
        output,
        "sense_key\tword_es\tpart_of_speech\tdefinition_en\tmexican_usage"
    )?;
    let sql = if mexican_only {
        "SELECT s.key, e.word, e.pos, s.definition_en, s.mexican_usage
         FROM entries e JOIN senses s ON s.entry_key = e.key
         WHERE s.mexican_usage = 1 ORDER BY e.word, e.pos, s.sense_index"
    } else {
        "SELECT s.key, e.word, e.pos, s.definition_en, s.mexican_usage
         FROM entries e JOIN senses s ON s.entry_key = e.key
         ORDER BY e.word, e.pos, s.sense_index"
    };
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;
    for row in rows {
        let (id, word, pos, definition, mexican) = row?;
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}",
            cell(&id),
            cell(&word),
            cell(&pos),
            cell(&definition),
            mexican
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const FIXTURE: &str = include_str!("../tests/fixtures/sample.jsonl");

    #[test]
    fn imports_only_spanish_and_keeps_regional_scope_per_sense() {
        let mut db = Connection::open_in_memory().unwrap();
        let counts = import_reader(&mut db, Cursor::new(FIXTURE)).unwrap();
        assert_eq!(counts.spanish_records, 2);
        assert_eq!(counts.entries, 2);
        assert_eq!(counts.senses, 3);
        assert_eq!(counts.mexican_senses, 1);
        let mut output = Vec::new();
        export_tsv(&db, &mut output, true).unwrap();
        let tsv = String::from_utf8(output).unwrap();
        assert!(tsv.contains("sidewalk"));
        assert!(!tsv.contains("footstool"));
        assert!(!tsv.contains("house"));
    }

    #[test]
    fn repeated_import_replaces_instead_of_duplicating() {
        let mut db = Connection::open_in_memory().unwrap();
        import_reader(&mut db, Cursor::new(FIXTURE)).unwrap();
        import_reader(&mut db, Cursor::new(FIXTURE)).unwrap();
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM senses", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn bad_json_rolls_back_without_destroying_old_data() {
        let mut db = Connection::open_in_memory().unwrap();
        import_reader(&mut db, Cursor::new(FIXTURE)).unwrap();
        assert!(import_reader(&mut db, Cursor::new("{not json}\n")).is_err());
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM senses", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn checks_region_labels_not_unrelated_text() {
        assert!(tagged_mexican(&json!({"tags": ["Mexico"]})));
        assert!(tagged_mexican(&json!({"raw_tags": ["Mexican Spanish"]})));
        assert!(!tagged_mexican(&json!({"tags": ["New Mexico"]})));
        assert!(!tagged_mexican(&json!({"glosses": ["A city in Mexico"]})));
    }

    #[test]
    fn ids_are_deterministic() {
        assert_eq!(
            fingerprint(&["banqueta", "noun"]),
            fingerprint(&["banqueta", "noun"])
        );
        assert_ne!(
            fingerprint(&["banqueta", "noun"]),
            fingerprint(&["banqueta", "verb"])
        );
    }
}
