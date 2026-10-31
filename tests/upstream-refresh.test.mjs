import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { inspect } from "../scripts/guard-upstream-refresh.mjs";

// Deliberately above the actual minimum: the production guard must never
// be weakened merely to accommodate a tiny test fixture.
test("refresh guard approves unchanged identities but rejects unreviewed changes", async () => {
  const folder = await mkdtemp(join(tmpdir(), "mexilex-refresh-guard-"));
  try {
    const oldManifest = join(folder, "previous.json");
    const nextManifest = join(folder, "candidate.json");
    const oldStudy = join(folder, "study.jsonl");
    const unchanged = join(folder, "safe.jsonl");
    const changed = join(folder, "changed.jsonl");
    const manifest = {
      format_version: 1,
      lexical_language: "es-MX",
      definition_language: "en",
      source_edition: "enwiktionary",
      entry_count: 813928
    };
    await writeFile(oldManifest, JSON.stringify(manifest));
    await writeFile(nextManifest, JSON.stringify({...manifest, entry_count: 813930}));
    const count = 100001;
    const id = index => index.toString(16).padStart(64, "0");
    const oldRows = Array.from({length: count}, (_, index) => JSON.stringify({
      format_version: 1, source: "mexilex",
      lexical_language: "es-MX", definition_language: "en",
      entry_id: id(index)
    })).join("\n") + "\n";
    const unchangedRows = Array.from({length: count}, (_, index) => JSON.stringify({
      format_version: 1, status: "exact", requires_review: false,
      old_entry_id: id(index), new_entry_id: id(index)
    })).join("\n") + "\n";
    const changedRows = Array.from({length: count}, (_, index) => JSON.stringify({
      format_version: 1, status: index === count-1 ? "missing" : "exact",
      requires_review: index === count-1,
      old_entry_id: id(index), new_entry_id: index === count-1 ? null : id(index)
    })).join("\n") + "\n";
    await Promise.all([
      writeFile(oldStudy, oldRows),
      writeFile(unchanged, unchangedRows),
      writeFile(changed, changedRows)
    ]);
    const success = await inspect(oldManifest, nextManifest, oldStudy, unchanged, "0");
    assert.equal(success.previous_study_entries, count);
    assert.equal(success.review_required, 0);
    assert.equal(success.statuses.exact, count);
    await assert.rejects(inspect(oldManifest, nextManifest, oldStudy, changed, "0"),
      /exceeds reviewed migration budget/);
    const approved = await inspect(oldManifest, nextManifest, oldStudy, changed, "1");
    assert.equal(approved.review_required, 1);
    assert.equal(approved.statuses.missing, 1);
    await assert.rejects(inspect(oldManifest, nextManifest, oldStudy, changed, "one"),
      /nonnegative integer/);
    // A report with the right row count but one missing old ID must fail.
    const duplicateRows = unchangedRows.trimEnd().split("\n");
    duplicateRows[count - 1] = duplicateRows[0];
    const duplicateReport = join(folder, "duplicate-report.jsonl");
    await writeFile(duplicateReport, duplicateRows.join("\n") + "\n");
    await assert.rejects(inspect(oldManifest, nextManifest, oldStudy, duplicateReport, "0"),
      /Duplicate previous lexical ID in reconciliation report/);
    // Candidate matches cannot merge two different previous records.
    const sharedTarget = id(count + 1);
    const conflictRows = unchangedRows.trimEnd().split("\n");
    for (const index of [0, 1]) {
      conflictRows[index] = JSON.stringify({
        format_version: 1, status: "candidate_review", requires_review: true,
        old_entry_id: id(index), new_entry_id: sharedTarget
      });
    }
    const conflicting = join(folder, "conflicting-candidates.jsonl");
    await writeFile(conflicting, conflictRows.join("\n") + "\n");
    await assert.rejects(inspect(oldManifest, nextManifest, oldStudy, conflicting, "2"),
      /Two old entries cannot claim the same candidate/);
    // A new candidate must not reuse the ID of any old exact/conflicting entry.
    const reservedRows = unchangedRows.trimEnd().split("\n");
    reservedRows[0] = JSON.stringify({
      format_version: 1, status: "candidate_review", requires_review: true,
      old_entry_id: id(0), new_entry_id: id(1)
    });
    const reusedId = join(folder, "reused-target.jsonl");
    await writeFile(reusedId, reservedRows.join("\n") + "\n");
    await assert.rejects(inspect(oldManifest, nextManifest, oldStudy, reusedId, "1"),
      /Candidate may not reuse an ID/);
    // The original snapshot itself must contain unique lexical IDs.
    const duplicatedStudy = join(folder, "duplicate-old.jsonl");
    const previousRows = oldRows.trimEnd().split("\n");
    previousRows[1] = previousRows[0];
    await writeFile(duplicatedStudy, previousRows.join("\n") + "\n");
    await assert.rejects(inspect(oldManifest, nextManifest, duplicatedStudy, unchanged, "0"),
      /Duplicate previous lexical ID/);


    await writeFile(nextManifest, JSON.stringify({...manifest, entry_count: 450000}));
    await assert.rejects(inspect(oldManifest, nextManifest, oldStudy, unchanged, "100001"),
      /more than 15%/);
  } finally {
    await rm(folder, {recursive:true, force:true});
  }
});
