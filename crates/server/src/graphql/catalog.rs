//! Items (tracks, episodes, audiobook chapters), artists and albums. Exports carry no artist or
//! album URIs, so artists and albums are identified by name.

use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, Enum, ID, Object};
use sqlx::PgPool;

use super::loaders::{ArtistMeta, ArtistMetaLoader, ItemArtistsLoader, ItemMeta, ItemMetaLoader};

use super::node::{NodeType, encode};
use super::page::{Page, PlainConnection};
use super::stats::{Stats, StatsFilter};

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(name = "SpotifyItemKind")]
#[graphql(remote = "spotify_export::ItemKind")]
pub enum ItemKind {
    Track,
    Episode,
    Audiobook,
}

impl ItemKind {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "track" => Some(Self::Track),
            "episode" => Some(Self::Episode),
            "audiobook" => Some(Self::Audiobook),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Item {
    pub uri: String,
    pub kind: String,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

/// `/art/albums/<id>` for an artwork bucket key like `albums/<id>.jpg`.
pub fn art_url(key: &str) -> String {
    format!("/art/{}", key.trim_end_matches(".jpg"))
}

impl Item {
    async fn meta(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<ItemMeta>> {
        Ok(ctx
            .data::<DataLoader<ItemMetaLoader>>()?
            .load_one(self.uri.clone())
            .await?)
    }

    pub async fn by_uri(pool: &PgPool, uri: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT uri, kind, name, artist, album FROM items WHERE uri = $1",
            uri
        )
        .fetch_optional(pool)
        .await
    }
}

#[Object(name = "SpotifyItem")]
impl Item {
    pub async fn id(&self) -> ID {
        encode(NodeType::Item, &self.uri)
    }

    async fn uri(&self) -> &str {
        &self.uri
    }

    async fn kind(&self) -> async_graphql::Result<ItemKind> {
        ItemKind::parse(&self.kind).ok_or_else(|| format!("unknown item kind {}", self.kind).into())
    }

    async fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The artist for tracks, the show for episodes, the book for audiobook chapters.
    async fn artist_name(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    async fn artist(&self) -> Option<Artist> {
        self.artist.clone().map(|name| Artist { name })
    }

    async fn album(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Album>> {
        let Some(name) = self.album.clone() else {
            return Ok(None);
        };
        let meta = self.meta(ctx).await?;
        Ok(Some(Album {
            name,
            artist: self.artist.clone(),
            artwork_key: meta.as_ref().and_then(|m| m.album_image_key.clone()),
            release_year: meta.and_then(|m| m.release_year),
        }))
    }

    /// Album (or podcast show) artwork, once enrichment has stored it.
    async fn artwork_url(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<String>> {
        Ok(self
            .meta(ctx)
            .await?
            .and_then(|m| m.album_image_key)
            .map(|key| art_url(&key)))
    }

    /// Whether Spotify Web API metadata has been fetched for this item.
    async fn enriched(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        Ok(self.meta(ctx).await?.is_some())
    }

    async fn duration_ms(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<i32>> {
        Ok(self.meta(ctx).await?.and_then(|m| m.duration_ms))
    }

    async fn explicit(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<bool>> {
        Ok(self.meta(ctx).await?.and_then(|m| m.explicit))
    }

    /// `1999`, `1999-03` or `1999-03-14`, as precise as Spotify knows it.
    async fn release_date(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<String>> {
        Ok(self.meta(ctx).await?.and_then(|m| m.release_date))
    }

    async fn release_year(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<i32>> {
        Ok(self.meta(ctx).await?.and_then(|m| m.release_year))
    }

    /// Every credited artist in credit order; empty until enriched.
    async fn artists(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Artist>> {
        Ok(ctx
            .data::<DataLoader<ItemArtistsLoader>>()?
            .load_one(self.uri.clone())
            .await?
            .unwrap_or_default()
            .into_iter()
            .map(|name| Artist { name })
            .collect())
    }

    async fn stats(&self, #[graphql(default)] filter: StatsFilter) -> async_graphql::Result<Stats> {
        Ok(Stats::new(filter.into_scope()?.item(&self.uri)))
    }
}

#[derive(Clone)]
pub struct Artist {
    pub name: String,
}

impl Artist {
    /// Whether any play or item names this artist.
    pub async fn exists(pool: &PgPool, name: &str) -> sqlx::Result<bool> {
        sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM items WHERE artist = $1)
                   OR EXISTS (SELECT 1 FROM basic_plays WHERE artist = $1) AS "exists!""#,
            name
        )
        .fetch_one(pool)
        .await
    }
}

#[Object(name = "SpotifyArtist")]
impl Artist {
    pub async fn id(&self) -> ID {
        encode(NodeType::Artist, &self.name)
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn stats(&self, #[graphql(default)] filter: StatsFilter) -> async_graphql::Result<Stats> {
        Ok(Stats::new(filter.into_scope()?.artist(&self.name)))
    }

    /// Spotify's genres for this artist; empty until enriched.
    async fn genres(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<String>> {
        Ok(self.meta(ctx).await?.map(|m| m.genres).unwrap_or_default())
    }

    async fn image_url(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<String>> {
        Ok(self
            .meta(ctx)
            .await?
            .and_then(|m| m.image_key)
            .map(|key| art_url(&key)))
    }
}

impl Artist {
    async fn meta(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<ArtistMeta>> {
        Ok(ctx
            .data::<DataLoader<ArtistMetaLoader>>()?
            .load_one(self.name.clone())
            .await?)
    }
}

#[derive(Clone)]
pub struct Album {
    pub name: String,
    pub artist: Option<String>,
    pub artwork_key: Option<String>,
    pub release_year: Option<i32>,
}

#[Object(name = "SpotifyAlbum")]
impl Album {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn artist(&self) -> Option<Artist> {
        self.artist.clone().map(|name| Artist { name })
    }

    async fn artwork_url(&self) -> Option<String> {
        self.artwork_key.as_deref().map(art_url)
    }

    async fn release_year(&self) -> Option<i32> {
        self.release_year
    }
}

/// Catalog search over item names and artists, using the trigram indexes.
pub struct Search {
    pub query: String,
}

impl Search {
    /// `%query%` with LIKE metacharacters escaped.
    fn pattern(&self) -> String {
        let escaped = self
            .query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        format!("%{escaped}%")
    }
}

#[Object(name = "SpotifySearch")]
impl Search {
    async fn items(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
    ) -> async_graphql::Result<PlainConnection<Item>> {
        let pool = ctx.data::<PgPool>()?;
        let page = Page::new(first, after, 20)?;

        let items = sqlx::query_as!(
            Item,
            // Equally similar matches (e.g. every track by a searched artist) rank by plays.
            "SELECT i.uri, i.kind, i.name, i.artist, i.album
             FROM items i
             CROSS JOIN LATERAL (SELECT count(*) AS plays FROM plays p WHERE p.item_uri = i.uri) c
             WHERE i.name ILIKE $1 OR i.artist ILIKE $1
             ORDER BY greatest(similarity(i.name, $2), similarity(i.artist, $2)) DESC,
                      c.plays DESC, i.uri
             LIMIT $3 OFFSET $4",
            self.pattern(),
            self.query,
            page.fetch(),
            page.offset,
        )
        .fetch_all(pool)
        .await?;

        Ok(page.plain(items))
    }

    async fn artists(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
    ) -> async_graphql::Result<PlainConnection<Artist>> {
        let pool = ctx.data::<PgPool>()?;
        let page = Page::new(first, after, 20)?;

        let names = sqlx::query_scalar!(
            r#"SELECT artist AS "artist!"
               FROM items
               WHERE artist ILIKE $1
               GROUP BY artist
               ORDER BY similarity(artist, $2) DESC, artist
               LIMIT $3 OFFSET $4"#,
            self.pattern(),
            self.query,
            page.fetch(),
            page.offset,
        )
        .fetch_all(pool)
        .await?;

        Ok(page.plain(names.into_iter().map(|name| Artist { name })))
    }
}
