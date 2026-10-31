// A real upstream change audit. Generates a conservative summary only; it does
// not mutate lexicon records, publish a Git commit, or touch trainer review data.
import assert from "node:assert/strict";
import { readFileSync, createReadStream } from "node:fs";
import { createInterface } from "node:readline";

const [, , option, ...rest] = process.argv;
if (option === "--size-check") {
  const [oldFile, newFile] = rest;
  assert.ok(oldFile && newFile);
  const oldManifest = JSON.parse(readFileSync(oldFile, "utf8"));
  const newManifest = JSON.parse(readFileSync(newFile, "utf8"));
  assert.equal(newManifest.format_version, 1);
  assert.equal(newManifest.lexical_language, "es-MX");
  assert.equal(newManifest.definition_language, "en");
  const change = newManifest.entry_count / oldManifest.entry_count;
  assert.ok(change >= .85 && change <= 1.15,
    "Upstream record count changed by more than 15%; review extraction compatibility before proceeding");
  console.log("Verified plausible upstream size:", oldManifest.entry_count, "to", newManifest.entry_count);
} else {
  const [oldStudy, newStudy, reportFile, newManifestFile, checksumFile] = [option, ...rest];
  assert.ok(oldStudy && newStudy && reportFile && newManifestFile && checksumFile);
  async function count(file) {
    let count = 0;
    for await (const line of createInterface({input: createReadStream(file), crlfDelay: Infinity})) {
      if (line.trim()) count++;
    }
    return count;
  }
  const [oldEntries, newEntries] = await Promise.all([count(oldStudy), count(newStudy)]);
  const stats = {
    exact: 0, candidate_review: 0, ambiguous: 0, id_conflict: 0, missing: 0,
    requires_review: 0, changed_definition: 0, changed_annotation: 0,
    added_senses: 0, missing_senses: 0, changed_forms: 0
  };
  let reviewed = 0;
  for await (const line of createInterface({input: createReadStream(reportFile), crlfDelay: Infinity})) {
    if (!line.trim()) continue;
    const row = JSON.parse(line);
    assert.ok(Object.hasOwn(stats, row.status), "Unexpected reconciliation status: " + row.status);
    stats[row.status]++;
    if (row.requires_review) stats.requires_review++;
    stats.changed_forms += Number(row.forms_changed === true);
    stats.added_senses += (row.added_sense_ids || []).length;
    for (const sense of row.senses || []) {
      if (Object.hasOwn(stats, sense.status)) stats[sense.status]++;
    }
    reviewed++;
  }
  assert.equal(reviewed, oldEntries, "Every old study entry must appear in the report");
  assert.equal(stats.exact + stats.candidate_review + stats.ambiguous + stats.id_conflict + stats.missing, oldEntries);
  const manifest = JSON.parse(readFileSync(newManifestFile, "utf8"));
  const checksum = readFileSync(checksumFile, "utf8").trim();
  assert.match(checksum, /^[0-9a-f]{64}$/);
  console.log(JSON.stringify({
    format_version: 1,
    analysis_kind: "actual-upstream-snapshot-comparison",
    changed_upstream_sha256: checksum,
    old_study_entries: oldEntries,
    new_study_entries: newEntries,
    new_corpus_entries: manifest.entry_count,
    new_corpus_senses: manifest.sense_count,
    stats,
    advisory_only: true,
    automatic_corpus_update: false,
    automatic_trainer_migration: false
  }, null, 2));
}
