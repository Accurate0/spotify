-- Before the artwork bucket was reachable, failed uploads were mistaken for broken image URLs
-- and the URLs were dropped. Re-fetch what lost them: tracks on albums without artwork, and
-- artists without an image (a few genuinely have none and are simply fetched once more).
DELETE FROM item_meta
WHERE status = 'ok'
  AND album_id IN (
      SELECT id FROM spotify_albums WHERE image_source_url IS NULL AND image_key IS NULL
  );

DELETE FROM spotify_artists
WHERE status = 'ok' AND image_source_url IS NULL AND image_key IS NULL;
