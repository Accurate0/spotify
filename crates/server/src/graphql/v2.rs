use async_graphql::{Context, ID, Object, Upload};

use super::catalog::{Artist, Item, Search};
use super::insights::Enrichment;
use super::mutation::{
    CreateAccountInput, CreateAccountPayload, CreateExportInput, CreateExportPayload,
    FinalizeExportInput, FinalizeExportPayload, UploadExportFileInput, UploadExportFilePayload,
};
use super::node::Node;
use super::objects::Account;
use super::stats::{Stats, StatsFilter};
use super::{MutationRoot, QueryRoot};

#[derive(Default)]
pub struct QueryRootV2;

#[Object(name = "Query")]
impl QueryRootV2 {
    async fn spotify_node(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Node>> {
        QueryRoot.node(ctx, id).await
    }

    async fn spotify_nodes(
        &self,
        ctx: &Context<'_>,
        ids: Vec<ID>,
    ) -> async_graphql::Result<Vec<Option<Node>>> {
        QueryRoot.nodes(ctx, ids).await
    }

    async fn spotify_accounts(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Account>> {
        QueryRoot.accounts(ctx).await
    }

    async fn spotify_account(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> async_graphql::Result<Option<Account>> {
        QueryRoot.account(ctx, slug).await
    }

    /// Listening stats across the filtered plays.
    async fn spotify_stats(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] filter: StatsFilter,
    ) -> async_graphql::Result<Stats> {
        QueryRoot.stats(ctx, filter).await
    }

    /// Search tracks, episodes and artists by name.
    async fn spotify_search(
        &self,
        ctx: &Context<'_>,
        query: String,
    ) -> async_graphql::Result<Search> {
        QueryRoot.search(ctx, query).await
    }

    async fn spotify_item(
        &self,
        ctx: &Context<'_>,
        uri: String,
    ) -> async_graphql::Result<Option<Item>> {
        QueryRoot.item(ctx, uri).await
    }

    async fn spotify_artist(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> async_graphql::Result<Option<Artist>> {
        QueryRoot.artist(ctx, name).await
    }

    #[graphql(entity)]
    async fn find_item_by_uri(
        &self,
        ctx: &Context<'_>,
        uri: String,
    ) -> async_graphql::Result<Option<Item>> {
        QueryRoot.find_item_by_uri(ctx, uri).await
    }

    #[graphql(entity)]
    async fn find_artist_by_name(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> async_graphql::Result<Option<Artist>> {
        QueryRoot.find_artist_by_name(ctx, name).await
    }

    /// Progress of Spotify Web API enrichment (metadata, genres, artwork).
    async fn spotify_enrichment(&self, ctx: &Context<'_>) -> async_graphql::Result<Enrichment> {
        QueryRoot.enrichment(ctx).await
    }

    /// Whether a file with this sha256 has already been imported for the account, so clients
    /// can skip uploading it.
    #[graphql(inaccessible)]
    async fn spotify_file_exists(
        &self,
        ctx: &Context<'_>,
        account: String,
        sha256: String,
    ) -> async_graphql::Result<bool> {
        QueryRoot.file_exists(ctx, account, sha256).await
    }
}

#[derive(Default)]
pub struct MutationRootV2;

#[Object(name = "Mutation")]
impl MutationRootV2 {
    async fn spotify_create_account(
        &self,
        ctx: &Context<'_>,
        input: CreateAccountInput,
    ) -> async_graphql::Result<CreateAccountPayload> {
        MutationRoot.create_account(ctx, input).await
    }

    #[graphql(inaccessible)]
    async fn spotify_create_export(
        &self,
        ctx: &Context<'_>,
        input: CreateExportInput,
    ) -> async_graphql::Result<CreateExportPayload> {
        MutationRoot.create_export(ctx, input).await
    }

    #[graphql(inaccessible)]
    async fn spotify_upload_export_file(
        &self,
        ctx: &Context<'_>,
        input: UploadExportFileInput,
        file: Upload,
    ) -> async_graphql::Result<UploadExportFilePayload> {
        MutationRoot.upload_export_file(ctx, input, file).await
    }

    #[graphql(inaccessible)]
    async fn spotify_finalize_export(
        &self,
        ctx: &Context<'_>,
        input: FinalizeExportInput,
    ) -> async_graphql::Result<FinalizeExportPayload> {
        MutationRoot.finalize_export(ctx, input).await
    }
}
