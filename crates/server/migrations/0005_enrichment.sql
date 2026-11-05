-- Metadata from the Spotify Web API, filled in gradually by `spotify-server enrich`.

-- Albums (and podcast shows, album_type 'show') arrive embedded in track and episode
-- responses, so they never cost a request of their own.
CREATE TABLE spotify_albums (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    album_type TEXT NOT NULL,
    release_date TEXT,
    release_year INT,
    image_source_url TEXT,
    -- Object key in the artwork bucket once downloaded.
    image_key TEXT,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- A row exists once fetching was attempted; status says how it went.
CREATE TABLE spotify_artists (
    id TEXT PRIMARY KEY,
    name TEXT,
    genres TEXT[] NOT NULL DEFAULT '{}',
    image_source_url TEXT,
    image_key TEXT,
    status TEXT NOT NULL CHECK (status IN ('ok', 'not_found', 'failed')),
    attempts INT NOT NULL DEFAULT 0,
    last_error TEXT,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE item_meta (
    uri TEXT PRIMARY KEY REFERENCES items (uri) ON DELETE CASCADE,
    duration_ms INT,
    explicit BOOLEAN,
    album_id TEXT REFERENCES spotify_albums (id),
    status TEXT NOT NULL CHECK (status IN ('ok', 'not_found', 'failed')),
    attempts INT NOT NULL DEFAULT 0,
    last_error TEXT,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX item_meta_album_idx ON item_meta (album_id);

-- Every credited artist of a track, in credit order (position 0 is the primary artist).
CREATE TABLE item_artists (
    item_uri TEXT NOT NULL REFERENCES items (uri) ON DELETE CASCADE,
    position INT NOT NULL,
    artist_id TEXT NOT NULL,
    artist_name TEXT NOT NULL,
    PRIMARY KEY (item_uri, position)
);

CREATE INDEX item_artists_artist_idx ON item_artists (artist_id);

-- One row per enrich run; the rolling 24h API budget is the sum of api_requests.
CREATE TABLE enrich_runs (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ,
    api_requests INT NOT NULL DEFAULT 0,
    image_downloads INT NOT NULL DEFAULT 0,
    items_done INT NOT NULL DEFAULT 0,
    artists_done INT NOT NULL DEFAULT 0,
    -- Set when Spotify answered 429 with a long Retry-After; runs skip until then.
    rate_limited_until TIMESTAMPTZ,
    note TEXT
);

CREATE INDEX enrich_runs_started_idx ON enrich_runs (started_at);

-- What to fetch next: tracks and episodes without metadata, and artists credited on fetched
-- tracks, scored by listening time (an artist by all of its tracks, so it follows its most
-- played tracks). Failed fetches are retried up to 5 times; not_found never is.
CREATE VIEW enrich_queue AS
WITH listening AS (
    SELECT item_uri, sum(ms_played)::bigint AS ms
    FROM plays
    WHERE item_uri IS NOT NULL
    GROUP BY item_uri
)
SELECT 'item' AS target, i.uri AS id, i.kind, coalesce(l.ms, 0) AS score
FROM items i
LEFT JOIN listening l ON l.item_uri = i.uri
LEFT JOIN item_meta m ON m.uri = i.uri
WHERE i.kind IN ('track', 'episode')
  AND (m.uri IS NULL OR (m.status = 'failed' AND m.attempts < 5))
UNION ALL
SELECT 'artist', a.artist_id, 'artist', sum(coalesce(l.ms, 0))::bigint
FROM item_artists a
LEFT JOIN listening l ON l.item_uri = a.item_uri
LEFT JOIN spotify_artists s ON s.id = a.artist_id
WHERE s.id IS NULL OR (s.status = 'failed' AND s.attempts < 5)
GROUP BY a.artist_id;

-- Artists are identified by name elsewhere (exports have no artist ids). Map each name to
-- the Spotify artist with the most listening under that name. Refreshed after each run.
CREATE MATERIALIZED VIEW artist_by_name AS
SELECT DISTINCT ON (a.artist_name) a.artist_name AS name, a.artist_id
FROM item_artists a
LEFT JOIN plays p ON p.item_uri = a.item_uri
GROUP BY a.artist_name, a.artist_id
ORDER BY a.artist_name, sum(p.ms_played) DESC NULLS LAST, a.artist_id;

CREATE UNIQUE INDEX artist_by_name_idx ON artist_by_name (name);
