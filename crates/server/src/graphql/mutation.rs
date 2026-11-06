use std::io::Read;

use async_graphql::{Context, Enum, ID, InputObject, Object, SimpleObject, Upload};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::node::{NodeType, decode_row_id};
use super::objects::{Account, Export, ExportFile, SpotifyLink};
use crate::enrich::spotify::SpotifyConfig;
use crate::import::{self, FileUpload, ImportOutcome};
use crate::recent;

#[derive(Default)]
pub struct MutationRoot;

#[derive(InputObject)]
#[graphql(name = "SpotifyCreateAccountInput")]
pub struct CreateAccountInput {
    /// Lowercase identifier used by the CLI, e.g. `me`.
    pub slug: String,
    pub display_name: Option<String>,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyCreateAccountPayload")]
pub struct CreateAccountPayload {
    pub account: Account,
}

#[derive(InputObject)]
#[graphql(name = "SpotifyCreateExportInput")]
pub struct CreateExportInput {
    /// Account slug.
    pub account: String,
    /// Where the export came from, e.g. its directory name.
    pub label: String,
    /// Newest timestamp in the export's files. When exports overlap, the newer one's values win.
    pub as_of: DateTime<Utc>,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyCreateExportPayload")]
pub struct CreateExportPayload {
    pub export: Export,
}

#[derive(InputObject)]
#[graphql(name = "SpotifyUploadExportFileInput")]
pub struct UploadExportFileInput {
    pub export_id: ID,
    /// The file's name inside the export, used to detect its kind.
    pub name: String,
    /// Hex sha256 of the file, verified by the server.
    pub sha256: String,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
#[graphql(name = "SpotifyUploadStatus")]
pub enum UploadStatus {
    Imported,
    /// Already imported for this account; nothing was changed.
    Duplicate,
    /// The file kind is not imported.
    Unsupported,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyUploadExportFilePayload")]
pub struct UploadExportFilePayload {
    pub status: UploadStatus,
    pub file: Option<ExportFile>,
}

#[derive(InputObject)]
#[graphql(name = "SpotifyFinalizeExportInput")]
pub struct FinalizeExportInput {
    pub export_id: ID,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyFinalizeExportPayload")]
pub struct FinalizeExportPayload {
    pub export: Export,
}

#[derive(InputObject)]
#[graphql(name = "SpotifyLinkAccountInput")]
pub struct LinkAccountInput {
    /// Account slug.
    pub account: String,
    /// The authorization code from Spotify's redirect, for `user-read-recently-played`.
    pub code: String,
    /// The redirect URI the code was issued for.
    pub redirect_uri: String,
}

#[derive(SimpleObject)]
#[graphql(name = "SpotifyLinkAccountPayload")]
pub struct LinkAccountPayload {
    pub account: Account,
    pub link: SpotifyLink,
}

#[Object(name = "Mutation")]
impl MutationRoot {
    async fn spotify_create_account(
        &self,
        ctx: &Context<'_>,
        input: CreateAccountInput,
    ) -> async_graphql::Result<CreateAccountPayload> {
        let pool = ctx.data::<PgPool>()?;
        let display_name = input.display_name.unwrap_or_else(|| input.slug.clone());

        let account = Account::create(pool, &input.slug, &display_name)
            .await
            .map_err(
                |e| match e.as_database_error().and_then(|d| d.constraint()) {
                    Some("accounts_slug_key") => {
                        format!("account {} already exists", input.slug).into()
                    }
                    Some("accounts_slug_check") => {
                        "slug must be lowercase letters, digits and dashes".into()
                    }
                    _ => async_graphql::Error::from(e),
                },
            )?;

        Ok(CreateAccountPayload { account })
    }

    #[graphql(inaccessible)]
    async fn spotify_create_export(
        &self,
        ctx: &Context<'_>,
        input: CreateExportInput,
    ) -> async_graphql::Result<CreateExportPayload> {
        let pool = ctx.data::<PgPool>()?;
        let account = Account::by_slug(pool, &input.account)
            .await?
            .ok_or("account not found")?;

        let export = Export::create(pool, account.id, &input.label, input.as_of).await?;

        Ok(CreateExportPayload { export })
    }

    #[graphql(inaccessible)]
    async fn spotify_upload_export_file(
        &self,
        ctx: &Context<'_>,
        input: UploadExportFileInput,
        file: Upload,
    ) -> async_graphql::Result<UploadExportFilePayload> {
        let pool = ctx.data::<PgPool>()?;
        let export_id = decode_row_id(&input.export_id, NodeType::Export)?;
        let export = Export::by_id(pool, export_id)
            .await?
            .ok_or("export not found")?;
        if export.finalized_at.is_some() {
            return Err("export is already finalized".into());
        }

        let Some(kind) = spotify_export::FileKind::classify(&input.name) else {
            return Ok(UploadExportFilePayload {
                status: UploadStatus::Unsupported,
                file: None,
            });
        };

        let content = file.value(ctx)?.content;
        let bytes = tokio::task::spawn_blocking(move || {
            let mut bytes = Vec::new();
            std::io::BufReader::new(content).read_to_end(&mut bytes)?;
            Ok::<_, std::io::Error>(bytes)
        })
        .await??;

        let outcome = import::import_file(
            pool,
            FileUpload {
                account_id: export.account_id,
                export_id: export.id,
                as_of: export.as_of,
                name: &input.name,
                kind,
                sha256: &input.sha256,
                bytes: &bytes,
            },
        )
        .await?;

        let (status, file_id) = match outcome {
            ImportOutcome::Imported { file_id } => (UploadStatus::Imported, Some(file_id)),
            ImportOutcome::Duplicate { file_id } => (UploadStatus::Duplicate, Some(file_id)),
            ImportOutcome::Unsupported => (UploadStatus::Unsupported, None),
        };

        let file = match file_id {
            Some(id) => ExportFile::by_id(pool, id).await?,
            None => None,
        };

        Ok(UploadExportFilePayload { status, file })
    }

    #[graphql(inaccessible)]
    async fn spotify_finalize_export(
        &self,
        ctx: &Context<'_>,
        input: FinalizeExportInput,
    ) -> async_graphql::Result<FinalizeExportPayload> {
        let pool = ctx.data::<PgPool>()?;
        let export_id = decode_row_id(&input.export_id, NodeType::Export)?;

        let export = Export::finalize(pool, export_id)
            .await?
            .ok_or("export not found")?;

        Ok(FinalizeExportPayload { export })
    }

    /// Links an account to the Spotify user who authorized `code`, so their recently played
    /// tracks are fetched until the next export covers them.
    #[graphql(inaccessible)]
    async fn spotify_link_account(
        &self,
        ctx: &Context<'_>,
        input: LinkAccountInput,
    ) -> async_graphql::Result<LinkAccountPayload> {
        let pool = ctx.data::<PgPool>()?;
        let config = ctx
            .data::<Option<SpotifyConfig>>()?
            .clone()
            .ok_or("spotify credentials aren't configured")?;
        let account = Account::by_slug(pool, &input.account)
            .await?
            .ok_or("account not found")?;

        let link = recent::link(pool, config, account.id, &input.code, &input.redirect_uri)
            .await
            .map_err(|e| match e {
                recent::LinkError::Database(e) => async_graphql::Error::from(e),
                e => e.to_string().into(),
            })?;

        Ok(LinkAccountPayload {
            account,
            link: SpotifyLink(link),
        })
    }
}
