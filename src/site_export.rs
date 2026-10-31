//! Build a small, lazy-loaded read-only website from the Git-tracked corpus.
//!
//! Only the search index and HTML/CSS/JS are deployed. Full lexical records are
//! requested on demand from the source commit, keeping Pages well under the
//! platform's published-site size limit.
use crate::corpus_load;
use rusqlite::types::Value as SqlValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct SiteStats {
    entries: usize,
    mexico_labeled_entries: usize,
    inflected_entries: usize,
    index_buckets: usize,
}

struct SearchRecord {
    word: String,
    key: String,
    pos: String,
    gloss: String,
    mexico: bool,
    form: bool,
    normalized: String,
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| match character {
            'á' | 'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

fn bucket(normalized: &str) -> Option<String> {
    let mut chars = normalized.chars();
    let a = chars.next()?;
    let b = chars.next()?;
    let clean = |c: char| if c.is_ascii_alphanumeric() { c } else { '_' };
    Some([clean(a), clean(b)].into_iter().collect())
}

fn str_field<'a>(record: &'a Value, field: &str) -> Result<&'a str> {
    record
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("corpus record missing {field}").into())
}

fn is_form(senses: &[Value]) -> bool {
    !senses.is_empty()
        && senses.iter().all(|sense| {
            sense
                .get("tags")
                .and_then(Value::as_array)
                .is_some_and(|tags| tags.iter().any(|tag| tag.as_str() == Some("form-of")))
        })
}

