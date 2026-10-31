# Versioned lexical-data releases

Mexilex stores its normalized Spanish vocabulary in Git under `corpus/`. The SQLite database and study export archives are **derived snapshots**, not the canonical source.

## Build and publication

Open **Actions → Package versioned lexical release → Run workflow**. Supply a version such as `0.1.0`.

- **Publish: false** (default): Build the SQLite and study archives, upload a 14-day preview artifact, and create no tag or release.
- **Publish: true**: Publish a versioned GitHub Release tagged `lex-v0.1.0`. An existing release is not silently overwritten.
- Both archives carry the exact Git commit, source metadata, upstream source checksum, and corpus attribution/license notices. A separate `SHA256SUMS` authenticates the archives.

The SQLite archive includes a generated local dictionary database. The study archive contains `study.jsonl` (headword-oriented vocabulary) and `mexico-labeled.jsonl` (explicitly Mexico-tagged subset). Neither contains personal review history.

## Compatibility boundary

Lexical-data content retains upstream licenses (not the repository MIT code license). Trainer consumers should pin a release and compare snapshots with `mexilex compare-study` before changing source data, preserving old review events for missing or ambiguous records.

The workflow also performs a preview-only smoke test when its own YAML changes on `main`. This test neither creates a tag nor publishes any release. A **permanent release still requires an explicit manual run with Publish: true**.
