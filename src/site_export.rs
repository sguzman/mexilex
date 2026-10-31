//! Build a small, lazy-loaded read-only website from the Git-tracked corpus.
//!
//! Only the search index and HTML/CSS/JS are deployed. Full lexical records are
//! requested on demand from the source commit, keeping Pages well under the
//! platform's published-site size limit.
use crate::corpus_load;
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
    form_buckets: usize,
}

struct SearchRecord {
    word: String,
    key: String,
    pos: String,
    gloss: String,
    mexico: bool,
    form: bool,
    normalized: String,
    frequency: Option<[f64; 4]>,
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
            'ñ' => 'ñ',
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

fn write_indexes(groups: BTreeMap<String, Vec<SearchRecord>>, directory: &Path) -> Result<usize> {
    fs::create_dir_all(directory)?;
    let mut count = 0;
    for (prefix, mut records) in groups {
        records.sort_unstable_by(|a, b| {
            a.normalized
                .cmp(&b.normalized)
                .then(a.pos.cmp(&b.pos))
                .then(a.key.cmp(&b.key))
        });
        let mut writer = BufWriter::new(File::create(directory.join(format!("{prefix}.json")))?);
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
                    record.form,
                    record.frequency
                ]),
            )?;
        }
        writer.write_all(b"]\n")?;
        writer.flush()?;
        count += 1;
    }
    Ok(count)
}

