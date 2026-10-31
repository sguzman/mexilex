# Versioned lexical-data releases

Mexilex stores its normalized Spanish vocabulary in Git under `corpus/`. The SQLite database and study export archives are **derived snapshots**, not the canonical source.

## Published first release

**[Mexilex lexical snapshot v0.1.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.1.0)** was successfully published with `mexilex-sqlite-v0.1.0.tar.gz`, `mexilex-study-v0.1.0.tar.gz`, and `SHA256SUMS`. The successful packaging/publishing workflow verified code, source reconstruction, data exports, archive contents, and checksums. This is a public GitHub Release, not a 14-day CI artifact. The associated tag pins the generated snapshot's Git commit.

## Verified lex-v0.2.0 publication

**[Mexilex lexical snapshot v0.2.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.2.0)** was published on 2026-10-31 from commit `893df1657cb3c912680817656bb22204a11cb2f8`. The study archive contains independent Wikcionario Mexico-word evidence, its source manifest and CC BY-SA/GFDL attribution alongside GPL-2.0 film-frequency records. It never binds Spanish Wiktionary senses to specific English glosses; the export contains no learner review history.

The [successful publication workflow](https://github.com/sguzman/mexilex/actions/runs/41251273959) downloaded its actual public assets, checked SHA-256 hashes, extracted the study archive and verified 1,946 Wikcionario word/POS rows, 2,304 Mexico-tagged senses, 1,235 matched headword spellings and the `banqueta` evidence.

## Verified lex-v0.3.0 publication

**[Mexilex lexical snapshot v0.3.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.3.0)** was published on 2026-10-31 from commit `e6c9a3ec5d689fdf643d922a2e0fc9dccf457aea`. Its study archive adds `mexico-radio-frequency.jsonl`, `mexico-radio-frequency-manifest.json`, and `CIEMPIESS-SOURCE.md`, preserving the **CC BY-SA 4.0** source attribution and revision separately from GPL film-subtitle evidence and CC BY-SA/GFDL Wiktionary content.

The underlying CIEMPIESS LIGHT sidecar contains 6,250 orthographic surface-word records from 167,118 tokens across 16,663 transcribed radio interview segments, with no original transcript, speaker metadata or review history. These counts do not label any sense as Mexico-specific. The [publishing and independent postpublication download check](https://github.com/sguzman/mexilex/actions/runs/41253033111) succeeded: all three public release assets were downloaded, their SHA-256 checksums verified, and the study archive validated against the expected word/utterance count and *casa* observation.

The earlier v0.1.0 and v0.2.0 public assets remain unchanged.

## Build and publication

Open **Actions → Package versioned lexical release → Run workflow**. Supply a version such as `0.1.0`.

- **Publish: false** (default): Build the SQLite and study archives, upload a 14-day preview artifact, and create no tag or release.
- **Publish: true**: Publish a versioned GitHub Release tagged `lex-v0.1.0`. An existing release is not silently overwritten.
- Both archives carry the exact Git commit, source metadata, upstream source checksum, and corpus attribution/license notices. A separate `SHA256SUMS` authenticates the archives.

The SQLite archive includes a generated local dictionary database. The study archive contains `study.jsonl` (headword-oriented vocabulary) and `mexico-labeled.jsonl` (explicitly Mexico-tagged subset). Neither contains personal review history.

## Compatibility boundary

Lexical-data content retains upstream licenses (not the repository MIT code license). Trainer consumers should pin a release and compare snapshots with `mexilex compare-study` before changing source data, preserving old review events for missing or ambiguous records.

The workflow performs a preview-only smoke test when its YAML changes on `main`. **The first release was explicitly published on 2026-10-31** through a one-time, exact-commit-message-gated release intent; subsequent versions use the ordinary manual workflow with `Publish: true`.

## Independently licensed Mexican frequency sidecar

The study archive additionally contains `mexico-dialogue-frequency.jsonl`, `mexico-dialogue-frequency-manifest.json`, and `ACTIV-ES-LICENSE`. Those records are based on **ACTIV-ES v0.2** Mexican/Spanish/Argentine film-subtitle frequency counts and are independently licensed under GPL-2.0. They are **not** Mexico-specific definitions. Keep this data, provenance and license separate from the Wiktionary entry records when importing to an external trainer.

## Independently verified release consumption

A GitHub Actions [consumer check](../.github/workflows/verify-release.yml) downloaded all three public `lex-v0.1.0` assets **from GitHub Releases**, independently verified the SHA-256 file, extracted both archives, checked SQLite integrity and lexical/sense counts, and validated the 124,444-entry study export and 12,528-record independent Mexican dialogue-frequency sidecar. Its successful run is [recorded in Actions](https://github.com/sguzman/mexilex/actions/runs/41247735698). This is more than a build-time artifact integrity check: it demonstrates that consumers can download and inspect the actual public release.

## Independently licensed Wikcionario evidence

The published `lex-v0.2.0` study archive includes `wikcionario-mexico.jsonl`, `wikcionario-mexico-manifest.json` and `WIKCIONARIO-SOURCE.md`. This source uses CC BY-SA/GFDL terms, independent of English Wiktionary glosses and GPL-2.0 film-frequency data. **The original v0.1.0 assets remain immutable and lack this sidecar.**

The original [v0.2 preview](https://github.com/sguzman/mexilex/actions/runs/41250676401) was verified before v0.2.0 publication. It was followed by the independently verified permanent [v0.2.0 release](https://github.com/sguzman/mexilex/releases/tag/lex-v0.2.0); the new radio source is included starting with [v0.3.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.3.0). Each prior release's assets remain unchanged.
