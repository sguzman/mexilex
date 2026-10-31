# Architecture and boundaries

## What is being modelled?

Mexilex models **Spanish lexical records as used in Mexico**, with **English-language definitions**. It does not model an English lexicon, universal linguistics, or a personal study history.

1. **Entry**: a Spanish written form, part of speech, morphological forms, source edition and source location.
2. **Sense**: an individual English gloss, labels, examples and a regional-usage flag. Labels belong to senses, not automatically to every meaning of a word.
3. **Provenance**: upstream edition and source line. The original record remains recoverable from source dumps, not reproduced as thousands of Git commits.
4. **Exports**: stateless read models. The trainer should reference a sense identifier and store its review history in its own private database.

The initial SQL schema is in `src/schema.sql`.

## Mexican Spanish versus generic Spanish

Default corpus: all Spanish-language lexical entries from English Wiktionary. Mexico-specific labels are deliberately **evidence-based and incomplete**. A Mexican flag means the upstream explicitly identifies a sense as Mexican; no flag means the region is not confirmed. It does **not** mean the word is European-Spanish-only.

The first classifier recognizes exact `Mexico`, `Mexican`, `Mexican Spanish`, and `Mexico City`-type tags, avoiding false positives like `New Mexico`. Later source audits may add more approved aliases, richer regional taxonomies, or corpus evidence. Do not invent region labels from a translation.

## Import flow

English Wiktionary -> Kaikki/Wiktextract JSONL -> stream only `lang_code=es` -> structured SQLite -> deterministic sense export.

Each import is a replace-all transaction. To support reproducible snapshots, later enhancements should record source dump dates/checksums and publish an attribution manifest. A huge dataset may contain hundreds of thousands of conjugated forms; that volume is not a scope failure.

IDs currently use upstream sense `id` where present, and otherwise hashed lexical attributes. They are reproducible for unchanged source text but **not guaranteed permanent across upstream edits**. Before the trainer synchronizes reviews across dictionary updates, add an explicit ID reconciliation protocol; do not couple spaced-repetition progress to SQLite row numbers.

## Public website

A later website is a **read-only presentation client** for the catalogue: searchable entries, definitions, regional labels, source credit and coverage statistics. It should have a System-default theme toggle (System/light/dark). It must not implement reviews, accounts, or private analytics. For large datasets use paginated search or generated index shards instead of loading the entire catalogue into a browser tab.

## Separate Rust/egui trainer

The trainer consumes versioned exports from Mexilex and, later, other lexical Tariums. It owns deck composition, review events, spaced-repetition schedules, retention metrics and learning progress. Its private database stays outside Mexilex and its public repo.

The contract needs explicit `language=es-MX`, `definition_language=en`, `sense_key`, source version, and attribution. Imported text and review progress should remain separable and upgradable.

## Out of scope

No universal ontology spanning every language; no manually populated master word list; no forced word cap; no embedded training-progress database; no licensed commercial Mexican dictionary scraping; no automatic declaration that an untagged word is Spain-only.
