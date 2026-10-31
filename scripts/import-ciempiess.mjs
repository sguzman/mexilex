// Aggregate licensed CIEMPIESS LIGHT Mexican radio transcriptions without
// retaining any audio IDs, speaker identities, utterances, or original text.
// Built-in Node.js only; no Python, npm, or network access required here.
import { readFileSync, writeFileSync, existsSync, unlinkSync } from "node:fs";
import { createHash } from "node:crypto";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const SOURCE_REVISION = "e9e6dc556ba8479f54160701a7542a67de1fbe14";
export const SOURCE_SHA256 = "c3887d0e4b6adefa4ef5aa80dc76f858c89ad1a131e7f2b3f49b7316cbdad1f4";
export const SOURCE_URL =
  "https://huggingface.co/datasets/ciempiess/ciempiess_light/resolve/" +
  SOURCE_REVISION + "/corpus/files/metadata_train.tsv";

const hash = data => createHash("sha256").update(data).digest("hex");

export function normalizeTranscriptMetadata(text, minOccurrences = 2) {
  if (!Number.isSafeInteger(minOccurrences) || minOccurrences < 1) {
    throw Error("Invalid minimum occurrence count");
  }
  const lines = text.replaceAll("\r\n", "\n").trimEnd().split("\n");
  const header = lines.shift().split("\t");
  if (new Set(header).size !== header.length ||
      !["audio_id", "normalized_text"].every(field => header.includes(field))) {
    throw Error("Missing or duplicated CIEMPIESS metadata columns");
  }
  const idColumn = header.indexOf("audio_id");
  const textColumn = header.indexOf("normalized_text");
  const ids = new Set();
  const counts = new Map();
  let tokenTotal = 0;
  for (const [index, line] of lines.entries()) {
    const cells = line.split("\t");
    if (cells.length !== header.length) {
      throw Error("Malformed metadata row " + (index + 2));
    }
    const id = cells[idColumn];
    const transcript = cells[textColumn];
    if (!id || !transcript || ids.has(id)) {
      throw Error("Empty transcript/ID or duplicated ID at row " + (index + 2));
    }
    ids.add(id);
    const seen = new Set();
    for (const match of transcript.normalize("NFC").toLocaleLowerCase("es").matchAll(/\p{L}[\p{L}\p{M}]*/gu)) {
      const word = match[0];
      tokenTotal++;
      const record = counts.get(word) || [0, 0];
      record[0]++;
      if (!seen.has(word)) {
        record[1]++;
        seen.add(word);
      }
      counts.set(word, record);
    }
  }
  if (!tokenTotal) throw Error("No words in Mexican transcript input");
  // A one-off name or transcription artifact does not become public evidence.
  // This is a surface-word count, not a lemma or Mexico-specific sense label.
  const rows = [...counts]
    .filter(([, value]) => value[0] >= minOccurrences)
    .sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)
    .map(([word, [occurrences, utterances]]) => [word, occurrences, utterances]);
  return {
    rows,
    stats: {
      utterances: ids.size,
      token_total: tokenTotal,
      distinct_surface_tokens: counts.size,
      published_surface_tokens: rows.length,
      excluded_singleton_types: [...counts.values()].filter(x => x[0] < minOccurrences).length,
      minimum_occurrences: minOccurrences
    }
  };
}

export function importCiempiess(input, output, manifestPath) {
  if (existsSync(output) || existsSync(manifestPath)) {
    throw Error("Refusing to overwrite existing evidence files");
  }
  const source = readFileSync(input);
  const sha = hash(source);
  if (sha !== SOURCE_SHA256) throw Error("Unreviewed CIEMPIESS source checksum: " + sha);
  const { rows, stats } = normalizeTranscriptMetadata(source.toString("utf8"));
  if (stats.utterances !== 16663 || rows.length < 1000) {
    throw Error("Unexpected verified source coverage");
  }
  const data = rows.map(row => JSON.stringify(row)).join("\n") + "\n";
  const metadata = {
    format_version: 1,
    source: "CIEMPIESS LIGHT Mexican radio conversations",
    source_url: SOURCE_URL,
    source_revision: SOURCE_REVISION,
    source_sha256: sha,
    output_sha256: hash(data),
    license: "CC BY-SA 4.0; see CIEMPIESS-SOURCE.md",
    lexical_language: "es-MX",
    frequency_scope: "Central-Mexican Spanish radio interview segments, mainly law and university topics",
    measure: "orthographic surface-token occurrences and distinct utterance occurrences",
    matching: "not matched to meanings or lemmas; no Mexico-only inference",
    normalization: "Unicode NFC, Spanish lowercase, sequences of Unicode letters and combining marks",
    privacy: "aggregated tokens occurring at least twice; no audio, utterances, audio IDs, or speaker metadata",
    ...stats
  };
  writeFileSync(output, data, { flag: "wx" });
  try {
    writeFileSync(manifestPath, JSON.stringify(metadata, null, 2) + "\n", { flag: "wx" });
  } catch (error) {
    unlinkSync(output);
    throw error;
  }
  return metadata;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [source, output, manifest] = process.argv.slice(2);
  if (!source || !output || !manifest) {
    throw Error("usage: node scripts/import-ciempiess.mjs <metadata.tsv> <evidence.jsonl> <manifest.json>");
  }
  console.log(JSON.stringify(importCiempiess(source, output, manifest)));
}
