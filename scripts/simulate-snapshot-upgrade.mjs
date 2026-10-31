// CI-only mutation of an actual Mexilex study export to exercise snapshot upgrades.
// No third-party packages, no modification of Git-tracked lexicon or user reviews.
import { createReadStream, createWriteStream } from "node:fs";
import { createInterface } from "node:readline";
import { once } from "node:events";
import { finished } from "node:stream/promises";

const [previous, destination] = process.argv.slice(2);
if (!previous || !destination || previous === destination) {
  throw new Error("usage: node scripts/simulate-snapshot-upgrade.mjs <previous.jsonl> <new.jsonl>");
}

const input = createInterface({ input: createReadStream(previous), crlfDelay: Infinity });
const output = createWriteStream(destination, { flags: "wx" });
let index = 0;
for await (const line of input) {
  if (!line.trim()) continue;
  const record = JSON.parse(line);
  if (record.format_version !== 1 || record.source !== "mexilex"
    || !Array.isArray(record.senses) || record.senses.length === 0) {
    throw new Error("Invalid source entry at position " + index);
  }
  switch (index) {
    case 0:
      record.senses[0].definition_en += " [simulated revised definition]";
      break;
    case 1:
      record.senses[0].tags = [...(record.senses[0].tags || []), "simulated-review-annotation"];
      break;
    case 2:
      record.forms = [{ form: "simulated-new-form" }];
      break;
    case 3:
      record.senses.push({ id: "ci-added-sense-unique", definition_en: "simulated second sense" });
      break;
    case 4:
      record.entry_id = "f".repeat(64);
      break;
    case 5:
      index++;
      continue; // Entire old entry missing from new export.
    default:
      break;
  }
  if (!output.write(JSON.stringify(record) + "\n")) await once(output, "drain");
  index++;
}
output.end();
await finished(output);
if (index < 100_000) throw new Error("Expected complete real study snapshot");
console.log("Created simulated changed study snapshot from", index, "source entries");
