//! Typed operations against the server's schema.graphql, checked at compile time by cynic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cynic::schema("spotify")]
pub mod schema {}

cynic::impl_scalar!(DateTime<Utc>, schema::DateTime);
cynic::impl_scalar!(chrono::NaiveDate, schema::NaiveDate);

/// Placeholder for a multipart file. Per the GraphQL multipart request spec the variable is
/// sent as `null` and the file is attached separately, mapped onto `variables.file`.
pub struct Upload;

impl Serialize for Upload {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

impl<'de> Deserialize<'de> for Upload {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom("Upload is input only"))
    }
}

cynic::impl_scalar!(Upload, schema::Upload);

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyAccount")]
pub struct Account {
    pub id: cynic::Id,
    pub slug: String,
    pub display_name: String,
    pub spotify_username: Option<String>,
    pub play_count: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct AccountsQuery {
    pub spotify_accounts: Vec<Account>,
}

#[derive(cynic::QueryVariables)]
pub struct AccountVariables {
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "AccountVariables")]
pub struct AccountQuery {
    #[arguments(slug: $slug)]
    pub spotify_account: Option<Account>,
}

#[derive(cynic::QueryVariables)]
pub struct FileExistsVariables {
    pub account: String,
    pub sha256: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FileExistsVariables")]
pub struct FileExistsQuery {
    #[arguments(account: $account, sha256: $sha256)]
    pub spotify_file_exists: bool,
}

#[derive(cynic::InputObject)]
#[cynic(graphql_type = "SpotifyCreateAccountInput")]
pub struct CreateAccountInput {
    pub slug: String,
    pub display_name: Option<String>,
}

#[derive(cynic::QueryVariables)]
pub struct CreateAccountVariables {
    pub input: CreateAccountInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyCreateAccountPayload")]
pub struct CreateAccountPayload {
    pub account: Account,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "CreateAccountVariables")]
pub struct CreateAccount {
    #[arguments(input: $input)]
    pub spotify_create_account: CreateAccountPayload,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyExport")]
pub struct Export {
    pub id: cynic::Id,
}

#[derive(cynic::InputObject)]
#[cynic(graphql_type = "SpotifyCreateExportInput")]
pub struct CreateExportInput {
    pub account: String,
    pub label: String,
    pub as_of: DateTime<Utc>,
}

#[derive(cynic::QueryVariables)]
pub struct CreateExportVariables {
    pub input: CreateExportInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyCreateExportPayload")]
pub struct CreateExportPayload {
    pub export: Export,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "CreateExportVariables")]
pub struct CreateExport {
    #[arguments(input: $input)]
    pub spotify_create_export: CreateExportPayload,
}

#[derive(cynic::Enum, Clone, Copy, Debug, PartialEq, Eq)]
#[cynic(graphql_type = "SpotifyUploadStatus")]
pub enum UploadStatus {
    Imported,
    Duplicate,
    Unsupported,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyExportFile")]
pub struct ExportFile {
    pub rows_total: i32,
    pub rows_inserted: i32,
    pub rows_merged: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyUploadExportFilePayload")]
pub struct UploadExportFilePayload {
    pub status: UploadStatus,
    pub file: Option<ExportFile>,
}

#[derive(cynic::InputObject)]
#[cynic(graphql_type = "SpotifyUploadExportFileInput")]
pub struct UploadExportFileInput {
    pub export_id: cynic::Id,
    pub name: String,
    pub sha256: String,
}

#[derive(cynic::QueryVariables)]
pub struct UploadExportFileVariables {
    pub input: UploadExportFileInput,
    pub file: Upload,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "UploadExportFileVariables")]
pub struct UploadExportFile {
    #[arguments(input: $input, file: $file)]
    pub spotify_upload_export_file: UploadExportFilePayload,
}

#[derive(cynic::InputObject)]
#[cynic(graphql_type = "SpotifyFinalizeExportInput")]
pub struct FinalizeExportInput {
    pub export_id: cynic::Id,
}

#[derive(cynic::QueryVariables)]
pub struct FinalizeExportVariables {
    pub input: FinalizeExportInput,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "SpotifyFinalizeExportPayload")]
pub struct FinalizeExportPayload {
    pub export: Export,
}

// The result is unused but GraphQL requires a selection.
#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "FinalizeExportVariables")]
pub struct FinalizeExport {
    #[arguments(input: $input)]
    pub spotify_finalize_export: FinalizeExportPayload,
}
