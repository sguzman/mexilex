# Mexican Spanish enrichment: source audit

Status: **research only**. The initial Git-tracked lexical corpus remains English-Wiktionary Spanish records with explicit Mexico-tagged senses where available. No external frequency or commercial dictionary content has been merged.

## Why ordinary Spanish words belong in Mexilex

The `es-MX` target means words useful to a learner in Mexico, **not** solely words that differ from Spain. An unlabeled Wiktionary entry is not evidence of absence from Mexico. Likewise, a Mexico-specific label can apply to one sense without applying to every definition of a word.

The current dataset records 813,928 entries and 879,019 senses (many entries are conjugated or inflected). It has 2,115 explicitly Mexico-labeled senses, which is only a partial annotation of Mexican usage.

## Sources evaluated (2026-10-31)

| Source | What it contributes | Access and rights | Decision |
| --- | --- | --- | --- |
| [English Wiktionary via Kaikki](https://kaikki.org/dictionary/Spanish/) | Spanish headwords, English definitions, forms, examples, regional labels | Structured source licensed independently of our MIT code; attribution/share-alike required | **Imported**: first Git corpus, original dump kept out of Git |
| [ACTIV-ES](https://github.com/francojc/activ-es) | Comparable spoken-dialogue word frequencies for Mexico, Argentina and Spain, including `mx_orf` and dispersion `mx_ord` | Public repository declares GPL-2.0, and data derives from film subtitles; verify redistribution and compatibility before republishing | **Candidate** for a separately licensed frequency sidecar; do not silently merge with Wiktionary into one MIT-labelled dataset |
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
