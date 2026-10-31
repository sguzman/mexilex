# Public dictionary website

The website is a **read-only presentation layer** for Mexilex. It is not a flashcard trainer, has no login, and stores no learning data.

## How search works

- A Rust generator scans `corpus/`, already committed to Git.
- It writes lightweight JSON search buckets, keyed by the first two letters of a normalized word. No JavaScript toolchain or npm dependencies.
- Browser search downloads **only the relevant prefix bucket**. A separate lightweight `mexico.json` browse index allows discovery of explicitly Mexico-tagged headwords without first knowing a word. By default it hides entries that are only inflected forms.
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

## Publish with GitHub Pages

1. On GitHub, open repository **Settings → Pages**. Set **Build and deployment → Source: GitHub Actions**.
2. In **Settings → Secrets and variables → Actions → Variables**, create `MEXILEX_DEPLOY_PAGES` with value `true`.
3. Rerun the **Build Mexican Spanish dictionary website** workflow in Actions. Its deployment job will publish the validated static artifact.

GitHub Pages is **not assumed to be enabled** merely because this workflow exists. The explicit opt-in prevents a misconfigured deployment from hiding build/test status. The site should then be hosted at the GitHub Pages URL shown by the successful deployment.

## Licensing

Dictionary content originates from English Wiktionary via Kaikki/Wiktextract. The site links back to Wiktionary, describes the source and licensing in the footer, and links each lexical record to its source shard. See [CORPUS-LICENSE.md](../CORPUS-LICENSE.md). The website code is MIT licensed; dictionary content retains its upstream terms.
