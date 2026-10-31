# Versioned lexical-data releases

Mexilex stores its normalized Spanish vocabulary in Git under `corpus/`. The SQLite database and study export archives are **derived snapshots**, not the canonical source.

## Published first release

**[Mexilex lexical snapshot v0.1.0](https://github.com/sguzman/mexilex/releases/tag/lex-v0.1.0)** was successfully published with `mexilex-sqlite-v0.1.0.tar.gz`, `mexilex-study-v0.1.0.tar.gz`, and `SHA256SUMS`. The successful packaging/publishing workflow verified code, source reconstruction, data exports, archive contents, and checksums. This is a public GitHub Release, not a 14-day CI artifact. The associated tag pins the generated snapshot's Git commit.

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
