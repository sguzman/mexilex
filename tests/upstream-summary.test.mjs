import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

const script = new URL("../scripts/summarize-upstream-review.mjs", import.meta.url).pathname;
function run(args) {
  return spawnSync(process.execPath, [script, ...args], {encoding:"utf8", timeout:10000});
}

test("actual-upstream summary counts old entries without silently claiming a migration", () => {
  const root = mkdtempSync(join(tmpdir(), "mexilex-upstream-summary-"));
  try {
    const oldManifest = join(root, "old-manifest.json");
    const newManifest = join(root, "new-manifest.json");
    const old = join(root, "old.jsonl");
    const next = join(root, "new.jsonl");
    const report = join(root, "compare.jsonl");
    const checksum = join(root, "source.sha256");
    const manifest = {format_version:1, lexical_language:"es-MX", definition_language:"en",entry_count:813928,sense_count:879019};
    writeFileSync(oldManifest, JSON.stringify(manifest));
    writeFileSync(newManifest, JSON.stringify({...manifest,entry_count:813929,sense_count:879020}));
    writeFileSync(old, '{"word":"banqueta"}\n{"word":"casa"}\n');
    writeFileSync(next, '{"word":"banqueta"}\n{"word":"casa"}\n{"word":"chamba"}\n');
    writeFileSync(report, [
      JSON.stringify({status:"exact",requires_review:true,senses:[{status:"same_definition"},{status:"missing"}]}),
      JSON.stringify({status:"candidate_review",requires_review:true,forms_changed:true,added_sense_ids:["s2"],senses:[{status:"changed_annotation"}]})
    ].join("\n")+"\n");
    writeFileSync(checksum, "a".repeat(64)+"\n");
    const size = run(["--size-check",oldManifest,newManifest]);
    assert.equal(size.status,0,size.stderr);
    const summary = run([old,next,report,newManifest,checksum]);
    assert.equal(summary.status,0,summary.stderr);
    const parsed = JSON.parse(summary.stdout);
    assert.equal(parsed.old_study_entries,2);
    assert.equal(parsed.new_study_entries,3);
    assert.equal(parsed.stats.exact,1);
    assert.equal(parsed.stats.candidate_review,1);
    assert.equal(parsed.stats.changed_annotation,1);
    assert.equal(parsed.stats.missing_senses,1);
    assert.equal(parsed.stats.missing,0); // Missing meaning is not a missing lexical entry
    assert.equal(parsed.stats.exact + parsed.stats.candidate_review
      + parsed.stats.ambiguous + parsed.stats.id_conflict + parsed.stats.missing, 2);
    assert.equal(parsed.stats.added_senses,1);
    assert.equal(parsed.stats.changed_forms,1);
    assert.equal(parsed.automatic_corpus_update,false);
    assert.equal(parsed.automatic_trainer_migration,false);
    // A genuine missing entry is counted separately from a missing sense.
    writeFileSync(report, [
      JSON.stringify({status:"exact",requires_review:false,senses:[{status:"same_definition"}]}),
      JSON.stringify({status:"missing",requires_review:true,senses:[]}),
    ].join("\n")+"\n");
    const missingEntry = run([old,next,report,newManifest,checksum]);
    assert.equal(missingEntry.status,0,missingEntry.stderr);
    assert.equal(JSON.parse(missingEntry.stdout).stats.missing,1);
    assert.equal(JSON.parse(missingEntry.stdout).stats.missing_senses,0);
    // Unknown sense-level categories must fail closed, never disappear from the summary.
    writeFileSync(report, [
      JSON.stringify({status:"exact",requires_review:true,senses:[{status:"unrecognized"}]}),
      JSON.stringify({status:"missing",requires_review:true,senses:[]}),
    ].join("\n")+"\n");
    const unexpected = run([old,next,report,newManifest,checksum]);
    assert.notEqual(unexpected.status,0);
    assert.match(unexpected.stderr,/Unknown sense-level reconciliation status/);
    // A migration report must include one row for every old record.
    writeFileSync(report,'{"status":"exact","requires_review":false}\n');
    const missing = run([old,next,report,newManifest,checksum]);
    assert.notEqual(missing.status,0);
    assert.match(missing.stderr,/Every old study entry must appear/);
    // A broken new input should fail before producing a migration summary.
    writeFileSync(newManifest,JSON.stringify({...manifest,entry_count:400000}));
    assert.notEqual(run(["--size-check",oldManifest,newManifest]).status,0);
  } finally {
    rmSync(root,{recursive:true,force:true});
  }
});
