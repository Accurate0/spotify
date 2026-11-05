//! Batch loaders so lists of plays don't query their item and account one by one.

use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::Loader;
use sqlx::PgPool;

use super::catalog::Item;
use super::objects::Account;

pub struct AccountLoader(pub PgPool);

impl Loader<i64> for AccountLoader {
    type Value = Account;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Account>, Self::Error> {
        let accounts = sqlx::query_as!(
            Account,
            "SELECT id, slug, display_name, spotify_username FROM accounts WHERE id = ANY($1)",
            keys
        )
        .fetch_all(&self.0)
        .await?;

        Ok(accounts.into_iter().map(|a| (a.id, a)).collect())
    }
}

pub struct ItemLoader(pub PgPool);

impl Loader<String> for ItemLoader {
    type Value = Item;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Item>, Self::Error> {
        let items = sqlx::query_as!(
            Item,
            "SELECT uri, kind, name, artist, album FROM items WHERE uri = ANY($1)",
            keys
        )
        .fetch_all(&self.0)
        .await?;

        Ok(items.into_iter().map(|i| (i.uri.clone(), i)).collect())
    }
}

/// Spotify metadata for an item, once enrichment has fetched it.
#[derive(Clone)]
pub struct ItemMeta {
    pub duration_ms: Option<i32>,
    pub explicit: Option<bool>,
    pub album_image_key: Option<String>,
    pub release_date: Option<String>,
    pub release_year: Option<i32>,
}

pub struct ItemMetaLoader(pub PgPool);

impl Loader<String> for ItemMetaLoader {
    type Value = ItemMeta;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, ItemMeta>, Self::Error> {
        let rows = sqlx::query!(
            "SELECT m.uri, m.duration_ms, m.explicit,
                    a.image_key, a.release_date, a.release_year
             FROM item_meta m LEFT JOIN spotify_albums a ON a.id = m.album_id
             WHERE m.uri = ANY($1) AND m.status = 'ok'",
            keys
        )
        .fetch_all(&self.0)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let meta = ItemMeta {
                    duration_ms: r.duration_ms,
                    explicit: r.explicit,
                    album_image_key: r.image_key,
                    release_date: r.release_date,
                    release_year: r.release_year,
                };
                (r.uri, meta)
            })
            .collect())
    }
}

/// Credited artist names of an item, in credit order.
pub struct ItemArtistsLoader(pub PgPool);

impl Loader<String> for ItemArtistsLoader {
    type Value = Vec<String>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Vec<String>>, Self::Error> {
        let rows = sqlx::query!(
            "SELECT item_uri, artist_name FROM item_artists
             WHERE item_uri = ANY($1) ORDER BY item_uri, position",
            keys
        )
        .fetch_all(&self.0)
        .await?;

        let mut credits: HashMap<String, Vec<String>> = HashMap::new();
        for row in rows {
            credits
                .entry(row.item_uri)
                .or_default()
                .push(row.artist_name);
        }
        Ok(credits)
    }
}

/// Spotify metadata for an artist name (see the artist_by_name view).
#[derive(Clone)]
pub struct ArtistMeta {
    pub genres: Vec<String>,
    pub image_key: Option<String>,
}

pub struct ArtistMetaLoader(pub PgPool);

impl Loader<String> for ArtistMetaLoader {
    type Value = ArtistMeta;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, ArtistMeta>, Self::Error> {
        let rows = sqlx::query!(
            r#"SELECT b.name AS "name!", s.genres, s.image_key
               FROM artist_by_name b JOIN spotify_artists s ON s.id = b.artist_id
               WHERE b.name = ANY($1) AND s.status = 'ok'"#,
            keys
        )
        .fetch_all(&self.0)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let meta = ArtistMeta {
                    genres: r.genres,
                    image_key: r.image_key,
                };
                (r.name, meta)
            })
            .collect())
    }
}
