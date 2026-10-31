import test from "node:test";
import assert from "node:assert/strict";
import { normalizeTranscriptMetadata } from "../scripts/import-ciempiess.mjs";

const fixture = [
  "audio_id\tspeaker_id\tgender\tduration\tnormalized_text",
  "first\tF_01\tfemale\t1.2\tNiño niño ¿qué pasa?",
  "second\tM_02\tmale\t1.8\tEl niño dice que sí y sí",
  "third\tM_03\tmale\t2.1\tNiño con qué",
].join("\n") + "\n";

test("counts Mexican transcribed surface forms, utterance coverage and accents", () => {
  const { rows, stats } = normalizeTranscriptMetadata(fixture);
  assert.equal(stats.utterances, 3);
  assert.equal(stats.token_total, 14);
  assert.deepEqual(rows.find(row => row[0] === "niño"), ["niño", 4, 3]);
  assert.deepEqual(rows.find(row => row[0] === "qué"), ["qué", 2, 2]);
  assert.deepEqual(rows.find(row => row[0] === "sí"), ["sí", 2, 1]);
  assert.equal(rows.some(row => row[0] === "pasa"), false);
  assert.equal(JSON.stringify(rows).includes("F_01"), false);
  assert.equal(JSON.stringify(rows).includes("M_02"), false);
  assert.equal(JSON.stringify(rows).includes("¿qué pasa?"), false);
});

test("rejects duplicated IDs, missing fields and malformed TSV", () => {
  assert.throws(() => normalizeTranscriptMetadata(fixture.replace("second\tM_02", "first\tM_02")), /duplicated ID/);
  assert.throws(() => normalizeTranscriptMetadata("audio_id\tspeaker_id\nid\tsomebody\n"), /Missing/);
  assert.throws(() => normalizeTranscriptMetadata(fixture + "last\tmissing\n"), /Malformed/);
  assert.throws(() => normalizeTranscriptMetadata(fixture, 0), /minimum occurrence/);
});

test("does not merge accented and unaccented words", () => {
  const { rows } = normalizeTranscriptMetadata("audio_id\tnormalized_text\n1\tsi sí\n2\tsi sí\n");
  assert.deepEqual(rows, [["si", 2, 2], ["sí", 2, 2]]);
});
