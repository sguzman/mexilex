# Mexilex: bounded first-release roadmap

**Target:** a reproducible, publicly inspectable `es-MX`-oriented Spanish lexicon with **English** definitions, a usable read-only dictionary website and a portable feed for a separate spaced-repetition trainer. The project is **not** the trainer, not a universal language ontology, and not restricted to a fixed count of words.

This is a weighted **engineering-readiness** estimate, not a percentage of all possible Spanish vocabulary. The study-app project is separate.

| Workstream | Weight | Completion | Evidence / acceptance |
| --- | ---: | ---: | --- |
| ✅ Import and normalize a real lexicon | 30% | 100% | English Wiktionary Spanish JSONL → Rust → SQLite; full import of 813,928 entries and 879,019 senses; per-meaning regional labels |
| ✅ Version the processed vocabulary in Git | 15% | 100% | 4,096 normalized JSONL shards committed on `main`; full snapshot reconstructs SQLite without redownloading upstream |
| 🟡 Read-only dictionary presentation | 15% | 85% | System/light/dark site, source attribution, live search index generation, word details, Mexico browse, headword/inflection separation; **manual browser QA remains** |
| 🟡 Portable trainer exports | 15% | 95% | 124,444 lemma-oriented study entries, Mexico-tagged subset, tests and versioned export format; **verify artifact publication and stable ID reconciliation before syncing real review history** |
| 🟡 Mexican usage evidence enrichment | 15% | 15% | 2,115 source-verified Mexico-labeled senses and [licensing-aware source audit](docs/mexican-source-audit.md); **no unverified commercial/bulk dictionary import; source selection and frequency evidence are still open** |
| ⬜ Public website deployment | 10% | 0% | GitHub Pages GitHub-Actions deployment workflow exists; **not considered complete until Pages is configured, deployed, and independently browser-tested** |

**Weighted progress: approximately 75%.** Figures are estimates with clear acceptance criteria, not assertions that all words in Mexico are already represented.

## Next bounded work

1. **Website QA:** enable Pages only when authorized/configured, then verify the public UI in a real browser on desktop and mobile (including accents, `ñ`, Mexico filters and raw-entry fetches).
2. **Regional evidence:** identify a redistributable Mexico-specific frequency/usage dataset; keep frequency data and dictionary regional meanings separate and preserve provenance.
3. **Trainer contract stability:** test upgrade reconciliation across two different upstream snapshots and guarantee that dictionary refreshes never erase a future trainer's review history.
4. **Permanent distribution:** promote verified study artifacts to durable versioned releases, including checksums and license notices, only after the contract is stable.

Do not add additional languages, user accounts, SRS algorithms, or unrelated ontology to Mexilex.
