-- Account data streaming history: minute precision, no URIs. Overlapping account exports
-- (2021 and 2022 both cover Apr-Aug 2021) repeat identical rows, which collapse here.
CREATE TABLE basic_plays (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    end_time TIMESTAMPTZ NOT NULL,
    artist TEXT,
    track TEXT,
    ms_played INT NOT NULL,
    file_id BIGINT NOT NULL REFERENCES export_files (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX basic_plays_dedupe_idx
    ON basic_plays (account_id, end_time, artist, track, ms_played) NULLS NOT DISTINCT;

-- Every play used for stats: extended plays, plus basic plays outside the account's extended
-- coverage (basic history duplicates extended history wherever both exist).
CREATE VIEW all_plays AS
SELECT
    p.account_id,
    p.ts,
    p.item_uri,
    coalesce(i.name, p.raw_name) AS name,
    coalesce(i.artist, p.raw_artist) AS artist,
    coalesce(i.album, p.raw_album) AS album,
    p.ms_played,
    p.skipped,
    p.is_video,
    TRUE AS is_extended
FROM plays p
LEFT JOIN items i ON i.uri = p.item_uri
UNION ALL
SELECT
    b.account_id,
    b.end_time,
    NULL,
    b.track,
    b.artist,
    NULL,
    b.ms_played,
    NULL,
    FALSE,
    FALSE
FROM basic_plays b
WHERE NOT EXISTS (
    SELECT 1
    FROM (SELECT min(ts) AS first, max(ts) AS last FROM plays WHERE account_id = b.account_id) c
    WHERE b.end_time BETWEEN c.first AND c.last
);

CREATE TABLE searches (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    ts TIMESTAMPTZ NOT NULL,
    platform TEXT,
    query TEXT NOT NULL,
    interaction_uris TEXT[] NOT NULL,
    file_id BIGINT NOT NULL REFERENCES export_files (id) ON DELETE CASCADE,
    UNIQUE (account_id, ts, query)
);

-- Snapshot kinds (library, playlists, follow, marquee, inferences) describe state at export
-- time. Every export's snapshot is kept; the current one is the newest export (by as_of)
-- that contains files of that kind. Playlist<N>.json can be split over several files, so
-- "current" selects an export, not a single file.
CREATE VIEW current_snapshot_files AS
SELECT f.id, f.account_id, f.kind
FROM export_files f
JOIN exports e ON e.id = f.export_id
WHERE f.kind IN ('LIBRARY', 'PLAYLIST', 'FOLLOW', 'MARQUEE', 'INFERENCES')
AND e.id = (
    SELECT e2.id
    FROM export_files f2
    JOIN exports e2 ON e2.id = f2.export_id
    WHERE f2.account_id = f.account_id AND f2.kind = f.kind
    ORDER BY e2.as_of DESC, e2.id DESC
    LIMIT 1
);

CREATE TABLE library_items (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES export_files (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('track', 'album', 'artist')),
    uri TEXT,
    name TEXT NOT NULL,
    artist TEXT,
    album TEXT
);

CREATE INDEX library_items_file_idx ON library_items (file_id);

CREATE TABLE playlists (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES export_files (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    -- Exports have no playlist id; the name is the only identifier.
    name TEXT NOT NULL,
    last_modified DATE,
    description TEXT,
    followers INT NOT NULL
);

CREATE INDEX playlists_file_idx ON playlists (file_id);

CREATE TABLE playlist_items (
    playlist_id BIGINT NOT NULL REFERENCES playlists (id) ON DELETE CASCADE,
    position INT NOT NULL,
    uri TEXT,
    name TEXT,
    artist TEXT,
    album TEXT,
    added_date DATE,
    PRIMARY KEY (playlist_id, position)
);

CREATE TABLE follow_snapshots (
    file_id BIGINT PRIMARY KEY REFERENCES export_files (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    followers INT NOT NULL,
    following INT NOT NULL,
    dismissing INT NOT NULL
);

CREATE TABLE marquee (
    file_id BIGINT NOT NULL REFERENCES export_files (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    artist TEXT NOT NULL,
    segment TEXT NOT NULL
);

CREATE INDEX marquee_file_idx ON marquee (file_id);

CREATE TABLE inferences (
    file_id BIGINT NOT NULL REFERENCES export_files (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    value TEXT NOT NULL
);

CREATE INDEX inferences_file_idx ON inferences (file_id);
