# Mexilex: bounded first-release roadmap

**Target:** a reproducible, publicly inspectable `es-MX`-oriented Spanish lexicon with **English** definitions, a usable read-only dictionary website and a portable feed for a separate spaced-repetition trainer. The project is **not** the trainer, not a universal language ontology, and not restricted to a fixed count of words.

This is a weighted **engineering-readiness** estimate, not a percentage of all possible Spanish vocabulary. The study-app project is separate.

| Workstream | Weight | Completion | Evidence / acceptance |
| --- | ---: | ---: | --- |
| ✅ Import and normalize a real lexicon | 30% | 100% | English Wiktionary Spanish JSONL → Rust → SQLite; full import of 813,928 entries and 879,019 senses; per-meaning regional labels |
| ✅ Version the processed vocabulary in Git | 15% | 100% | 4,096 normalized JSONL shards committed on `main`; full snapshot reconstructs SQLite without redownloading upstream |
| 🟡 Read-only dictionary presentation | 15% | 99% | Real Chromium browser tested live search/detail fetch, dark/System themes, both discovery buttons, no overflow at 390px, and corrected 320px narrow layout; **physical phone QA remains** |
| 🟡 Portable trainer exports | 15% | 99% | 124,444 headword-style entries, independent GPL frequency evidence, checksummed public releases through lex-v0.2.0, 124,444-entry simulated revision comparison, and one-to-one candidate guards preventing homograph review-history collisions; **validate two real upstream versions before enabling trainer auto-migration** |
| 🟡 Mexican usage evidence enrichment | 15% | 60% | 2,115 English-Wiktionary Mexico-tagged senses, 12,528 headwords attested in Mexican film dialogue and **1,235 independently Mexico-tagged Spanish-Wiktionary spellings** matched to English-defined words, all with separate provenance and licensing; **broader representative written/spoken Mexican sources remain open** |
| 🟡 Public website deployment | 10% | 99% | Public GitHub Pages verified in user's Firefox; HTTPS, index and source shard checks, plus live Chromium desktop/mobile browser regression tests now run automatically after deployment; **physical phone QA remains** |

**Weighted progress: approximately 94% (unchanged in this distribution-only round; the earlier 91% → 94% increase reflects independently sourced Mexican regional evidence).** Figures are estimates with clear acceptance criteria, not assertions that all words in Mexico are already represented.

## Next bounded work

1. **Website QA:** real Chromium checks pass for desktop, 390px mobile, and 320px narrow layouts; full HTTPS lookups, discovery buttons, and light/dark controls are tested automatically on every deployment. A physical phone QA pass remains outstanding.
2. **Regional evidence:** ACTIV-ES Mexican film-dialogue frequencies and Spanish Wiktionary's independently Mexico-tagged spellings are now both normalized and versioned in Git. Maintain separate evidence meanings and find a more representative Mexican written/spoken source with clear redistribution rights.
3. **Trainer contract stability:** `compare-study` handles changed definitions/annotations/forms/senses and now prevents multiple old homographs from being proposed for one new ID. A read-only real upstream review plus checksum-and-budget-gated refresh are implemented. The first live review found identical upstream bytes, so a genuine new-version migration remains untested; never auto-migrate private review histories.
4. **Permanent distribution:** public [lex-v0.1.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.1.0) and [lex-v0.2.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.2.0) now published, the latter including independently licensed Wikcionario Mexican-word records. Both have verified public checksums, full-source archives and stateless exports; v0.2.0 passed post-publication download and evidence-count checks.

Do not add additional languages, user accounts, SRS algorithms, or unrelated ontology to Mexilex.
