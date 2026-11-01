# Mexilex Constitution

## Purpose and independence

Mexilex is a public, reproducible lexicon of Spanish used in Mexico with English definitions. Its research objects are lexical entries and individual senses. The normalized lexical corpus, source-attributed Mexican-usage evidence, Rust processing tools, independent ontology and public read-only dictionary belong to this repository. Mexilex is not an account service or a personal learning-history database.

Mexilex remains an independent project. A public index such as Tarium may link to its published ontology and gallery, but must not copy or own the corpus or implementation. Membership of other projects requires their own explicit approval.

## Accuracy and evidence

- Store source-derived entries, senses, forms, glosses, morphological labels, regional tags and source provenance separately according to their documented relationships.
- A Mexico-specific label applies only to an individual sense when the upstream source explicitly supports it. Missing labels must never be interpreted as proof a word or sense is not used in Mexico.
- Regional attestations, film-dialogue frequencies and radio-interview observations are separate evidence types, not proof of nationwide representativeness or a verified regional meaning.
- Retain source edition, relevant licenses and attribution. Do not treat lexicon data or third-party frequency evidence as MIT-licensed merely because project-owned code is MIT.
- Corpus updates must preserve deterministic exports, source traceability and the documented review/migration gate. Uncertain sense continuity across upstream revisions must not silently reassign study records.

## Public information boundary

Never place secrets, credentials, private conversations, private notes, identifying learning data, unpublished resources, private paths or confidential source material into repository code, tests, data, issues, documentation, assets, releases, workflows, logs or commit history. Describe this public work in neutral technical language. Exclude material whose publication rights are not established.

## Public interface and governance

Maintain a machine-readable ontology at [ontology/ontology.json](ontology/ontology.json) and a truthful public gallery at [gallery/index.json](gallery/index.json). The gallery refers to the independently hosted dictionary and approved releases; it is not a second copy of the lexical corpus.

The public site remains read-only and offers System/Light/Dark appearance. User-specific spaced-repetition schedules, retention statistics and review history belong to a separate application, never to Mexilex's public data.

The owner must explicitly authorize each new external registry enrollment and any substantive change of scope. Do not interpret public visibility, shared naming, or repository adjacency as permission to register another project or process a work queue.
