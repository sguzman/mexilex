//! Conservative cross-snapshot reconciliation for consumers of Mexilex study exports.
//! The output is a *report*, never a mutation of private trainer progress.
//!
//! Exact lexical IDs match automatically. A changed lexical ID may have a
//! candidate with matching word, POS, etymology number and source edition,
//! but this is deliberately marked for review, not silently migrated.
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Identity {
    source: String,
    word: String,
    pos: String,
    etymology: Option<i64>,
}

#[derive(Debug, Clone)]
struct Entry {
    id: String,
    identity: Identity,
    senses: BTreeMap<String, String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct CompareStats {
    pub(crate) previous: usize,
    pub(crate) current: usize,
    pub(crate) exact: usize,
    pub(crate) candidate_review: usize,
    pub(crate) ambiguous: usize,
    pub(crate) id_conflicts: usize,
    pub(crate) missing: usize,
    pub(crate) changed_definitions: usize,
    pub(crate) missing_senses: usize,
    pub(crate) added_entries: usize,
}

fn required<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {key}").into())
}

fn read_study<R: BufRead>(reader: R) -> Result<BTreeMap<String, Entry>> {
    let mut entries = BTreeMap::new();
    for (offset, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(&line)
            .map_err(|error| format!("invalid study JSONL line {}: {error}", offset + 1))?;
        if row.get("format_version").and_then(Value::as_u64) != Some(1)
            || required(&row, "lexical_language")? != "es-MX"
            || required(&row, "definition_language")? != "en"
            || required(&row, "source")? != "mexilex"
        {
            return Err(format!("unexpected study export contract at line {}", offset + 1).into());
        }
        let id = required(&row, "entry_id")?.to_owned();
        if id.is_empty() {
            return Err(format!("empty entry ID at line {}", offset + 1).into());
        }
        let identity = Identity {
            source: required(&row, "source_edition")?.to_owned(),
            word: required(&row, "word")?.to_owned(),
            pos: required(&row, "part_of_speech")?.to_owned(),
            etymology: row.get("etymology_number").and_then(Value::as_i64),
        };
        if identity.word.is_empty() {
            return Err(format!("empty headword at line {}", offset + 1).into());
        }
        let raw_senses = row
            .get("senses")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("missing senses at line {}", offset + 1))?;
        let mut senses = BTreeMap::new();
        if raw_senses.is_empty() {
            return Err(format!("zero senses at line {}", offset + 1).into());
        }
        for sense in raw_senses {
            let sense_id = required(sense, "id")?.to_owned();
            let gloss = required(sense, "definition_en")?.to_owned();
            if sense_id.is_empty() || gloss.is_empty() {
                return Err(format!("empty sense ID or gloss at line {}", offset + 1).into());
            }
            if senses.insert(sense_id, gloss).is_some() {
                return Err(format!("duplicate sense ID in entry at line {}", offset + 1).into());
            }
        }
        if entries
            .insert(
                id.clone(),
                Entry {
                    id,
                    identity,
                    senses,
                },
            )
            .is_some()
        {
            return Err(format!("duplicate lexical entry ID at line {}", offset + 1).into());
        }
    }
    Ok(entries)
}