fn load_mexican_frequency(corpus: &Path) -> Result<BTreeMap<String, [f64; 4]>> {
    let base = corpus.parent().unwrap_or_else(|| Path::new("."));
    let path = base.join("evidence/activ-es-v02.jsonl");
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let manifest: Value = serde_json::from_reader(File::open(
        base.join("evidence/activ-es-v02.manifest.json"),
    )?)?;
    if manifest.get("format_version").and_then(Value::as_u64) != Some(1)
        || manifest.get("source_commit").and_then(Value::as_str)
            != Some(crate::evidence::SOURCE_COMMIT)
    {
        return Err("unsupported Mexican frequency evidence provenance".into());
    }
    let mut by_word = BTreeMap::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let row: Value = serde_json::from_str(&line?)?;
        let cells = row
            .as_array()
            .ok_or("expected one Mexican evidence array per line")?;
        if cells.len() != 5 {
            return Err("expected five Mexican frequency fields".into());
        }
        let word = cells[0]
            .as_str()
            .ok_or("missing frequency headword")?
            .to_lowercase();
        let mut rates = [0.0; 4];
        for (index, rate) in rates.iter_mut().enumerate() {
            *rate = cells[index + 1]
                .as_f64()
                .ok_or("missing frequency metric")?;
            if !rate.is_finite() || *rate < 0.0 {
                return Err("invalid Mexican dialogue frequency metric".into());
            }
        }
        if rates[0] <= 0.0 {
            return Err("Mexico attestation must be positive".into());
        }
        if by_word.insert(word, rates).is_some() {
            return Err("duplicate Mexico evidence word".into());
        }
    }
    if manifest["attested_mexico"].as_u64() != Some(by_word.len() as u64) {
        return Err("Mexico evidence count differs from provenance manifest".into());
    }
    Ok(by_word)
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
    let mut inflection_groups: BTreeMap<String, Vec<SearchRecord>> = BTreeMap::new();
    let frequency = load_mexican_frequency(corpus)?;
    let mut stats = SiteStats::default();
    let mut mexican_browse: Vec<Value> = Vec::new();
    let mut dialogue_browse: BTreeMap<String, Value> = BTreeMap::new();

    for path in corpus_load::shards(corpus)? {
        for (line_index, line) in BufReader::new(File::open(&path)?).lines().enumerate() {
            let line = line?;
            let record: Value = serde_json::from_str(&line)
                .map_err(|error| format!("{}:{}: {error}", path.display(), line_index + 1))?;
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
            let mexican_sense = senses
                .iter()
                .find(|sense| sense.get("mexican_usage").and_then(Value::as_bool) == Some(true));
            // Search previews for Mexico-tagged entries must show the Mexican
            // meaning, not an unrelated generic sense that happens to come first.
            let gloss = str_field(mexican_sense.unwrap_or(first_sense), "definition_en")?;
            let mexican = mexican_sense.is_some();
            let form = crate::lexical::is_form(senses);
            stats.entries += 1;
            if mexican {
                stats.mexico_labeled_entries += 1;
            }
            if form {
                stats.inflected_entries += 1;
            }
            let normalized = normalize(word);
            if let Some(prefix) = bucket(&normalized) {
                if mexican && !form {
                    mexican_browse.push(json!([
                        word,
                        id,
                        str_field(&record, "part_of_speech")?,
                        gloss.chars().take(140).collect::<String>(),
                        true,
                        false
                    ]));
                }
                if !form {
                    if let Some(rate) = frequency.get(&word.to_lowercase()) {
                        let candidate = json!([
                            word, id, str_field(&record, "part_of_speech")?,
                            gloss.chars().take(140).collect::<String>(), mexican, false, rate
                        ]);
                        let slot = dialogue_browse.entry(word.to_lowercase()).or_insert(Value::Null);
                        if slot.is_null() || (mexican && slot[4] != true) {
                            *slot = candidate;
                        }
                    }
                }
                let target = if form {
                    &mut inflection_groups
                } else {
                    &mut groups
                };
                target.entry(prefix).or_default().push(SearchRecord {
                    word: word.to_owned(),
                    key: id.to_owned(),
                    pos: str_field(&record, "part_of_speech")?.to_owned(),
                    gloss: gloss.chars().take(140).collect(),
                    mexico: mexican,
                    form,
                    normalized,
                    frequency: frequency.get(&word.to_lowercase()).copied(),
                });
            }
        }
    }
    let expected = manifest
        .get("entry_count")
        .and_then(Value::as_u64)
        .ok_or("manifest missing entry count")?;
    if stats.entries as u64 != expected {
        return Err(format!(
            "site corpus is incomplete: expected {expected}, found {}",
            stats.entries
        )
        .into());
    }

    stats.index_buckets = write_indexes(groups, &destination.join("index"))?;
    stats.form_buckets = write_indexes(inflection_groups, &destination.join("forms"))?;
    // A small, independently downloadable browse list for discovering Mexico-tagged
    // headwords without knowing an initial query. No word list is hand-maintained.
    mexican_browse.sort_unstable_by(|a, b| {
        normalize(a[0].as_str().unwrap_or("")).cmp(&normalize(b[0].as_str().unwrap_or("")))
    });
    fs::write(
        destination.join("mexico.json"),
        format!("{}\n", serde_json::to_string(&mexican_browse)?),
    )?;
    // Top attested headwords are distinct from Mexico-specific sense labels.
    let mut spoken: Vec<Value> = dialogue_browse.into_values().collect();
    spoken.sort_unstable_by(|a, b| {
        b[6][0].as_f64().unwrap_or(0.0)
            .total_cmp(&a[6][0].as_f64().unwrap_or(0.0))
            .then(normalize(a[0].as_str().unwrap_or("")).cmp(&normalize(b[0].as_str().unwrap_or(""))))
    });
    spoken.truncate(500);
    fs::write(
        destination.join("dialogue.json"),
        format!("{}\n", serde_json::to_string(&spoken)?),
    )?;
    for (name, content) in [
        ("index.html", include_str!("../site/index.html")),
        ("style.css", include_str!("../site/style.css")),
        ("app.js", include_str!("../site/app.js")),
        ("search.mjs", include_str!("../site/search.mjs")),
        ("favicon.svg", include_str!("../site/favicon.svg")),
        ("flag-license.txt", include_str!("../site/flag-license.txt")),
        (
            "mexico-frequency-license.txt",
            include_str!("../evidence/ACTIV-ES-LICENSE"),
        ),
    ] {
        fs::write(destination.join(name), content)?;
    }
    let config = json!({
        "commit": commit.to_ascii_lowercase(),
        "entries": stats.entries,
        "senses": manifest["sense_count"],
        "mexican_senses": manifest["mexico_labeled_sense_count"],
        "index_buckets": stats.index_buckets,
        "form_buckets": stats.form_buckets,
        "mexico_labeled_entries": stats.mexico_labeled_entries,
        "inflected_entries": stats.inflected_entries,
        "mexico_dialogue_headwords": frequency.len(),
        "frequency_source": if frequency.is_empty() { Value::Null } else { json!("ACTIV-ES v0.2, Mexican film dialogue; GPL-2.0") },
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
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("mexilex-site-{}-{nanos}", std::process::id()));
        let corpus = path.join("corpus");
        let site = path.join("site");
        crate::corpus::export_corpus(&db, &corpus).unwrap();
        let stats = build_site(&corpus, &site, &"a".repeat(40)).unwrap();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.mexico_labeled_entries, 1);
        assert_eq!(stats.index_buckets, 2);
        assert_eq!(stats.form_buckets, 0);
        let featured: Value =
            serde_json::from_reader(File::open(site.join("mexico.json")).unwrap()).unwrap();
        assert_eq!(featured.as_array().unwrap().len(), 1);
        assert_eq!(featured[0][0], "banqueta");
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
    fn external_frequency_evidence_appears_without_relabeling_general_spanish() {
        let mut db = Connection::open_in_memory().unwrap();
        crate::import_reader(
            &mut db,
            Cursor::new(include_str!("../tests/fixtures/sample.jsonl")),
        )
        .unwrap();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("mexilex-evidence-{}-{nanos}", std::process::id()));
        let corpus = root.join("corpus");
        let site = root.join("site");
        let evidence = root.join("evidence");
        crate::corpus::export_corpus(&db, &corpus).unwrap();
        fs::create_dir_all(&evidence).unwrap();
        fs::write(
            evidence.join("activ-es-v02.jsonl"),
            "[\"banqueta\",1.06,0.0,0.12,0.42]\n[\"casa\",120.12,160.43,154.62,8.99]\n",
        )
        .unwrap();
        fs::write(
            evidence.join("activ-es-v02.manifest.json"),
            format!(
                "{{\"format_version\":1,\"source_commit\":\"{}\",\"attested_mexico\":2}}",
                crate::evidence::SOURCE_COMMIT
            ),
        )
        .unwrap();
        build_site(&corpus, &site, &"e".repeat(40)).unwrap();
        let banqueta: Value =
            serde_json::from_reader(File::open(site.join("index/ba.json")).unwrap()).unwrap();
        let casa: Value =
            serde_json::from_reader(File::open(site.join("index/ca.json")).unwrap()).unwrap();
        let config: Value =
            serde_json::from_reader(File::open(site.join("config.json")).unwrap()).unwrap();
        assert_eq!(config["mexico_dialogue_headwords"], 2);
        assert_eq!(banqueta[0][6][0], 1.06);
        assert_eq!(banqueta[0][4], true);
        assert_eq!(casa[0][6][0], 120.12);
        assert_eq!(casa[0][4], false); // frequency is not a Mexico-specific sense
        let dialogue: Value =
            serde_json::from_reader(File::open(site.join("dialogue.json")).unwrap()).unwrap();
        assert_eq!(dialogue[0][0], "casa");
        assert_eq!(dialogue[1][0], "banqueta");
        assert_eq!(dialogue[0][4], false);
        assert!(site.join("mexico-frequency-license.txt").exists());
        fs::write(
            evidence.join("activ-es-v02.manifest.json"),
            "{\"format_version\":1,\"source_commit\":\"wrong\",\"attested_mexico\":2}",
        )
        .unwrap();
        assert!(build_site(&corpus, &root.join("other-site"), &"e".repeat(40)).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inflected_forms_are_separate_from_default_search() {
        let mut db = Connection::open_in_memory().unwrap();
        let fixture = concat!(
            "{\"lang_code\":\"es\",\"word\":\"caminar\",\"pos\":\"verb\",\"senses\":[{\"glosses\":[\"to walk\"],\"tags\":[]}]}\n",
            "{\"lang_code\":\"es\",\"word\":\"caminé\",\"pos\":\"verb\",\"senses\":[{\"glosses\":[\"first-person preterite of caminar\"],\"tags\":[\"form-of\"]}]}\n"
        );
        crate::import_reader(&mut db, Cursor::new(fixture)).unwrap();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "mexilex-index-split-{}-{nanos}",
            std::process::id()
        ));
        let source = root.join("corpus");
        let target = root.join("site");
        crate::corpus::export_corpus(&db, &source).unwrap();
        let result = build_site(&source, &target, &"f".repeat(40)).unwrap();
        assert_eq!(result.entries, 2);
        assert_eq!(result.inflected_entries, 1);
        assert_eq!(result.index_buckets, 1);
        assert_eq!(result.form_buckets, 1);
        let headwords: Value =
            serde_json::from_reader(File::open(target.join("index/ca.json")).unwrap()).unwrap();
        let inflections: Value =
            serde_json::from_reader(File::open(target.join("forms/ca.json")).unwrap()).unwrap();
        assert_eq!(headwords.as_array().unwrap().len(), 1);
        assert_eq!(inflections.as_array().unwrap().len(), 1);
        assert_eq!(headwords[0][0], "caminar");
        assert_eq!(inflections[0][0], "caminé");
        assert_eq!(headwords[0][5], false);
        assert_eq!(inflections[0][5], true);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn mexico_previews_prefer_explicitly_regional_sense_even_if_it_is_not_first() {
        let mut db = Connection::open_in_memory().unwrap();
        let fixture = r#"{"lang_code":"es","word":"banqueta","pos":"noun","senses":[{"glosses":["footstool"],"tags":[]},{"glosses":["sidewalk"],"tags":["Mexico"]}]}"#;
        crate::import_reader(&mut db, Cursor::new(fixture)).unwrap();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "mexilex-regional-preview-{}-{nanos}",
            std::process::id()
        ));
        let source = root.join("corpus");
        let site = root.join("site");
        crate::corpus::export_corpus(&db, &source).unwrap();
        build_site(&source, &site, &"b".repeat(40)).unwrap();
        let idx: Value =
            serde_json::from_reader(File::open(site.join("index/ba.json")).unwrap()).unwrap();
        let browse: Value =
            serde_json::from_reader(File::open(site.join("mexico.json")).unwrap()).unwrap();
        assert_eq!(idx[0][3], "sidewalk");
        assert_eq!(browse[0][3], "sidewalk");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn normalizes_search_prefixes_without_erasing_source_spelling() {
        assert_eq!(normalize("Niño"), "niño");
        assert_eq!(normalize("ÁRBOL"), "arbol");
        assert_eq!(bucket("áx"), Some("_x".to_string()));
        assert_eq!(bucket(&normalize("árbol")), Some("ar".to_string()));
        assert_eq!(bucket(&normalize("ñame")), Some("_a".to_string()));
    }
}
