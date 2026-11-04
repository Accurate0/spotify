//! Items (tracks, episodes, audiobook chapters), artists and albums. Exports carry no artist or
//! album URIs, so artists and albums are identified by name.

use async_graphql::{Context, Enum, ID, Object};
use sqlx::PgPool;

use super::node::{NodeType, encode};
use super::page::{Page, PlainConnection};
use super::stats::{Stats, StatsFilter};

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
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

impl Item {
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

#[Object]
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

    async fn album(&self) -> Option<Album> {
        self.album.clone().map(|name| Album {
            name,
            artist: self.artist.clone(),
        })
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

#[Object]
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
}

#[derive(Clone)]
pub struct Album {
    pub name: String,
    pub artist: Option<String>,
}

#[Object]
impl Album {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn artist(&self) -> Option<Artist> {
        self.artist.clone().map(|name| Artist { name })
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

#[Object]
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
