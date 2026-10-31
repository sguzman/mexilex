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
    const oldLine = JSON.stringify({entry_id:"x"}) + "\n";
    const unchangedRow = JSON.stringify({format_version:1,status:"exact",requires_review:false}) + "\n";
    const changedRow = JSON.stringify({format_version:1,status:"missing",requires_review:true}) + "\n";
    const count = 100001;
    await Promise.all([
      writeFile(oldStudy, oldLine.repeat(count)),
      writeFile(unchanged, unchangedRow.repeat(count)),
      writeFile(changed, unchangedRow.repeat(count-1) + changedRow)
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
    await writeFile(nextManifest, JSON.stringify({...manifest, entry_count: 450000}));
    await assert.rejects(inspect(oldManifest, nextManifest, oldStudy, unchanged, "100001"),
      /more than 15%/);
  } finally {
    await rm(folder, {recursive:true, force:true});
  }
});
