use async_graphql::{Context, Enum, ID, Object};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::node::{NodeType, encode};

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(remote = "spotify_export::FileKind")]
pub enum FileKind {
    ExtendedAudio,
    ExtendedVideo,
    AccountStreamingHistory,
    Library,
    Playlist,
    SearchQueries,
    Follow,
    Marquee,
    Inferences,
    Userdata,
    Private,
}

#[derive(Clone)]
pub struct Account {
    pub id: i64,
    pub slug: String,
    pub display_name: String,
    pub spotify_username: Option<String>,
}

impl Account {
    pub async fn by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, slug, display_name, spotify_username FROM accounts WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn by_slug(pool: &PgPool, slug: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, slug, display_name, spotify_username FROM accounts WHERE slug = $1",
            slug
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn all(pool: &PgPool) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, slug, display_name, spotify_username FROM accounts ORDER BY slug"
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &PgPool, slug: &str, display_name: &str) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            "INSERT INTO accounts (slug, display_name) VALUES ($1, $2)
             RETURNING id, slug, display_name, spotify_username",
            slug,
            display_name
        )
        .fetch_one(pool)
        .await
    }
}

#[Object]
impl Account {
    pub async fn id(&self) -> ID {
        encode(NodeType::Account, self.id)
    }

    async fn slug(&self) -> &str {
        &self.slug
    }

    async fn display_name(&self) -> &str {
        &self.display_name
    }

    async fn spotify_username(&self) -> Option<&str> {
        self.spotify_username.as_deref()
    }

    async fn play_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        let pool = ctx.data::<PgPool>()?;
        Ok(sqlx::query_scalar!(
            r#"SELECT count(*) AS "count!" FROM plays WHERE account_id = $1"#,
            self.id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn exports(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Export>> {
        let pool = ctx.data::<PgPool>()?;
        Ok(sqlx::query_as!(
            Export,
            "SELECT id, account_id, label, as_of, created_at, finalized_at
             FROM exports WHERE account_id = $1 ORDER BY as_of DESC",
            self.id
        )
        .fetch_all(pool)
        .await?)
    }
}

#[derive(Clone)]
pub struct Export {
    pub id: i64,
    pub account_id: i64,
    pub label: String,
    pub as_of: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub finalized_at: Option<DateTime<Utc>>,
}

impl Export {
    pub async fn by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, account_id, label, as_of, created_at, finalized_at
             FROM exports WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        account_id: i64,
        label: &str,
        as_of: DateTime<Utc>,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            "INSERT INTO exports (account_id, label, as_of) VALUES ($1, $2, $3)
             RETURNING id, account_id, label, as_of, created_at, finalized_at",
            account_id,
            label,
            as_of
        )
        .fetch_one(pool)
        .await
    }

    pub async fn finalize(pool: &PgPool, id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE exports SET finalized_at = coalesce(finalized_at, now()) WHERE id = $1
             RETURNING id, account_id, label, as_of, created_at, finalized_at",
            id
        )
        .fetch_optional(pool)
        .await
    }
}

#[Object]
impl Export {
    pub async fn id(&self) -> ID {
        encode(NodeType::Export, self.id)
    }

    async fn label(&self) -> &str {
        &self.label
    }

    async fn as_of(&self) -> DateTime<Utc> {
        self.as_of
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    async fn finalized_at(&self) -> Option<DateTime<Utc>> {
        self.finalized_at
    }

    async fn account(&self, ctx: &Context<'_>) -> async_graphql::Result<Account> {
        Account::by_id(ctx.data::<PgPool>()?, self.account_id)
            .await?
            .ok_or_else(|| "account not found".into())
    }

    async fn files(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<ExportFile>> {
        let pool = ctx.data::<PgPool>()?;
        Ok(sqlx::query_as!(
            ExportFile,
            "SELECT id, export_id, name, kind, sha256, rows_total, rows_inserted, rows_merged,
                    imported_at
             FROM export_files WHERE export_id = $1 ORDER BY name",
            self.id
        )
        .fetch_all(pool)
        .await?)
    }
}

#[derive(Clone)]
pub struct ExportFile {
    pub id: i64,
    pub export_id: i64,
    pub name: String,
    pub kind: String,
    pub sha256: String,
    pub rows_total: i32,
    pub rows_inserted: i32,
    pub rows_merged: i32,
    pub imported_at: DateTime<Utc>,
}

impl ExportFile {
    pub async fn by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, export_id, name, kind, sha256, rows_total, rows_inserted, rows_merged,
                    imported_at
             FROM export_files WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
    }
}

#[Object]
impl ExportFile {
    pub async fn id(&self) -> ID {
        encode(NodeType::ExportFile, self.id)
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn kind(&self) -> async_graphql::Result<FileKind> {
        spotify_export::FileKind::parse(&self.kind)
            .map(Into::into)
            .ok_or_else(|| format!("unknown file kind {}", self.kind).into())
    }

    async fn sha256(&self) -> &str {
        &self.sha256
    }

    async fn rows_total(&self) -> i32 {
        self.rows_total
    }

    async fn rows_inserted(&self) -> i32 {
        self.rows_inserted
    }

    async fn rows_merged(&self) -> i32 {
        self.rows_merged
    }

    async fn imported_at(&self) -> DateTime<Utc> {
        self.imported_at
    }

    async fn export(&self, ctx: &Context<'_>) -> async_graphql::Result<Export> {
        Export::by_id(ctx.data::<PgPool>()?, self.export_id)
            .await?
            .ok_or_else(|| "export not found".into())
    }
}
