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
