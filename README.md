# Mexilex

**Mexican Spanish lexical data, explained in English.** Mexilex is a public, reproducible dictionary and machine-readable lexical catalogue, not a flashcard scheduler.

The target variety is `es-MX` and the definition language is `en`. Importing general Spanish entries is intentional: words used in Mexico do not need a special regional label. Individual word *senses* are marked as Mexican only when an upstream source explicitly says so. Absence of a Mexican label is **not** evidence that a sense is unused in Mexico.

## Responsibilities

- Import existing dictionary data automatically, without manually adding words or imposing an arbitrary vocabulary cap.
- Preserve separate meanings, parts of speech, forms, examples, provenance, and regional tags.
- Normalize into a Git-tracked, inspectable JSONL lexical corpus with deterministic shards; build SQLite for local consumers.
- Export read-only sense data suitable for a separate future spaced-repetition trainer.
- Later: a static, read-only public dictionary website (with System/light/dark selector).

Mexilex will **not** hold user accounts, flashcard review dates, retention scores, or other private learning progress. A separate Rust/egui application will own those.

## Source and first import

Start with the **English edition of Wiktionary's Spanish data**, structured by [Kaikki / Wiktextract](https://kaikki.org/dictionary/Spanish/). This is a Spanish vocabulary source, *not* an English dictionary. Its glosses are in English.

The currently available [Spanish-only JSONL export](https://kaikki.org/dictionary/Spanish/kaikki.org-dictionary-Spanish.jsonl) is convenient but explicitly marked **deprecated** by Kaikki. The importer also accepts the [full English-edition raw JSONL(.gz)](https://kaikki.org/dictionary/rawdata.html), selecting only records whose `lang_code` is `es`, so we are not locked to the older feed.

Example (download only when you are ready to handle roughly 1 GB of input):

```sh
mkdir -p data
curl -fL --retry 3 \
  -o data/kaikki-spanish.jsonl \
  https://kaikki.org/dictionary/Spanish/kaikki.org-dictionary-Spanish.jsonl
cargo run --release -- import data/kaikki-spanish.jsonl data/mexilex.sqlite
cargo run --release -- export-corpus data/mexilex.sqlite corpus
cargo run --release -- stats data/mexilex.sqlite
cargo run --release -- lookup data/mexilex.sqlite banqueta
cargo run --release -- export-tsv data/mexilex.sqlite data/all-senses.tsv
cargo run --release -- export-tsv data/mexilex.sqlite data/mexican-labeled.tsv --mexican-only
```

For the larger full raw extraction, download its compressed `.jsonl.gz` file and pass that path to `import`; input is streamed and never loaded as a whole into memory. Import runs in a single SQLite transaction and rolls back malformed data.

**Important:** The CLI assumes an English-Wiktionary feed. It does not translate Spanish Wiktionary definitions. A future optional Spanish-Wiktionary enrichment adapter must not overwrite English glosses.

## Data licensing

The repository's MIT license applies to **our code**. Wiktionary/Kaikki-derived lexical material has its **own source licenses**, including CC BY-SA and GFDL; do not relicense extracted dictionary content as MIT. Any released database or public website that republishes data must retain appropriate attribution, comply with share-alike terms, and link to the original Wiktionary content and license information.

Upstream: [Wiktionary](https://en.wiktionary.org/) · [Kaikki](https://kaikki.org/) · [Wiktextract](https://github.com/tatuylonen/wiktextract). For research use see Tatu Ylonen, *Wiktextract: Wiktionary as Machine-Readable Structured Data*, LREC 2022.

The **normalized corpus in `corpus/` is versioned in Git**; raw upstream dumps in `data/` and built SQLite files are ignored. Each export writes deterministic, hashed JSONL shards and `corpus/manifest.json`. Refreshes are made in one batch commit, never one commit per word. GitHub releases can later distribute a prebuilt SQLite database.

The [one-shot corpus seeding workflow](.github/workflows/seed-corpus.yml) fetches the English-Wiktionary **Spanish** JSONL feed on GitHub runners, imports it, exports the normalized shards, and makes one data commit to `main`. It also supports manual subsequent refreshes. This keeps the initial huge download off your computer, but it remains a costly input download. If the output exceeds the defined Git size budget, the workflow stops instead of silently truncating the dictionary.

## Architecture

See [docs/architecture.md](docs/architecture.md) for the ontology, source policy, stable identifiers and boundaries between Mexilex, the public site, and the future trainer.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

The checked-in three-line fixture tests both the importer and deterministic Git corpus export without downloading the upstream dataset.
