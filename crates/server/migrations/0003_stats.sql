-- Time range covered by each account's extended history, maintained by the importer. Basic
-- plays inside it duplicate extended plays and are excluded from all_plays; looking the range
-- up per basic play was the bulk of every stats query.
CREATE TABLE extended_coverage (
    account_id BIGINT PRIMARY KEY REFERENCES accounts (id) ON DELETE CASCADE,
    first_ts TIMESTAMPTZ NOT NULL,
    last_ts TIMESTAMPTZ NOT NULL
);

INSERT INTO extended_coverage (account_id, first_ts, last_ts)
SELECT account_id, min(ts), max(ts) FROM plays GROUP BY account_id;

-- Expose the item kind on all_plays so stats can separate music from podcasts. Basic plays
-- have no kind (no URI). New columns must come last for CREATE OR REPLACE VIEW.
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
    i.kind
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
    NULL
FROM basic_plays b
LEFT JOIN extended_coverage c ON c.account_id = b.account_id
WHERE c.account_id IS NULL OR b.end_time NOT BETWEEN c.first_ts AND c.last_ts;

-- The one place stats filters live, so every stats query applies them identically. A single
-- SELECT in a STABLE SQL function is inlined by the planner, so filters still reach indexes.
--   accounts: empty means all accounts
--   kinds: item kinds to include; basic plays (no kind) count as tracks
--   min_ms: plays shorter than this are ignored (Spotify counts a stream at 30s)
--   item_uri / artist: optional narrowing to one item or artist
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
