// Fail closed before a refreshed snapshot can replace the published corpus.
import assert from "node:assert/strict";
import { createReadStream, readFileSync } from "node:fs";
import { createInterface } from "node:readline";

export async function inspect(oldManifest, newManifest, oldStudy, report, maximum) {
  assert.match(String(maximum), /^(0|[1-9][0-9]*)$/, "Review budget must be a nonnegative integer");
  const max = Number(maximum);
  assert.ok(Number.isSafeInteger(max), "Review budget exceeds a safe integer");
  const old = JSON.parse(readFileSync(oldManifest, "utf8"));
  const updated = JSON.parse(readFileSync(newManifest, "utf8"));
  assert.equal(old.format_version, 1);
  assert.equal(updated.format_version, 1);
  assert.equal(old.lexical_language, "es-MX");
  assert.equal(updated.lexical_language, "es-MX");
  assert.equal(old.definition_language, "en");
  assert.equal(updated.definition_language, "en");
  assert.equal(updated.source_edition, old.source_edition);
  const ratio = updated.entry_count / old.entry_count;
  assert.ok(Number.isFinite(ratio) && ratio >= 0.85 && ratio <= 1.15,
    "Upstream entry count changed by more than 15%; refuse publication");

  const previousIds = new Set();
  for await (const line of createInterface({ input: createReadStream(oldStudy), crlfDelay: Infinity })) {
    if (!line.trim()) continue;
    const row = JSON.parse(line);
    assert.equal(row.format_version, 1, "Invalid previous study contract");
    assert.equal(row.source, "mexilex");
    assert.equal(row.lexical_language, "es-MX");
    assert.equal(row.definition_language, "en");
    assert.match(row.entry_id, /^[a-f0-9]{64}$/, "Invalid previous lexical ID");
    assert.ok(!previousIds.has(row.entry_id), "Duplicate previous lexical ID");
    previousIds.add(row.entry_id);
  }
  const previous = previousIds.size;
  const seenOld = new Set();
  const proposedTargets = new Set();
  const statuses = { exact: 0, candidate_review: 0, ambiguous: 0, id_conflict: 0, missing: 0 };
  let observed = 0;
  let requiresReview = 0;
  for await (const line of createInterface({ input: createReadStream(report), crlfDelay: Infinity })) {
    if (!line.trim()) continue;
    const row = JSON.parse(line);
    assert.equal(row.format_version, 1);
    assert.ok(previousIds.has(row.old_entry_id), "Report references an unknown previous lexical ID");
    assert.ok(!seenOld.has(row.old_entry_id), "Duplicate previous lexical ID in reconciliation report");
    seenOld.add(row.old_entry_id);
    assert.ok(Object.hasOwn(statuses, row.status), "Unknown study status");
    assert.equal(typeof row.requires_review, "boolean");
    if (row.status === "exact") {
      assert.equal(row.new_entry_id, row.old_entry_id, "Exact match must preserve lexical ID");
    } else if (row.status === "candidate_review") {
      assert.match(row.new_entry_id, /^[a-f0-9]{64}$/, "Candidate must reference one valid new lexical ID");
      assert.ok(!proposedTargets.has(row.new_entry_id), "Two old entries cannot claim the same candidate");
      proposedTargets.add(row.new_entry_id);
    } else {
      assert.equal(row.new_entry_id, null, "Unresolved status may not select a new lexical ID");
    }
    statuses[row.status]++;
    if (row.requires_review) requiresReview++;
    if (row.status !== "exact") assert.equal(row.requires_review, true,
      "Changed IDs may never bypass review");
    observed++;
  }
  assert.ok(previous > 100_000, "Previous real study export unexpectedly small");
  assert.equal(observed, previous, "One report row required per previous study entry");
  assert.equal(seenOld.size, previous, "All previous lexical IDs must be reconciled once");
  assert.equal(Object.values(statuses).reduce((sum, value) => sum + value, 0), previous);
  assert.ok(requiresReview <= max,
    "Extraction exceeds reviewed migration budget: " + requiresReview + " > " + max);
  return {
    format_version: 1, previous_entries: old.entry_count, new_entries: updated.entry_count,
    previous_study_entries: previous, review_required: requiresReview,
    approved_review_budget: max, statuses,
    corpus_unchanged_until_gates_pass: true
  };
}

if (process.argv[1]?.endsWith("guard-upstream-refresh.mjs")) {
  const args = process.argv.slice(2);
  if (args.length !== 5) throw new Error("Expected old/new manifests, old study, report, review limit");
  console.log(JSON.stringify(await inspect(...args), null, 2));
}
