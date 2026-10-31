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

An external GitHub Actions runner successfully fetched the live HTML, the deployed search index and the pinned `raw.githubusercontent.com` lexical shard for **banqueta**, verifying **813,928 entries** and its Mexico-tagged **sidewalk** meaning over HTTPS. [Live HTTP verification](https://github.com/sguzman/mexilex/actions/runs/41241399334) succeeded on 2026-10-31. User Firefox and phone QA remain separate acceptance steps. The live HTTPS test will also rerun after future successful website deployment workflows.

Deployment verification run: https://github.com/sguzman/mexilex/actions/runs/41241157586

## Licensing

Dictionary content originates from English Wiktionary via Kaikki/Wiktextract. The site links back to Wiktionary, describes the source and licensing in the footer, and links each lexical record to its source shard. See [CORPUS-LICENSE.md](../CORPUS-LICENSE.md). The website code is MIT licensed; dictionary content retains its upstream terms.

## Browser QA record (2026-10-31)

The generated site artifact was inspected with Chromium at desktop (1365 px) and mobile (390 px) widths. A local test verified populated statistics, search for `banqueta`, the Mexico filter, a 60-word Mexican vocabulary discovery sample, System-to-Dark switching, and no mobile horizontal overflow. No browser page errors occurred in the test. Because the environment could not access remote sites, JSON index requests were fulfilled from the **actual generated index files**, while the external GitHub lexical shard response was mocked; this does **not** prove live Pages or raw.githubusercontent.com delivery. Real deployed cross-origin fetches and phone QA remain outstanding.

## Mexican flag identity

The browser favicon, header mark and hero badge use the actual Mexican flag with its national coat of arms, sourced from the MIT-licensed [flag-icons](https://github.com/lipis/flag-icons) project. The website includes the copyright/license text in `flag-license.txt`; the site's footer links to the original artwork. A subtle green-white-red ribbon sits below the navigation. The design follows System/Light/Dark themes and remains responsive on narrow screens.


## Independent Mexican spoken-frequency evidence

The optional Git-tracked `evidence/activ-es-v02.jsonl` sidecar contains observed surface-word frequencies from Mexican, Spanish, and Argentine film dialogue (ACTIV-ES v0.2). The full source file is not bundled into the canonical Wiktionary lexicon. When present, the Rust site builder validates the sidecar manifest, attaches comparable per-100,000-word rates to matching headword results, and the website offers **Rank by frequency in Mexican film dialogue**. Selecting an attested word shows the source-specific rates and Mexican film dispersion.

The site **never** turns frequency evidence into a Mexico-specific definition label. Words attested in dialogue can be ordinary general Spanish, and film subtitle data is not representative of all spoken or written Mexico. The sidecar and site copy of [ACTIV-ES's GPL-2.0 license](../evidence/ACTIV-ES-LICENSE) remain separate from Wiktionary content licensing. Import details and limitations are documented in [evidence/README.md](../evidence/README.md).


## Browse commonly attested Mexican film-dialogue words

The **Common in Mexican film dialogue** button loads a small `dialogue.json` list of up to 500 distinct word spellings ranked by observed Mexico-corpus relative frequency. Both the browse list and the underlying search indexes come from the separately licensed ACTIV-ES v0.2 frequency sidecar and the Git-tracked Wiktionary lexicon. This discovery mode does **not** assert that these words have uniquely Mexican meanings. The original English definitions, sense-level regional tags, source provenance, and license separation remain intact.

## Narrow-phone layout regression (320 CSS pixels)

The live Chromium browser QA also emulates a 320-pixel phone and fails if the actual layout viewport grows wider than the requested device width. The narrow header conserves space by hiding the top navigation's duplicate GitHub Source link; the project remains linked in the footer. This is a layout automation check, not a claim to have tested a physical phone.

## Live browser QA (2026-10-31)

The [no-npm Chromium CDP test](../scripts/site-browser-smoke.mjs) runs from GitHub Actions and exercises the **public deployed website**, rather than mocked local pages. It successfully searched `banqueta`, fetched its live English detail, switched Dark/System theme, loaded 60 Mexican film-dialogue and 60 Mexico-tagged discovery candidates, and checked mobile widths of 390 and 320 CSS pixels. A regression exposed a 336px expanded layout on a nominal 320px screen; narrow-header CSS corrected that to 320px. The [post-deployment live check](https://github.com/sguzman/mexilex/actions/runs/41248347398) passed after the fix. This is emulated-device QA, **not** a claim that a physical phone was tested.

## Separately attributed Spanish Wiktionary Mexico spellings

An independently licensed **WIKCIONARIO MX** badge is generated for headword spellings that appear in Spanish Wiktionary with at least one explicitly Mexico-tagged sense. The source is a small, normalized, Git-tracked sidecar with a SHA-256 manifest and [CC BY-SA/GFDL attribution](../evidence/WIKCIONARIO-SOURCE.md). It contains 1,912 distinct normalized spellings; 1,235 match non-inflection-only English-defined headwords. The indicator and its detail-pane link do **not** identify an English sense as Mexican and never alter the existing Mexico-specific English definition badges. The rest are retained in the independent evidence data without manufacturing English definitions. Narrow mobile layouts use wrapping badges rather than horizontal overflow.

### Live external regression evidence (2026-10-31)

The site now displays **WIKCIONARIO MX** for source-verified word spellings such as `banqueta` while retaining the independently sourced English `sidewalk` meaning. The [public Pages deployment](https://github.com/sguzman/mexilex/actions/runs/41250676458) succeeded; the [real HTTP and Chromium check](https://github.com/sguzman/mexilex/actions/runs/41250901603) verified the `banqueta` search-row evidence, source link, normal meanings, desktop and narrow-phone layout. The [standalone no-mocks browser QA](https://github.com/sguzman/mexilex/actions/runs/41250901523) also passed.
