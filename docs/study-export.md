# Study export contract (v1)

The **Mexilex study export** is a machine-readable input for a separate Rust/egui spaced-repetition application. It is a catalog of lexical entries, not a private user database and not a prebuilt Anki deck.

```sh
cargo run --release -- export-study corpus data/study.jsonl
cargo run --release -- export-study corpus data/mexican-study.jsonl --mexico-only
cargo run --release -- export-study corpus data/all-forms.jsonl --include-forms
```

Every invocation writes a **new** output file; it never overwrites another export or changes anything in the Git-tracked `corpus/`. The default export removes records whose every meaning has the `form-of` tag (e.g. many conjugated verbs). That does not remove conjugations from the source corpus: those remain available to the trainer on demand. `--mexico-only` keeps entries that have at least one explicitly tagged Mexico-specific meaning, while preserving all their distinct meanings.

Each JSONL line contains one lexical entry:

```json
{
  "format_version": 1,
  "source": "mexilex",
  "source_edition": "enwiktionary",
  "lexical_language": "es-MX",
  "definition_language": "en",
  "entry_id": "<content-derived lexical key>",
  "word": "banqueta",
  "part_of_speech": "noun",
  "etymology_number": null,
  "forms": [],
  "senses": [
    {"id": "<upstream sense identifier>", "definition_en": "sidewalk", "mexican_usage": true}
  ],
  "mexico_labeled": true,
  "inflection_only": false
}
```

The example above illustrates the core fields; real records retain the full sense objects, including examples and raw tags.

## Responsibilities and limits

- All word content comes from the Git corpus. The downstream trainer owns review events, due dates, spaced-repetition state, deck memberships and personal performance metrics.
- Do **not** treat two senses with the same headword as automatically two unambiguous front/back flashcards. The trainer may generate more precise card prompts with contextual example sentences.
- `entry_id` and sense IDs identify a source record, but a future Wiktionary refresh may change text-derived hashes or upstream IDs. Persist a source snapshot ID alongside progress and reconcile renamed/changed entries instead of wiping learning history.
- An entry lacking Mexico-specific tags may still be ordinary Mexican Spanish; filters never assert that unlabeled words are Spain-only.
- Public redistribution must preserve Wiktionary's separate lexical-content licensing and attribution (see [CORPUS-LICENSE.md](../CORPUS-LICENSE.md)); Mexilex code remains MIT licensed.

No manual vocabulary ceiling is enforced. Statistics from the site build indicate the majority of imported records are inflected forms, so the default study export is intentionally much smaller than the full dictionary.

## Downloadable build artifact

The [Validate committed lexical corpus](../.github/workflows/validate-corpus.yml) workflow now packages two files in the `mexilex-study-data` artifact:

- `study.jsonl`: all headword-style entries, excluding pure inflections by default.
- `mexico-labeled.jsonl`: smaller subset with at least one explicitly Mexico-tagged sense.

Each artifact also contains `manifest.json` with the source commit and language contract, `SHA256SUMS`, and `CORPUS-LICENSE.md`. This permits the future trainer to import a verified snapshot without cloning the entire source corpus or downloading Wiktionary.

**Artifact retention is 14 days.** It is not a permanent release channel, and not an upload of personal review history. For durable public distribution, a later release workflow should publish these files with stable version names and checksums.

## Comparing dictionary snapshots before upgrading a trainer

Lexical IDs are derived from source content and may change across upstream refreshes. A trainer must **not** destroy or silently reassign learning history when that happens.

Generate two exports from distinct versions of the corpus, then compare:

```sh
mexilex compare-study old-study.jsonl new-study.jsonl reconciliation.jsonl
```

The comparison emits a **non-destructive, one-row-per-previous-entry JSONL report**:

- `exact`: the lexical ID still exists in the new snapshot. Individual senses are checked for equal English definitions.
- `candidate_review`: the old ID disappeared but precisely one new entry has the same source edition, Spanish headword, part of speech, and etymology number. **Never auto-migrate this candidate without review.**
- `ambiguous`: multiple lexical entries share those fallback attributes. Candidate IDs are included; none is selected.
- `id_conflict`: an existing entry ID unexpectedly points to a different headword, part of speech, etymology, or source. Never treat that as a safe match.
- `missing`: no match is found. Historical review events should remain in the trainer, marked as referring to an older snapshot.

**Important:** An `exact` entry-ID match is not automatically a safe content match. Each report row now has `requires_review`. Only `false` means that the old meanings, annotations, forms, and sense set are unchanged. New meanings, modified examples/tags, changed forms, gloss edits, missing senses, and ambiguous matches all set it to `true`. The report identifies `changed_annotation`, `changed_definition`, and `missing` sense statuses, plus `added_sense_ids` and `forms_changed`. The comparison still does not alter trainer data.

For matched entries, individual old senses are tagged `same_definition`, `changed_definition`, `changed_annotation`, or `missing` based on source sense ID, definition text, and complete sense metadata. There is no fuzzy semantic matching and no automatic deletion of review history. Both inputs must be compatible version-one Mexilex study exports in the `es-MX` → `en` contract. The command refuses to overwrite an existing reconciliation report.

The build pipeline also compares a full study export to itself, verifying that the complete current snapshot yields exact matches only. Real-world cross-version testing remains a separate acceptance criterion before trainer synchronization is enabled.

## Mexican film-dialogue frequency for study prioritization

CI and versioned study packages include a **separate** `mexico-dialogue-frequency.jsonl` sidecar with 12,528 Mexico-attested non-form-only headword spellings. The same package includes `mexico-dialogue-frequency-manifest.json` and GPL-2.0 license text. The trainer may use these rates to *prioritize study candidates*, but must not reinterpret a film-dialogue frequency as a `mexican_usage` sense annotation or overwrite personal progress. The stable version-one `study.jsonl` schema is unchanged.

## Full-corpus upgrade simulation

The corpus CI workflow also creates a **temporary** revised study snapshot from the real 124,444-entry export using a dependency-free Node script. It exercises a changed English gloss, altered sense annotations, new word forms, a newly added sense, a replaced entry ID, and a missing entry. A new reconciliation JSONL report is checked for flagged and unchanged records. This is a regression simulation, **not evidence of validation against two distinct upstream Wiktionary editions**; the latter remains a release criterion before automatic trainer upgrades.
