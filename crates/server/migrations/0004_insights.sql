-- Listening context for breakdowns (device, country, shuffle, offline, start/end reasons).
-- scoped_plays returns all_plays rows, so it is dropped while the view gains columns.
DROP FUNCTION scoped_plays;

-- Exports mix verbose platform strings ("Android OS 11 API 30 (Google, Pixel 5)",
-- "OS X 10.11.6 [x86 8]") with short ones ("android", "osx"); group both into families.
CREATE FUNCTION device_family(platform TEXT) RETURNS TEXT
LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE
        WHEN platform IS NULL THEN NULL
        WHEN platform ILIKE '%android%' THEN 'Android'
        WHEN platform ILIKE 'ios%' OR platform ILIKE '%iphone%' OR platform ILIKE '%ipad%' THEN 'iOS'
        WHEN platform ILIKE 'os x%' OR platform ILIKE 'osx%' OR platform ILIKE '%macos%' THEN 'macOS'
        WHEN platform ILIKE '%windows%' THEN 'Windows'
        WHEN platform ILIKE '%linux%' THEN 'Linux'
        WHEN platform ILIKE '%web%' THEN 'Web player'
        ELSE 'Other'
    END
$$;

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
    p.reason_end
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
    NULL
FROM basic_plays b
LEFT JOIN extended_coverage c ON c.account_id = b.account_id
WHERE c.account_id IS NULL OR b.end_time NOT BETWEEN c.first_ts AND c.last_ts;

-- Unchanged from 0003_stats.sql.
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
      AND a.ms_played >= min_ms
      AND (scoped_plays.item_uri IS NULL OR a.item_uri = scoped_plays.item_uri)
      AND (scoped_plays.artist IS NULL OR a.artist = scoped_plays.artist)
$$;
