# CIEMPIESS LIGHT — licensed Mexican spoken-language evidence

Source: [CIEMPIESS LIGHT Corpus](https://huggingface.co/datasets/ciempiess/ciempiess_light), by Carlos Daniel Hernández Mena and Abel Herrera, created by UNAM's Language Technologies Laboratory (2015–2016). The corpus is distributed under **Creative Commons Attribution–ShareAlike 4.0 International (CC BY-SA 4.0)**. Original citation: *CIEMPIESS LIGHT CORPUS: Audio and Transcripts of Mexican Spanish Broadcast Conversations*, LDC2017S23, DOI: 10.35111/64rg-yk97.

Pinned dataset revision: `e9e6dc556ba8479f54160701a7542a67de1fbe14`. Only the 1.6 MB transcript metadata TSV is imported, not the audio files. Source SHA-256: `c3887d0e4b6adefa4ef5aa80dc76f858c89ad1a131e7f2b3f49b7316cbdad1f4` (verified in [GitHub Actions](https://github.com/sguzman/mexilex/actions/runs/41252396790)).

This corpus comprises **16,663 transcribed utterances** from Central-Mexican Spanish radio interviews, mainly on legal and university topics. It is spontaneous speech rather than film dialogue, but is not a representative sample of all Mexican speakers, genders, ages, regions or topics. The dataset card notes its gender imbalance.

The Git-tracked derived sidecar `ciempiess-light-frequency.jsonl` records `[surface_word, occurrences, utterances]`. Counts are exact within the corpus but **not** a measurement of general Mexican-language frequency, word meaning or Mexican exclusivity. Only surface tokens appearing at least twice are retained, suppressing one-off names/transcription errors. No transcript segments, audio, audio IDs, speaker metadata or individual histories are redistributed.

The manifest records the source checksum, revision, licensing, normalization rule, token denominator, utterance count and output checksum. This evidence is independent of Wiktionary English definitions, Wikcionario regional labels, and GPL-2.0 ACTIV-ES film dialogue data. Do not combine their licenses or compare raw counts as though denominators or genres were equivalent.
