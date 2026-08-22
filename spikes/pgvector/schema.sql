-- Spike schema for pgvector recall parity probe (READ-ONLY of production vault).
-- facts ← semantic_vault; honeypot ← honeypot; evidence ← evidence.

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE facts (
    id            TEXT PRIMARY KEY,
    content       TEXT NOT NULL,
    content_norm  tsvector,
    embedding     vector(1024),
    confidence    DOUBLE PRECISION,
    created_at    TEXT,
    source_file   TEXT
);

CREATE TABLE honeypot (
    id             TEXT PRIMARY KEY,
    vault_id       TEXT,
    content        TEXT NOT NULL,
    content_norm   tsvector,
    embedding      vector(1024),
    is_latest      INTEGER NOT NULL DEFAULT 1,
    supersedes_id  TEXT,
    confidence     DOUBLE PRECISION,
    source_file    TEXT
);

CREATE TABLE evidence (
    id            TEXT PRIMARY KEY,
    fact_id       TEXT NOT NULL,
    evidence_text TEXT NOT NULL,
    content_norm  tsvector,
    char_start    INTEGER,
    char_end      INTEGER
);

-- HNSW for cosine distance (<=>). Small corpus (478 latest) — no scale win expected.
CREATE INDEX honeypot_embedding_hnsw
    ON honeypot
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

CREATE INDEX honeypot_content_norm_gin ON honeypot USING gin (content_norm);
CREATE INDEX facts_content_norm_gin ON facts USING gin (content_norm);
CREATE INDEX evidence_content_norm_gin ON evidence USING gin (content_norm);
