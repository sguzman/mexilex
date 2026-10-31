PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS entries (
    key TEXT PRIMARY KEY,
    word TEXT NOT NULL,
    pos TEXT NOT NULL,
    etymology_number INTEGER,
    forms_json TEXT NOT NULL,
    tags_json TEXT NOT NULL,
    source_edition TEXT NOT NULL,
    source_line INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS entries_word_idx ON entries(word);
CREATE TABLE IF NOT EXISTS senses (
    key TEXT PRIMARY KEY,
    entry_key TEXT NOT NULL REFERENCES entries(key) ON DELETE CASCADE,
    sense_index INTEGER NOT NULL,
    definition_en TEXT NOT NULL,
    glosses_json TEXT NOT NULL,
    tags_json TEXT NOT NULL,
    raw_tags_json TEXT NOT NULL,
    examples_json TEXT NOT NULL,
    mexican_usage INTEGER NOT NULL CHECK (mexican_usage IN (0, 1))
);
CREATE INDEX IF NOT EXISTS senses_entry_idx ON senses(entry_key);
CREATE INDEX IF NOT EXISTS senses_mexican_usage_idx ON senses(mexican_usage);
