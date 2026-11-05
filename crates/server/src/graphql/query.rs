use async_graphql::{Context, ID, Object};
use sqlx::PgPool;

use super::catalog::{Artist, Item, Search};
use super::insights::{self, Enrichment};
use super::node::{Node, NodeType, decode};
use super::objects::{Account, Export, ExportFile};
use super::stats::{Stats, StatsFilter};

#[derive(Default)]
pub struct QueryRoot;

#[Object(name = "Query")]
impl QueryRoot {
    pub(super) async fn node(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Option<Node>> {
        load_node(ctx.data::<PgPool>()?, &id).await
    }

    pub(super) async fn nodes(
        &self,
        ctx: &Context<'_>,
        ids: Vec<ID>,
    ) -> async_graphql::Result<Vec<Option<Node>>> {
        let pool = ctx.data::<PgPool>()?;
        let mut nodes = Vec::with_capacity(ids.len());
        for id in &ids {
            nodes.push(load_node(pool, id).await?);
        }
        Ok(nodes)
    }

    pub(super) async fn accounts(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Account>> {
        let pool = ctx.data::<PgPool>()?;
        Ok(Account::all(pool).await?)
    }

    pub(super) async fn account(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> async_graphql::Result<Option<Account>> {
        Ok(Account::by_slug(ctx.data::<PgPool>()?, &slug).await?)
    }

    /// Listening stats across the filtered plays.
    pub(super) async fn stats(
        &self,
        #[graphql(default)] filter: StatsFilter,
    ) -> async_graphql::Result<Stats> {
        Ok(Stats::new(filter.into_scope()?))
    }

    /// Search tracks, episodes and artists by name.
    pub(super) async fn search(&self, query: String) -> async_graphql::Result<Search> {
        let query = query.trim().to_owned();
        if query.is_empty() {
            return Err("search query is empty".into());
        }
        Ok(Search { query })
    }

    pub(super) async fn item(
        &self,
        ctx: &Context<'_>,
        uri: String,
    ) -> async_graphql::Result<Option<Item>> {
        Ok(Item::by_uri(ctx.data::<PgPool>()?, &uri).await?)
    }

    pub(super) async fn artist(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> async_graphql::Result<Option<Artist>> {
        Ok(Artist::exists(ctx.data::<PgPool>()?, &name)
            .await?
            .then_some(Artist { name }))
    }

    /// Federation lookup for an `Item` another subgraph references by uri, e.g. home-gateway's
    /// media players.
    #[graphql(entity)]
    pub(super) async fn find_item_by_uri(
        &self,
        ctx: &Context<'_>,
        uri: String,
    ) -> async_graphql::Result<Option<Item>> {
        Ok(Item::by_uri(ctx.data::<PgPool>()?, &uri).await?)
    }

    /// Federation lookup for an `Artist` referenced by name; null when it has no plays.
    #[graphql(entity)]
    pub(super) async fn find_artist_by_name(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> async_graphql::Result<Option<Artist>> {
        Ok(Artist::exists(ctx.data::<PgPool>()?, &name)
            .await?
            .then_some(Artist { name }))
    }

    /// Progress of Spotify Web API enrichment (metadata, genres, artwork).
    pub(super) async fn enrichment(&self, ctx: &Context<'_>) -> async_graphql::Result<Enrichment> {
        Ok(insights::enrichment(ctx.data::<PgPool>()?).await?)
    }

    /// Whether a file with this sha256 has already been imported for the account, so clients
    /// can skip uploading it.
    #[graphql(inaccessible)]
    pub(super) async fn file_exists(
        &self,
        ctx: &Context<'_>,
        account: String,
        sha256: String,
    ) -> async_graphql::Result<bool> {
        let pool = ctx.data::<PgPool>()?;
        let account = Account::by_slug(pool, &account)
            .await?
            .ok_or("account not found")?;
        Ok(
            crate::import::existing_file(pool, account.id, &sha256.to_ascii_lowercase())
                .await?
                .is_some(),
        )
    }
}

async fn load_node(pool: &PgPool, id: &ID) -> async_graphql::Result<Option<Node>> {
    let (ty, key) = decode(id)?;
    let row_id = || {
        key.parse::<i64>()
            .map_err(|_| async_graphql::Error::new(format!("invalid id {:?}", id.as_str())))
    };

    Ok(match ty {
        NodeType::Account => Account::by_id(pool, row_id()?).await?.map(Node::Account),
        NodeType::Export => Export::by_id(pool, row_id()?).await?.map(Node::Export),
        NodeType::ExportFile => ExportFile::by_id(pool, row_id()?)
            .await?
            .map(Node::ExportFile),
        NodeType::Item => Item::by_uri(pool, &key).await?.map(Node::Item),
        NodeType::Artist => Artist::exists(pool, &key)
            .await?
            .then_some(Node::Artist(Artist { name: key })),
    })
}
