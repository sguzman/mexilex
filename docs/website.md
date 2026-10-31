# Public dictionary website

The website is a **read-only presentation layer** for Mexilex. It is not a flashcard trainer, has no login, and stores no learning data.

## How search works

- A Rust generator scans `corpus/`, already committed to Git.
- It writes lightweight JSON search buckets, keyed by the first two letters of a normalized word. No JavaScript toolchain or npm dependencies.
- Browser search downloads **only the relevant prefix bucket**. A separate lightweight `mexico.json` browse index allows discovery of explicitly Mexico-tagged headwords without first knowing a word. The normal `index/` directory includes headwords only; `forms/` contains inflected forms and is fetched only if the user activates **Include conjugated and inflected forms**.
- Selecting a result retrieves **one Git-tracked lexical shard** from `raw.githubusercontent.com`, pinned to the build's exact commit SHA.
- Definitions are English and the recorded headwords are Spanish. A Mexico badge means **explicitly tagged Mexican usage**, not the sole set of words used in Mexico.
- Interface theme defaults to System, with manual Light and Dark choices.

The site artifact **does not duplicate the entire corpus**, which helps keep GitHub Pages under its published-site size limits. The source corpus stays in Git.

## Build and preview

```sh
cargo run --release -- build-site corpus data/site "$(git rev-parse HEAD)"
```

Serve `data/site` through any local static HTTP server. File-based `file://` preview is not supported because the browser fetches JSON indexes; opening `index.html` as a local file may fail. The website requires network access to GitHub when an entry is selected, although search buckets are hosted with the site.

The [website workflow](../.github/workflows/site.yml) builds and tests the index, checks JavaScript syntax, and uploads a preview artifact from GitHub Actions.

## Public GitHub Pages deployment

**Published URL (reported by GitHub Pages): https://sguzman.github.io/mexilex/**

The [website workflow](../.github/workflows/site.yml) now automatically deploys the validated static site after each site-relevant `main` change. The deployment job succeeded on 2026-10-31, and GitHub reported the URL above. No extra repository deployment variable is required.

An independent HTTP check from this execution environment was not possible because its external DNS/network access failed. The deployment itself is verified by GitHub Actions, but end-user Firefox and actual cross-origin source fetches still need confirmation; do not conflate those separate checks.

Deployment verification run: https://github.com/sguzman/mexilex/actions/runs/41241157586

## Licensing

Dictionary content originates from English Wiktionary via Kaikki/Wiktextract. The site links back to Wiktionary, describes the source and licensing in the footer, and links each lexical record to its source shard. See [CORPUS-LICENSE.md](../CORPUS-LICENSE.md). The website code is MIT licensed; dictionary content retains its upstream terms.

## Browser QA record (2026-10-31)

The generated site artifact was inspected with Chromium at desktop (1365 px) and mobile (390 px) widths. A local test verified populated statistics, search for `banqueta`, the Mexico filter, a 60-word Mexican vocabulary discovery sample, System-to-Dark switching, and no mobile horizontal overflow. No browser page errors occurred in the test. Because the environment could not access remote sites, JSON index requests were fulfilled from the **actual generated index files**, while the external GitHub lexical shard response was mocked; this does **not** prove live Pages or raw.githubusercontent.com delivery. Real deployed cross-origin fetches and phone QA remain outstanding.
