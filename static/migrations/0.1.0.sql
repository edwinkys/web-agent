CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS version (
    id BOOL PRIMARY KEY DEFAULT true,
    version TEXT NOT NULL,
    CONSTRAINT single_row_table CHECK (id)
);

INSERT INTO version (version)
VALUES ('0.1.0');
