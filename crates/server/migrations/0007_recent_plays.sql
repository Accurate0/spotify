-- Recent plays from the Web API's recently-played endpoint, filling the weeks before the next
-- extended history export. Like basic plays, they are hidden inside extended_coverage, so once
-- an export covers a period its plays win without matching rows one by one.
DROP FUNCTION scoped_plays;

-- An account's Spotify user authorisation (user-read-recently-played). `last_played_at` is
-- the newest play fetched, the `after` cursor for the next poll.
CREATE TABLE spotify_links (
    account_id BIGINT PRIMARY KEY REFERENCES accounts (id) ON DELETE CASCADE,
    spotify_user_id TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    scope TEXT NOT NULL,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_played_at TIMESTAMPTZ,
    last_polled_at TIMESTAMPTZ,
    -- Set when Spotify rejects the refresh token; polling skips the link until relinked.
    revoked_at TIMESTAMPTZ
);

-- The API reports no play length, platform or reasons, so only the track and time are kept.
CREATE TABLE recent_plays (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    played_at TIMESTAMPTZ NOT NULL,
    item_uri TEXT NOT NULL REFERENCES items (uri),
    context_uri TEXT,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, played_at)
);

CREATE INDEX recent_plays_item_idx ON recent_plays (item_uri);

-- `source` says where each play came from: 'extended', 'basic' or 'recent'. Recent plays have
-- no ms_played. New columns must come last for CREATE OR REPLACE VIEW.
CREATE OR REPLACE VIEW all_plays AS
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
    TRUE AS is_extended,
    i.kind,
    p.platform,
    p.conn_country,
    p.shuffle,
    p.offline,
    p.reason_start,
    p.reason_end,
    'extended' AS source
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
    FALSE,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    'basic'
FROM basic_plays b
LEFT JOIN extended_coverage c ON c.account_id = b.account_id
WHERE c.account_id IS NULL OR b.end_time NOT BETWEEN c.first_ts AND c.last_ts
UNION ALL
SELECT
    r.account_id,
    r.played_at,
    r.item_uri,
    i.name,
    i.artist,
    i.album,
    NULL,
    NULL,
    FALSE,
    FALSE,
    i.kind,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    'recent'
FROM recent_plays r
JOIN items i ON i.uri = r.item_uri
LEFT JOIN extended_coverage c ON c.account_id = r.account_id
WHERE c.account_id IS NULL OR r.played_at NOT BETWEEN c.first_ts AND c.last_ts;

-- As in 0004_insights.sql, except plays without a length (recent plays, which the API only
-- lists after 30 seconds) pass the min_ms filter.
CREATE FUNCTION scoped_plays(
    accounts BIGINT[],
    from_ts TIMESTAMPTZ,
    to_ts TIMESTAMPTZ,
    kinds TEXT[],
    min_ms INT,
    item_uri TEXT,
    artist TEXT
) RETURNS SETOF all_plays
LANGUAGE sql STABLE AS $$
    SELECT *
    FROM all_plays a
    WHERE (cardinality(accounts) = 0 OR a.account_id = ANY(accounts))
      AND (from_ts IS NULL OR a.ts >= from_ts)
      AND (to_ts IS NULL OR a.ts < to_ts)
      AND (a.kind = ANY(kinds) OR (a.kind IS NULL AND 'track' = ANY(kinds)))
      AND (a.ms_played IS NULL OR a.ms_played >= min_ms)
      AND (scoped_plays.item_uri IS NULL OR a.item_uri = scoped_plays.item_uri)
      AND (scoped_plays.artist IS NULL OR a.artist = scoped_plays.artist)
$$;
