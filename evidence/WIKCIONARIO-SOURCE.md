# Mexico-tagged Español evidence from Spanish Wiktionary

The original work belongs to contributors of **Wikcionario (Spanish Wiktionary)**:
https://es.wiktionary.org/wiki/Wikcionario:Portada

Source: Kaikki postprocessed Spanish-Wiktionary Español entries tagged `Mexico`, at
https://kaikki.org/eswiktionary/Espa%C3%B1ol/tags/p9/Mexico/index.html

Independent content license: **CC BY-SA and GNU Free Documentation License (GFDL)**, following Wiktionary's upstream licensing. The transformed records and the original Spanish Wiktionary material **are not MIT-licensed**; the MIT repository license covers Mexilex code, not these records. Consult:
- https://es.wiktionary.org/wiki/Wikcionario:Derechos_de_autor
- https://creativecommons.org/licenses/by-sa/4.0/
- https://www.gnu.org/licenses/fdl-1.3.html

Every source word can be checked through the corresponding Spanish Wiktionary page (for example `banqueta` at https://es.wiktionary.org/wiki/banqueta). The generated manifest records a source SHA-256 and transformed coverage.

## Meaning and limitations

This is **spelling-level corroboration**, not a translation alignment. The separate evidence file contains `[Spanish word, Spanish-Wiktionary POS, count of Mexico-tagged Spanish senses, in-English-headword-corpus]`. The last flag is an accent-preserving, case-insensitive spelling match against non-inflection-only English Wiktionary Spanish headwords.

**Do not** set `mexican_usage=true` on any particular English-Wiktionary meaning because of this sidecar. Homographs and meanings may differ across Wiktionary editions. Other country labels and meanings remain untouched. The regional tag is an independent source's claim; it is not a statement about exclusive usage, and no word missing from this sidecar should be presumed absent from Mexico.

There are no Spanish definitions, original entry dumps, or personal study records redistributed in this sidecar. Original licensed content remains independently attributed and reusable under its own license. The normalized word-level evidence is stored in Git under `evidence/`, consistent with Mexilex's processed-data policy.
