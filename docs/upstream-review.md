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
