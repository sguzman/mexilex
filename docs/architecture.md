# Architecture and boundaries

## What is being modelled?

Mexilex models **Spanish lexical records as used in Mexico**, with **English-language definitions**. It does not model an English lexicon, universal linguistics, or a personal study history.

1. **Entry**: a Spanish written form, part of speech, morphological forms, source edition and source location.
2. **Sense**: an individual English gloss, labels, examples and a regional-usage flag. Labels belong to senses, not automatically to every meaning of a word.
3. **Provenance**: upstream edition and source line. The raw upstream dump is an ignored build input; the normalized record is exported into versioned JSONL shards in Git, updated with one batch commit.
4. **Exports**: stateless read models. The trainer should reference a sense identifier and store its review history in its own private database.

The initial SQL schema is in `src/schema.sql`.

## Mexican Spanish versus generic Spanish

Default corpus: all Spanish-language lexical entries from English Wiktionary. Mexico-specific labels are deliberately **evidence-based and incomplete**. A Mexican flag means the upstream explicitly identifies a sense as Mexican; no flag means the region is not confirmed. It does **not** mean the word is European-Spanish-only.

The first classifier recognizes exact `Mexico`, `Mexican`, `Mexican Spanish`, and `Mexico City`-type tags, avoiding false positives like `New Mexico`. Later source audits may add more approved aliases, richer regional taxonomies, or corpus evidence. Do not invent region labels from a translation.

## Import flow

English Wiktionary -> Kaikki/Wiktextract JSONL -> stream only `lang_code=es` -> structured SQLite -> deterministic, Git-tracked sharded JSONL corpus and stateless flashcard TSV export.

Each import is a replace-all transaction. Each Git export writes to a new or empty directory: `corpus/ab/c.jsonl` is one of up to 4096 small, stable hash buckets ordered by entry key. `corpus/manifest.json` records counts and attribution. The seeding workflow records the upstream checksum and commits one full snapshot, with a size-budget safety check. Subsequent full-corpus updates are **manual-only**: an independent read-only upstream review supplies the exact source SHA-256 and the approved study-reconciliation budget. The publishing workflow revalidates both before replacing the tracked corpus. An import may contain hundreds of thousands of inflected forms: that volume is not a scope failure.

IDs currently use upstream sense `id` where present, and otherwise hashed lexical attributes. They are reproducible for unchanged source text but **not guaranteed permanent across upstream edits**. Before the trainer synchronizes reviews across dictionary updates, add an explicit ID reconciliation protocol; do not couple spaced-repetition progress to SQLite row numbers.

## Public website

A later website is a **read-only presentation client** for the catalogue: searchable entries, definitions, regional labels, source credit and coverage statistics. It should have a System-default theme toggle (System/light/dark). It must not implement reviews, accounts, or private analytics. For large datasets use paginated search or generated index shards instead of loading the entire catalogue into a browser tab.

## Separate Rust/egui trainer

The trainer consumes versioned exports from Mexilex and, later, other lexical Tariums. It owns deck composition, review events, spaced-repetition schedules, retention metrics and learning progress. Its private database stays outside Mexilex and its public repo.

The contract needs explicit `language=es-MX`, `definition_language=en`, `sense_key`, source version, and attribution. Imported text and review progress should remain separable and upgradable.

## Out of scope

No universal ontology spanning every language; no manually populated master word list; no forced word cap; no embedded training-progress database; no licensed commercial Mexican dictionary scraping; no automatic declaration that an untagged word is Spain-only.

## Git history and dataset policy

The normalized lexical source **belongs in Git**. The upstream raw Wiktionary dump and the compiled SQLite cache **do not**. Never commit one word or one changed sense at a time; refresh the full corpus through a single batch commit. Stable hash sharding helps Git diff individual entries without forcing a monolithic, ever-changing gigabyte-sized blob into history. This limits history *growth rate*, not the number of lexical entries.

The source dictionary content retains upstream CC BY-SA / GFDL licensing obligations. `LICENSE` is for Mexilex code only. Distributed records, APIs and websites must carry the attribution and license text. Repo size must be monitored: if automatic import exceeds its budget, fail the workflow rather than quietly drop words or pretend coverage is complete. The repository should explicitly identify an incomplete or unseeded corpus until a real import succeeds.

## Rebuilding the index from Git

`build-from-corpus corpus data/mexilex.sqlite` regenerates a local SQLite index without fetching the raw third-party dump. It validates the corpus format version, languages, shard placement, uniqueness, and manifest counts. SQLite is a derived consumer artifact, while Git's normalized shards remain the authoritative public lexical dataset. The original upstream line number is not included in this normalized snapshot and is represented as zero when rebuilding.

## Trainer handoff

`export-study` is a **stateless read model** that filters pure form-of records by default. It does not modify the Git corpus and emits full source IDs, English meanings, examples, usage annotations and word forms. It intentionally does not generate duplicate ambiguous front/back cards for every meaning: the separate trainer may choose its own recognition, production and sentence-cloze templates. See [study-export.md](study-export.md).
