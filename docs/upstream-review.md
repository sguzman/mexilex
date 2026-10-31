# Upstream snapshot review (read-only)

[Review real upstream Wiktionary snapshot safely](../.github/workflows/review-upstream.yml) inspects the current [Kaikki English-Wiktionary Spanish JSONL](https://kaikki.org/dictionary/Spanish/) without touching `main` vocabulary, deployment, releases, or personal training data.

This pipeline intentionally follows these rules:

1. Download an upstream snapshot, verify size, and calculate SHA-256. Compare with `corpus/upstream.sha256`. **If identical, report "unchanged" and skip the costly rebuild. Do not claim that two distinct revisions were compared.**
2. If changed, run the existing Rust importer into a disposable SQLite file and export a new normalized corpus into `data/upstream-review/corpus` (never `corpus/`).
3. Validate schema and entry-count sanity (a drop or surge greater than 15% requires investigation), then export the old and new stateless study JSONL datasets.
4. Use `compare-study` on the real new extraction, producing conservative identity/definition/annotation/form/sense difference counts and a one-row-per-prior-entry reconciliation file.
5. Upload checksums, manifests, summary and report as a **14-day GitHub Actions artifact**. No repository writes, no Pages deployment, no automated review-history migration.

A "new" JSONL extraction may change because of Wiktextract processing, upstream Wiktionary edits, or export behavior; the change report does not imply that every difference represents a human editorial change. New input bytes are necessary but not sufficient to claim meaningful lexical revisions. The report's identity candidates must be reviewed before switching a trainer to a new snapshot.

The original source dump is not stored in Git. Normalized versions remain the canonical and auditable public corpus. Publication of a changed snapshot is a **separate explicit workflow**, not part of this review.

## First live upstream review: 2026-10-31

[Review run 41249568665](https://github.com/sguzman/mexilex/actions/runs/41249568665) fetched the genuine Kaikki English-Wiktionary Spanish source and compared its SHA-256 to the normalized Git snapshot's recorded upstream hash. Both were exactly:

`357d407e62ae702fc8ef3c494223b17ba0774904fc28c2a9952fa9ae279c26f9`

The source is **unchanged**, so the build correctly skipped the disposable rebuild and reconciliation. This does **not** count as testing two distinct editions. A real future revision remains the acceptance target.

## Mandatory reviewed refresh gate

The separate [Seed or refresh tracked lexical corpus](../.github/workflows/seed-corpus.yml) workflow is now **manual-only**. To publish a replacement, an operator must provide both the approved, exact 64-character upstream SHA-256 from a real review and a nonnegative maximum number of earlier study entries approved to require reconciliation.

The source checksum is independently checked again at publication time. The workflow rejects unchanged bytes, reconstructs the candidate outside `corpus/`, verifies Rust formatting, Clippy and tests, generates a real old-versus-new reconciliation report, rejects a corpus-size change exceeding 15%, and enforces a 900 MB normalized-Git budget. The [guard script](../scripts/guard-upstream-refresh.mjs) rejects candidate reports exceeding the approved review budget or omitting previous study entries. Only after all checks pass is the tracked `corpus/` replaced in one Git commit.

The [unit test](../tests/upstream-refresh.test.mjs) uses 100,001 synthetic entries to exercise the same minimum-size checks and rejects both budget violations and large corpus changes. No refresh has been performed by this new gate; the existing dictionary, deployed site, and v0.1.0 release remain unchanged.
