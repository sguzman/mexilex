# Upstream snapshot review (read-only)

[Review real upstream Wiktionary snapshot safely](../.github/workflows/review-upstream.yml) inspects the current [Kaikki English-Wiktionary Spanish JSONL](https://kaikki.org/dictionary/Spanish/) without touching `main` vocabulary, deployment, releases, or personal training data.

The pipeline also runs **weekly on Mondays at 12:00 UTC** through GitHub Actions. A cron review is bounded and entirely read-only. If the current Kaikki extract matches the pinned upstream SHA-256, it skips expensive conversion and records the unchanged status. If the bytes have genuinely changed, it generates a disposable comparison and a visible Actions run summary; no commit, release, or trainer migration occurs automatically. Scheduled runs depend on GitHub Actions scheduling and are not a guarantee of instantaneous notification.

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

The [unit test](../tests/upstream-refresh.test.mjs) uses 100,001 unique synthetic lexical entries to exercise the same minimum-size checks. The guard now verifies that every old lexical entry ID appears exactly once in the report, rejects repeated old IDs and candidate target collisions, prevents candidate reuse of any old lexical ID, and enforces the approved migration review budget. It rejects large corpus changes. The companion [summary test](../tests/upstream-summary.test.mjs) separately verifies that a removed sense increments `missing_senses` without falsely increasing the number of missing *entries*. No refresh has been performed by this new gate; the existing dictionary, deployed site, and v0.1.0 release remain unchanged.

## Scheduling and genuine historical editions

The scheduled inspection uses the existing pinned current-source checksum as its baseline; it does **not** pretend that prior v0.1/v0.2/v0.3 **release packages** are independent upstream Wiktionary editions, because those releases reused one lexical corpus. A separately published [July 2026 English Wiktionary-derived flattened dataset](https://huggingface.co/datasets/jake-anto/wiktionary) uses a different parser and relational schema, so it cannot simply be fed into the same Rust JSONL importer or treated as the missing compatible full snapshot. A successful review of genuinely different compatible Kaikki inputs, preserving the conservative migration report, remains the trainer acceptance criterion. The monthly/weekly inspection never bypasses the manually approved refresh gate.

## Verified read-only weekly review and safety gates (2026-10-31)

The [first scheduled-review workflow run](https://github.com/sguzman/mexilex/actions/runs/41253870010) successfully downloaded the upstream source and confirmed it still matches pinned SHA-256 `357d407e62ae702fc8ef3c494223b17ba0774904fc28c2a9952fa9ae279c26f9`. The [subsequent workflow run](https://github.com/sguzman/mexilex/actions/runs/41253967943) also passed. No distinct edition or migration was falsely claimed. The [final refresh-safety tests](https://github.com/sguzman/mexilex/actions/runs/41254080794) passed, covering one-to-one candidate matching, complete old-ID report coverage, and separate missing-sense versus missing-entry statistics. A new upstream source checksum will be analyzed in a temporary workspace, not committed or released without the preexisting manual reviewed-refresh gate.