pub(crate) fn compare_readers<O: BufRead, N: BufRead, W: Write>(
    old_reader: O,
    new_reader: N,
    mut out: W,
) -> Result<CompareStats> {
    let old = read_study(old_reader)?;
    let new = read_study(new_reader)?;
    let mut by_identity: HashMap<Identity, Vec<&Entry>> = HashMap::new();
    for entry in new.values() {
        by_identity
            .entry(entry.identity.clone())
            .or_default()
            .push(entry);
    }
    let mut stats = CompareStats {
        previous: old.len(),
        current: new.len(),
        added_entries: new.keys().filter(|key| !old.contains_key(*key)).count(),
        ..CompareStats::default()
    };

    for entry in old.values() {
        let exact = new.get(&entry.id);
        let collision = exact.is_some_and(|target| target.identity != entry.identity);
        let candidates: Vec<&Entry> = if exact.is_none() {
            by_identity
                .get(&entry.identity)
                .cloned()
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let (status, target) = if collision {
            stats.id_conflicts += 1;
            ("id_conflict", None)
        } else if let Some(target) = exact {
            stats.exact += 1;
            ("exact", Some(target))
        } else if candidates.len() == 1 {
            stats.candidate_review += 1;
            ("candidate_review", Some(candidates[0]))
        } else if candidates.len() > 1 {
            stats.ambiguous += 1;
            ("ambiguous", None)
        } else {
            stats.missing += 1;
            ("missing", None)
        };

        let mut senses = Vec::new();
        if let Some(target) = target {
            for (id, old_gloss) in &entry.senses {
                match target.senses.get(id) {
                    Some(new_gloss) if new_gloss == old_gloss => {
                        senses.push(json!({"old_id":id,"new_id":id,"status":"same_definition"}));
                    }
                    Some(_) => {
                        stats.changed_definitions += 1;
                        senses.push(json!({"old_id":id,"new_id":id,"status":"changed_definition"}));
                    }
                    None => {
                        stats.missing_senses += 1;
                        senses.push(json!({"old_id":id,"new_id":null,"status":"missing"}));
                    }
                }
            }
        }
        let options: Vec<&str> = if status == "ambiguous" {
            candidates
                .iter()
                .map(|candidate| candidate.id.as_str())
                .collect()
        } else {
            Vec::new()
        };
        let row = json!({
            "format_version":1,
            "old_entry_id":entry.id,
            "new_entry_id":target.map(|target|target.id.as_str()),
            "status":status,
            "word":entry.identity.word,
            "part_of_speech":entry.identity.pos,
            "source_edition":entry.identity.source,
            "candidate_ids":options,
            "old_sense_count":entry.senses.len(),
            "new_sense_count":target.map(|candidate|candidate.senses.len()),
            "senses":senses
        });
        serde_json::to_writer(&mut out, &row)?;
        out.write_all(b"\n")?;
    }
    Ok(stats)
}

pub(crate) fn compare_files(old: &Path, new: &Path, destination: &Path) -> Result<CompareStats> {
    if old == destination || new == destination || destination.exists() {
        return Err("comparison output must be a new file, separate from both inputs".into());
    }
    if let Some(parent) = destination.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let result = (|| -> Result<CompareStats> {
        let old_file = BufReader::new(File::open(old)?);
        let new_file = BufReader::new(File::open(new)?);
        let mut output = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination)?,
        );
        let stats = compare_readers(old_file, new_file, &mut output)?;
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
    use std::io::Cursor;

    fn entry(id: &str, word: &str, pos: &str, senses: &[(&str, &str)]) -> String {
        let senses: Vec<Value> = senses
            .iter()
            .map(|(id, gloss)| json!({"id":id,"definition_en":gloss}))
            .collect();
        json!({
            "format_version":1,"source":"mexilex","source_edition":"enwiktionary",
            "lexical_language":"es-MX","definition_language":"en",
            "entry_id":id,"word":word,"part_of_speech":pos,
            "etymology_number":null,"senses":senses
        })
        .to_string()
    }

    fn compare(old: &str, new: &str) -> (CompareStats, Vec<Value>) {
        let mut bytes = Vec::new();
        let counts = compare_readers(Cursor::new(old), Cursor::new(new), &mut bytes).unwrap();
        let report: Vec<Value> = String::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        (counts, report)
    }

    #[test]
    fn exact_snapshot_retains_source_ids_and_glosses() {
        let snapshot = format!(
            "{}\n",
            entry("old", "banqueta", "noun", &[("s1", "sidewalk")])
        );
        let (stats, report) = compare(&snapshot, &snapshot);
        assert_eq!(stats.exact, 1);
        assert_eq!(stats.added_entries, 0);
        assert_eq!(report[0]["status"], "exact");
        assert_eq!(report[0]["senses"][0]["status"], "same_definition");
    }

    #[test]
    fn changed_gloss_never_silently_preserves_old_card_content() {
        let old = entry("stable", "chamba", "noun", &[("id1", "work")]);
        let new = entry("stable", "chamba", "noun", &[("id1", "a job")]);
        let (stats, report) = compare(&old, &new);
        assert_eq!(stats.changed_definitions, 1);
        assert_eq!(report[0]["senses"][0]["status"], "changed_definition");
    }

    #[test]
    fn altered_entry_id_only_proposes_a_candidate_for_review() {
        let old = entry("old", "banqueta", "noun", &[("s1", "sidewalk")]);
        let new = entry("new", "banqueta", "noun", &[("s1", "sidewalk")]);
        let (stats, report) = compare(&old, &new);
        assert_eq!(stats.candidate_review, 1);
        assert_eq!(report[0]["status"], "candidate_review");
        assert_eq!(report[0]["new_entry_id"], "new");
        assert_eq!(report[0]["senses"][0]["status"], "same_definition");
    }

    #[test]
    fn homographs_with_multiple_candidates_are_ambiguous() {
        let old = entry("old", "cura", "noun", &[("s1", "priest")]);
        let new = format!(
            "{}\n{}",
            entry("new-a", "cura", "noun", &[("s1", "priest")]),
            entry("new-b", "cura", "noun", &[("s2", "cure")])
        );
        let (stats, report) = compare(&old, &new);
        assert_eq!(stats.ambiguous, 1);
        assert_eq!(report[0]["new_entry_id"], Value::Null);
        assert_eq!(report[0]["candidate_ids"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn deletions_remain_in_report_and_invalid_input_is_rejected() {
        let old = entry("old", "chale", "interjection", &[("s1", "damn")]);
        let (stats, report) = compare(&old, "");
        assert_eq!(stats.missing, 1);
        assert_eq!(report[0]["status"], "missing");
        let duplicate = format!("{old}\n{old}");
        assert!(compare_readers(Cursor::new(&duplicate), Cursor::new(&old), Vec::new()).is_err());
        assert!(compare_readers(Cursor::new("not json"), Cursor::new(&old), Vec::new()).is_err());
    }

    #[test]
    fn reused_entry_id_with_different_headword_is_not_treated_as_exact() {
        let old = entry("same-id", "cura", "noun", &[("s1", "priest")]);
        let new = entry("same-id", "curar", "verb", &[("s1", "to cure")]);
        let (stats, report) = compare(&old, &new);
        assert_eq!(stats.exact, 0);
        assert_eq!(stats.id_conflicts, 1);
        assert_eq!(report[0]["status"], "id_conflict");
        assert_eq!(report[0]["new_entry_id"], Value::Null);
    }

    #[test]
    fn missing_sense_does_not_silently_reassign_reviews() {
        let old = entry(
            "x",
            "banco",
            "noun",
            &[("financial", "bank"), ("seat", "bench")],
        );
        let new = entry("x", "banco", "noun", &[("financial", "bank")]);
        let (stats, report) = compare(&old, &new);
        assert_eq!(stats.missing_senses, 1);
        assert_eq!(report[0]["senses"][1]["status"], "missing");
    }
}