pub(crate) fn build_site(corpus: &Path, destination: &Path, commit: &str) -> Result<SiteStats> {
    if commit.len() != 40 || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected a full 40-digit Git commit SHA".into());
    }
    if destination.exists() && fs::read_dir(destination)?.next().is_some() {
        return Err(format!(
            "site output must be empty (refusing to overwrite {})",
            destination.display()
        )
        .into());
    }

    let manifest: Value = serde_json::from_reader(File::open(corpus.join("manifest.json"))?)?;
    if manifest.get("format_version").and_then(Value::as_u64) != Some(1) {
        return Err("unsupported normalized corpus schema".into());
    }
    if str_field(&manifest, "lexical_language")? != "es-MX"
        || str_field(&manifest, "definition_language")? != "en"
    {
        return Err("site requires es-MX vocabulary with English definitions".into());
    }
    let mut groups: BTreeMap<String, Vec<SearchRecord>> = BTreeMap::new();
    let mut stats = SiteStats::default();

    for path in corpus_load::shards(corpus)? {
        for (line_index, line) in BufReader::new(File::open(&path)?).lines().enumerate() {
            let line = line?;
            let record: Value = serde_json::from_str(&line).map_err(|error| {
                format!("{}:{}: {error}", path.display(), line_index + 1)
            })?;
            let word = str_field(&record, "word")?;
            let id = str_field(&record, "id")?;
            if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(format!("invalid lexical key in {}", path.display()).into());
            }
            let senses = record
                .get("senses")
                .and_then(Value::as_array)
                .ok_or("missing senses array")?;
            let Some(first_sense) = senses.first() else {
                return Err("entry is missing all senses".into());
            };
            let gloss = str_field(first_sense, "definition_en")?;
            let mexican = senses.iter().any(|sense| {
                sense.get("mexican_usage").and_then(Value::as_bool) == Some(true)
            });
            let form = is_form(senses);
            stats.entries += 1;
            if mexican {
                stats.mexico_labeled_entries += 1;
            }
            if form {
                stats.inflected_entries += 1;
            }
            let normalized = normalize(word);
            if let Some(prefix) = bucket(&normalized) {
                groups.entry(prefix).or_default().push(SearchRecord {
                    word: word.to_owned(),
                    key: id.to_owned(),
                    pos: str_field(&record, "part_of_speech")?.to_owned(),
                    gloss: gloss.chars().take(140).collect(),
                    mexico: mexican,
                    form,
                    normalized,
                });
            }
        }
    }
    let expected = manifest
        .get("entry_count")
        .and_then(Value::as_u64)
        .ok_or("manifest missing entry count")?;
    if stats.entries as u64 != expected {
        return Err(format!("site corpus is incomplete: expected {expected}, found {}", stats.entries).into());
    }

    fs::create_dir_all(destination.join("index"))?;
    for (prefix, mut records) in groups {
        records.sort_unstable_by(|a, b| {
            a.normalized
                .cmp(&b.normalized)
                .then(a.form.cmp(&b.form))
                .then(a.pos.cmp(&b.pos))
                .then(a.key.cmp(&b.key))
        });
        let mut writer = BufWriter::new(File::create(
            destination.join("index").join(format!("{prefix}.json")),
        )?);
        writer.write_all(b"[")?;
        for (index, record) in records.into_iter().enumerate() {
            if index != 0 {
                writer.write_all(b",")?;
            }
            serde_json::to_writer(
                &mut writer,
                &json!([
                    record.word,
                    record.key,
                    record.pos,
                    record.gloss,
                    record.mexico,
                    record.form
                ]),
            )?;
        }
        writer.write_all(b"]\n")?;
        writer.flush()?;
        stats.index_buckets += 1;
    }
    for (name, content) in [
        ("index.html", include_str!("../site/index.html")),
        ("style.css", include_str!("../site/style.css")),
        ("app.js", include_str!("../site/app.js")),
        ("favicon.svg", include_str!("../site/favicon.svg")),
    ] {
        fs::write(destination.join(name), content)?;
    }
    let config = json!({
        "commit": commit.to_ascii_lowercase(),
        "entries": stats.entries,
        "senses": manifest["sense_count"],
        "mexican_senses": manifest["mexico_labeled_sense_count"],
        "index_buckets": stats.index_buckets,
        "mexico_labeled_entries": stats.mexico_labeled_entries,
        "inflected_entries": stats.inflected_entries,
        "dictionary_language": "es-MX",
        "definition_language": "en"
    });
    fs::write(
        destination.join("config.json"),
        format!("{}\n", serde_json::to_string_pretty(&config)?),
    )?;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn site_index_shows_mexico_tagged_senses_and_general_words() {
        let mut db = Connection::open_in_memory().unwrap();
        crate::import_reader(
            &mut db,
            Cursor::new(include_str!("../tests/fixtures/sample.jsonl")),
        )
        .unwrap();
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("mexilex-site-{}-{nanos}", std::process::id()));
        let corpus = path.join("corpus");
        let site = path.join("site");
        crate::corpus::export_corpus(&db, &corpus).unwrap();
        let stats = build_site(&corpus, &site, &"a".repeat(40)).unwrap();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.mexico_labeled_entries, 1);
        assert_eq!(stats.index_buckets, 2);
        let banqueta: Value =
            serde_json::from_reader(File::open(site.join("index/ba.json")).unwrap()).unwrap();
        assert_eq!(banqueta[0][0], "banqueta");
        assert_eq!(banqueta[0][4], true);
        assert_eq!(banqueta[0][5], false);
        let house: Value =
            serde_json::from_reader(File::open(site.join("index/ca.json")).unwrap()).unwrap();
        assert_eq!(house[0][0], "casa");
        assert_eq!(house[0][4], false);
        let config: Value =
            serde_json::from_reader(File::open(site.join("config.json")).unwrap()).unwrap();
        assert_eq!(config["entries"], 2);
        assert_eq!(config["dictionary_language"], "es-MX");
        assert_eq!(config["commit"], "a".repeat(40));
        assert!(build_site(&corpus, &site, &"a".repeat(40)).is_err());
        assert!(build_site(&corpus, &path.join("other"), "bad").is_err());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn normalizes_search_prefixes_without_erasing_source_spelling() {
        assert_eq!(normalize("Niño"), "nino");
        assert_eq!(normalize("ÁRBOL"), "arbol");
        assert_eq!(bucket("áx"), Some("_x".to_string()));
        assert_eq!(bucket(&normalize("árbol")), Some("ar".to_string()));
        assert_eq!(bucket(&normalize("ñame")), Some("na".to_string()));
    }
}
