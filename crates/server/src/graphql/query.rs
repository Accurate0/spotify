use async_graphql::{Context, ID, Object};
use sqlx::PgPool;

use super::node::{Node, NodeType, decode};
use super::objects::{Account, Export, ExportFile};

#[derive(Default)]
pub struct QueryRoot;

#[Object(name = "Query")]
impl QueryRoot {
    async fn node(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Node>> {
        load_node(ctx.data::<PgPool>()?, &id).await
    }

    async fn nodes(
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

    async fn accounts(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Account>> {
        let pool = ctx.data::<PgPool>()?;
        Ok(Account::all(pool).await?)
    }

    async fn account(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> async_graphql::Result<Option<Account>> {
        Ok(Account::by_slug(ctx.data::<PgPool>()?, &slug).await?)
    }

    /// Whether a file with this sha256 has already been imported for the account, so clients
    /// can skip uploading it.
    async fn file_exists(
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
    Ok(match ty {
        NodeType::Account => Account::by_id(pool, key).await?.map(Node::Account),
        NodeType::Export => Export::by_id(pool, key).await?.map(Node::Export),
        NodeType::ExportFile => ExportFile::by_id(pool, key).await?.map(Node::ExportFile),
    })
}
