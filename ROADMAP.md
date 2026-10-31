# Mexilex: bounded first-release roadmap

**Target:** a reproducible, publicly inspectable `es-MX`-oriented Spanish lexicon with **English** definitions, a usable read-only dictionary website and a portable feed for a separate spaced-repetition trainer. The project is **not** the trainer, not a universal language ontology, and not restricted to a fixed count of words.

This is a weighted **engineering-readiness** estimate, not a percentage of all possible Spanish vocabulary. The study-app project is separate.

| Workstream | Weight | Completion | Evidence / acceptance |
| --- | ---: | ---: | --- |
| ✅ Import and normalize a real lexicon | 30% | 100% | English Wiktionary Spanish JSONL → Rust → SQLite; full import of 813,928 entries and 879,019 senses; per-meaning regional labels |
| ✅ Version the processed vocabulary in Git | 15% | 100% | 4,096 normalized JSONL shards committed on `main`; full snapshot reconstructs SQLite without redownloading upstream |
| 🟡 Read-only dictionary presentation | 15% | 99% | Real Chromium browser tested live search/detail fetch, dark/System themes, all three discovery buttons (region-tagged, film dialogue, radio interviews), no overflow at 390px or 320px; **physical phone QA remains** |
| 🟡 Portable trainer exports | 15% | 99% | 124,444 headword-style entries, independent GPL frequency evidence, checksummed public releases through lex-v0.3.0, 124,444-entry simulated revision comparison, and one-to-one candidate guards preventing homograph review-history collisions; **validate two real upstream versions before enabling trainer auto-migration** |
| 🟡 Mexican usage evidence enrichment | 15% | 70% | 2,115 English-Wiktionary Mexico-tagged senses, 12,528 film-dialogue-attested headwords, 1,235 independently Mexico-tagged Wikcionario spellings matched to English-defined words, and **6,250 surface words observed at least twice in 16,663 Mexican radio interview segments**. Separate source attribution, checksums and licenses; **broader representative written and conversational sources still remain open** |
| 🟡 Public website deployment | 10% | 99% | Public GitHub Pages verified in user's Firefox; HTTPS, index and source shard checks, plus two independent live Chromium desktop/mobile browser regression suites run only after successful deployment; **physical phone QA remains** |

**Weighted progress: approximately 95% (unchanged this round; verified radio browse, deployment-order fixes, and conservative cross-snapshot gate hardening strengthen existing workstreams without completing remaining release criteria).** Figures are estimates with clear acceptance criteria, not assertions that all words in Mexico are already represented.

## Next bounded work

1. **Website QA:** real Chromium checks pass for desktop, 390px mobile, and 320px narrow layouts; full HTTPS lookups, all three discovery modes, and light/dark controls are tested automatically after each deployment. A physical phone QA pass remains outstanding.
2. **Regional evidence:** ACTIV-ES film frequencies, Spanish Wiktionary Mexico-tagged spelling evidence and CIEMPIESS LIGHT radio-interview counts are all versioned in Git with distinct licenses and provenance. The radio evidence is topic- and speaker-biased, not representative of all Mexico; find a more representative, redistributable written/spoken source and maintain sense-level semantic separation.
3. **Trainer contract stability:** `compare-study` handles changed definitions/annotations/forms/senses and prevents multiple homographs from sharing one new ID. The refresh guard now verifies every old lexical ID appears exactly once, candidate targets are genuinely new and unique, and missing senses are not counted as missing entries. A **weekly read-only Kaikki source review** checks for genuine changes; the first post-scheduling review still found identical source bytes. A real cross-version migration remains untested; no automatic trainer review-history migration.
4. **Permanent distribution:** public [lex-v0.1.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.1.0), [lex-v0.2.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.2.0) and [lex-v0.3.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.3.0) are immutable releases. v0.3.0 includes separate CIEMPIESS radio word evidence and passed GitHub's public download, SHA-256 verification, corpus and study contract checks ([successful CI](https://github.com/sguzman/mexilex/actions/runs/41253033111)).

Do not add additional languages, user accounts, SRS algorithms, or unrelated ontology to Mexilex.
