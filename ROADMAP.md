# Mexilex: bounded first-release roadmap

**Target:** a reproducible, publicly inspectable `es-MX`-oriented Spanish lexicon with **English** definitions, a usable read-only dictionary website and a portable feed for a separate spaced-repetition trainer. The project is **not** the trainer, not a universal language ontology, and not restricted to a fixed count of words.

This is a weighted **engineering-readiness** estimate, not a percentage of all possible Spanish vocabulary. The study-app project is separate.

| Workstream | Weight | Completion | Evidence / acceptance |
| --- | ---: | ---: | --- |
| ✅ Import and normalize a real lexicon | 30% | 100% | English Wiktionary Spanish JSONL → Rust → SQLite; full import of 813,928 entries and 879,019 senses; per-meaning regional labels |
| ✅ Version the processed vocabulary in Git | 15% | 100% | 4,096 normalized JSONL shards committed on `main`; full snapshot reconstructs SQLite without redownloading upstream |
| 🟡 Read-only dictionary presentation | 15% | 92% | System/light/dark site, source attribution, indexed lookup, Mexico discovery, split forms; desktop/mobile Chromium QA of generated assets passed with mocked network, **real deployed fetch and mobile-device QA remain** |
| 🟡 Portable trainer exports | 15% | 98% | 124,444 headword-style entries; downloadable checksummed packages; full same-version reconciliation passed; **validate real cross-version migrations and publish a permanent release before enabling trainer synchronization** |
| 🟡 Mexican usage evidence enrichment | 15% | 15% | 2,115 source-verified Mexico-labeled senses and [licensing-aware source audit](docs/mexican-source-audit.md); **no unverified commercial/bulk dictionary import; source selection and frequency evidence are still open** |
| 🟡 Public website deployment | 10% | 85% | GitHub Pages build and deploy jobs succeeded; GitHub reported https://sguzman.github.io/mexilex/; **user Firefox and live external fetch verification still outstanding** |

**Weighted progress: approximately 84% (+8 percentage points from the previous 76% report).** Figures are estimates with clear acceptance criteria, not assertions that all words in Mexico are already represented.

## Next bounded work

1. **Website QA:** the public Pages deployment succeeded; verify the actual URL in Firefox on desktop and mobile, including accents, `ñ`, Mexico filters and live raw-entry fetching.
2. **Regional evidence:** identify a redistributable Mexico-specific frequency/usage dataset; keep frequency data and dictionary regional meanings separate and preserve provenance.
3. **Trainer contract stability:** `compare-study` now rejects ambiguous, missing, or changed identities and passed a 124,444-entry self-comparison; validate a genuinely different upstream snapshot and consumer-specific migration behavior before synchronizing private reviews.
4. **Permanent distribution:** the opt-in, versioned packaging workflow passes a full preview build with checksums and license notices. Publish a durable release explicitly when the contract is ready, and verify the published downloads.

Do not add additional languages, user accounts, SRS algorithms, or unrelated ontology to Mexilex.
