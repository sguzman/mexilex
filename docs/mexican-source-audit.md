# Mexican Spanish enrichment: source audit

Status: **ACTIV-ES v0.2 source-frequency sidecar imported and versioned**. Wiktionary remains the canonical lexical corpus; Mexico-specific sense annotations are unchanged. No commercial or unlicensed bulk dictionary has been merged.

## Why ordinary Spanish words belong in Mexilex

The `es-MX` target means words useful to a learner in Mexico, **not** solely words that differ from Spain. An unlabeled Wiktionary entry is not evidence of absence from Mexico. Likewise, a Mexico-specific label can apply to one sense without applying to every definition of a word.

The current dataset records 813,928 entries and 879,019 senses (many entries are conjugated or inflected). It has 2,115 explicitly Mexico-labeled senses, which is only a partial annotation of Mexican usage.

## Sources evaluated (2026-10-31)

| Source | What it contributes | Access and rights | Decision |
| --- | --- | --- | --- |
| [English Wiktionary via Kaikki](https://kaikki.org/dictionary/Spanish/) | Spanish headwords, English definitions, forms, examples, regional labels | Structured source licensed independently of our MIT code; attribution/share-alike required | **Imported**: first Git corpus, original dump kept out of Git |
| [ACTIV-ES](https://github.com/francojc/activ-es) | Comparable spoken-dialogue word frequencies for Mexico, Argentina and Spain, including `mx_orf` and dispersion `mx_ord` | GPL-2.0-only data, originally derived from film subtitles, redistributed with its independent source license and pinned upstream commit | **Imported as a separate GPL-2.0 evidence layer**: 12,528 attested exact headword spellings, from 21,280 matched source headword spellings; never treated as a regional meaning flag |
| [CIEMPIESS LIGHT](https://huggingface.co/datasets/ciempiess/ciempiess_light) | 16,663 transcribed Central-Mexican radio-conversation segments, complementary to film dialogue | Explicit CC BY-SA 4.0 dataset license; version-pinned text TSV and SHA-256 verified, audio not needed | **Imported**: 167,118 token occurrences; 6,250 surface spellings occurring at least twice in independent Git-tracked aggregate counts, with 5,951 singleton spellings suppressed; neither a general-national frequency estimate nor a sense label |
| [OpenSLR SLR21](https://openslr.org/21/) | General Spanish word-frequency counts (not country-specific definitions) | CC BY-SA 3.0 US, publicly downloadable | **Fallback** for general frequency, not a Mexico-specific ranking |
| [UNAM Mexican lexicon project](https://repositorio.unam.mx/contenidos/ficha/diccionario-lexico-del-espanol-de-mexico-y-sus-frecuencias-de-uso-1502104) | Research describing a Mexican lexical-frequency database | The UNAM repository listing displays CC BY 4.0 terms, but an independent bulk lexicon file and rights to its contents have **not** been verified | **Research**; do not mistake the project listing for a downloadable dataset |
| [El Colegio de México corpora](https://repositorio-cell.colmex.mx/) | Mexican written/spoken usage evidence and frequency methods | Searchable corpus repository; rights and authorized bulk reuse must be verified per collection | **Reference**, not imported |
| [CELEN vocabulary lists](https://sites.google.com/view/celen/descarga) | General and tagged Spanish frequency lists | CC BY-NC 4.0 (noncommercial) | **Hold**: do not redistribute in the open, potentially commercial trainer without an explicit licensing decision |
| [DEM](https://dem.colmex.mx/) | Lexical definitions based on Mexican Spanish | Readable online, but no open bulk database redistribution verified | **Reference**, not scraped |

## Proposed enrichment rule

Any future frequency source must be versioned and attributed independently from Wiktionary. Frequency counts reflect *attested occurrence in the chosen corpus*, not a determination that a word is (or is not) Mexican. For example, an ACTIV-ES Mexico rate can influence study priority while the dictionary's `mexican_usage` sense flag remains a distinct, source-verified semantic label.

Avoid creating an unqualified `mexican=true` flag simply because a word appears in Mexican film dialogue.

Before an importer is promoted to a tracked release:

1. Verify the exact file, column definitions, source version, checksum, and redistribution permission for transformed outputs.
2. Keep source-specific frequency evidence in an **independent provenance namespace**, rather than rewriting Wiktionary glosses or Mexican regional labels.
3. Test POS/lemma matching and collision handling; form frequencies must not be misrepresented as headword frequencies.
4. Document methodology, source coverage, time period and bias (spoken vs written, dialogue vs real conversation, sample size).

This intentionally prevents recreating Linguarium as an unlimited cross-language ontology. Lexical information is included only when it improves meaning, browsing, or flashcard selection.

## First verified frequency snapshot (2026-10-31)

The [ACTIV-ES v0.2 Git-tracked evidence](../evidence/README.md) was generated from 80,787 source word-frequency rows, with 21,280 exact case-insensitive surface-word matches to our canonical non-inflection-only headwords; 12,528 had positive observed rates in the Mexican film-dialogue subset. The transformation excluded zero-Mexico observations and form-only records; a missing observation **does not** imply a word is never used in Mexico. A pinned source commit, SHA-256, independent GPL-2.0 license and field-level provenance are included. The website adds an opt-in rank-by-film-dialogue-frequency switch and displays Mexico/Spain/Argentina comparison rates for attested words. No Wiktionary definition or `mexican_usage` flag was altered.

## Explicitly excluded restricted dataset

Mark Davies's [Corpus del Español, Mexico database at Gonzaga University](https://repository.gonzaga.edu/corp_span/57/) publicly describes downloadable Mexican text/corpus tables, but its published restrictions prohibit distribution outside Gonzaga and using the data to create commercial or consumer software. **Do not import, normalize, or rehost that database in Mexilex.** Searchable or technically downloadable is not equivalent to having redistribution rights.

## Independently Mexico-tagged Spanish Wiktionary entries (2026-10-31)

Spanish Wiktionary's published [Mexico-tagged Español subset](https://kaikki.org/eswiktionary/Espa%C3%B1ol/tags/p9/Mexico/index.html) was fetched and audited as a separate CC BY-SA/GFDL source (not commercial Mexican dictionary content). The source held 1,966 entries with 2,304 Mexico-tagged Spanish senses; the normalized Git sidecar stores 1,946 unique word/POS pairs across 1,912 distinct case-normalized spellings. Exactly 1,235 spellings match English-Wiktionary non-form-only headwords. The remaining 677 are source-evidenced spellings **without guaranteed English definitions**, not missing Mexican meaning tags that we should invent.

The site uses this as spelling-level corroboration, and every affected entry can link to its Spanish Wiktionary page. The foreign-language sense meanings were **not translated, mapped, or automatically attached to English senses**. It therefore adds independent regional evidence without increasing the 2,115 source-verified English-Wiktionary Mexico-specific senses. Reproducible importer, license and checksum: [evidence/WIKCIONARIO-SOURCE.md](../evidence/WIKCIONARIO-SOURCE.md), [manifest](../evidence/eswiktionary-mexico.manifest.json).

## Independently licensed Mexican radio speech (2026-10-31)

The [CIEMPIESS LIGHT](../evidence/CIEMPIESS-SOURCE.md) UNAM corpus provides **spontaneous radio speech** instead of scripted film subtitles. A pinned 1.6 MB TSV contains 16,663 annotated utterances and 167,118 orthographic word tokens. We commit only aggregated counts for the 6,250 unique surface spellings appearing at least twice, with the count of distinct utterances in which each token appears. This source is **CC BY-SA 4.0**, independent of the GPL-2.0 ACTIV-ES film data and Wiktionary material. See [manifest](../evidence/ciempiess-light-frequency.manifest.json) for the exact source and output SHA-256 hashes.

The corpus overrepresents discussions of law, university life and broadcast interviews, and the dataset's speaker representation is not balanced. It is **not** a representative cross-section of spoken or written Spanish nationwide. Surface word counts never automatically label a Wiktionary meaning as Mexican, infer a lemma or replace personal learning data. The site displays this evidence as an optional, clearly attributed detail panel without altering its lexical search index.
