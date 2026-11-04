CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE accounts (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9][a-z0-9-]*$'),
    display_name TEXT NOT NULL,
    spotify_username TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One upload of a directory or zip. `as_of` (newest timestamp in any of its files, computed
-- by the client) decides which export's values win when plays overlap.
CREATE TABLE exports (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    as_of TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finalized_at TIMESTAMPTZ
);

CREATE INDEX exports_account_idx ON exports (account_id);

CREATE TABLE export_files (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    export_id BIGINT NOT NULL REFERENCES exports (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    rows_total INT NOT NULL DEFAULT 0,
    rows_inserted INT NOT NULL DEFAULT 0,
    rows_merged INT NOT NULL DEFAULT 0,
    imported_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, sha256)
);

CREATE INDEX export_files_export_idx ON export_files (export_id);

-- Tracks, episodes and audiobook chapters. Names are mutable across exports ("Charli XCX" ->
-- "Charli xcx"); the values from the newest export win.
CREATE TABLE items (
    uri TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('track', 'episode', 'audiobook')),
    name TEXT,
    artist TEXT,
    album TEXT,
    as_of TIMESTAMPTZ NOT NULL
);

CREATE INDEX items_name_trgm_idx ON items USING gin (name gin_trgm_ops);
CREATE INDEX items_artist_trgm_idx ON items USING gin (artist gin_trgm_ops);
CREATE INDEX items_artist_idx ON items (artist);

CREATE TABLE plays (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    ts TIMESTAMPTZ NOT NULL,
    item_uri TEXT REFERENCES items (uri),
    ms_played INT NOT NULL,
    platform TEXT,
    conn_country TEXT,
    reason_start TEXT,
    reason_end TEXT,
    shuffle BOOLEAN,
    skipped BOOLEAN,
    offline BOOLEAN,
    offline_ts TIMESTAMPTZ,
    incognito BOOLEAN,
    is_video BOOLEAN NOT NULL,
    -- Only set when item_uri is null, so these plays can still be matched and displayed.
    raw_name TEXT,
    raw_artist TEXT,
    raw_album TEXT,
    -- as_of of the export whose values this row currently holds.
    source_as_of TIMESTAMPTZ NOT NULL,
    first_file_id BIGINT NOT NULL REFERENCES export_files (id),
    last_file_id BIGINT NOT NULL REFERENCES export_files (id)
);

CREATE INDEX plays_dedupe_idx ON plays (account_id, item_uri, ms_played, ts);
CREATE INDEX plays_account_ts_idx ON plays (account_id, ts);
CREATE INDEX plays_item_idx ON plays (item_uri);
