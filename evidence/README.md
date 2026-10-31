# ACTIV-ES Mexico film-dialogue frequency evidence

**This source is separate from the Wiktionary lexical corpus.** It represents observed word occurrence in film subtitles from Mexican productions, compared with Spanish and Argentine films. It does **not** prove that a given meaning is uniquely Mexican, nor does absence prove a word is unused in Mexico.

Source: [francojc/activ-es](https://github.com/francojc/activ-es), ACTIV-ES v0.2, wordlists/plain/aes1grams.csv. Original authors: Jerid Francom, Mans Hulden, Adam Ussishkin. Pinned upstream commit: `e42124efbd0c6c30fc8b6a93e5d3a1e9bc4c7e08`.

- `mx_orf`, `es_orf`, `ar_orf`: observed relative frequency per 100,000 words, by country corpus.
- `mx_ord`: Mexican film-document dispersion per 10 documents.
- Source data: **GPL-2.0-only**, separately attributed; see [ACTIV-ES-LICENSE](ACTIV-ES-LICENSE).
- Lexicon: independently licensed Wiktionary data; repository `LICENSE` applies to Mexilex code, **not all data**.
- Selection: case-insensitive **exact surface-word** matches to Git-tracked entries that are not exclusively inflected forms. No diacritic folding, POS/sense allocation, or invented Mexican tags. Entries observed only in Spain/Argentina are excluded from the Mexico-attested subset.
- CSV source dump is an external build input, not stored here. Processed JSONL is committed to `evidence/` as a separate provenance layer.

The generated file `activ-es-v02.jsonl` has one JSON array per line: `[word, mexico_per_100k, spain_per_100k, argentina_per_100k, mexico_dispersion_per_10_documents]`. The adjacent `activ-es-v02.manifest.json` records the source SHA-256 and record counts. Do not use these values as dictionary sense labels, broad statements about all speakers, or a general frequency ranking of written Mexican Spanish.

`cargo run --release -- import-mx-frequency data/aes1grams.csv corpus evidence/activ-es-v02.jsonl` creates the sidecar. The [seed workflow](../.github/workflows/seed-mx-frequency.yml) downloads the exact pinned upstream CSV, verifies data and commits the normalized sidecar to `main`. Frequency data must remain explicitly separate in websites and exports.
