//! Imports export files into Postgres. Each file is imported in a single transaction.

mod account;
mod extended;

use chrono::{DateTime, Utc};
use spotify_export::{FileKind, ParseError};
use sqlx::PgPool;

enum Parsed {
    Extended(Vec<spotify_export::ExtendedPlay>),
    BasicHistory(Vec<spotify_export::BasicPlay>),
    Searches(Vec<spotify_export::Search>),
    Library(Vec<spotify_export::LibraryItem>),
    Playlists(Vec<spotify_export::Playlist>),
    Follow(spotify_export::Follow),
    Marquee(Vec<spotify_export::Marquee>),
    Inferences(Vec<String>),
    Username(Option<String>),
}

impl Parsed {
    /// Callers check `FileKind::is_importable` first; private files are never parsed.
    fn parse(kind: FileKind, bytes: &[u8]) -> Result<Self, ParseError> {
        use spotify_export as e;

        Ok(match kind {
            FileKind::ExtendedAudio => Self::Extended(e::parse_extended(bytes, false)?),
            FileKind::ExtendedVideo => Self::Extended(e::parse_extended(bytes, true)?),
            FileKind::AccountStreamingHistory => Self::BasicHistory(e::parse_basic_history(bytes)?),
            FileKind::SearchQueries => Self::Searches(e::parse_searches(bytes)?),
            FileKind::Library => Self::Library(e::parse_library(bytes)?),
            FileKind::Playlist => Self::Playlists(e::parse_playlists(bytes)?),
            FileKind::Follow => Self::Follow(e::parse_follow(bytes)?),
            FileKind::Marquee => Self::Marquee(e::parse_marquee(bytes)?),
            FileKind::Inferences => Self::Inferences(e::parse_inferences(bytes)?),
            FileKind::Userdata => Self::Username(e::parse_username(bytes)?),
            FileKind::Private => unreachable!("private files are not imported"),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("sha256 mismatch: client sent {expected}, file hashes to {actual}")]
    Checksum { expected: String, actual: String },
    #[error("failed to parse {name}: {source}")]
    Parse {
        name: String,
        source: spotify_export::ParseError,
    },
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub struct FileUpload<'a> {
    pub account_id: i64,
    pub export_id: i64,
    pub as_of: DateTime<Utc>,
    pub name: &'a str,
    pub kind: FileKind,
    pub sha256: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportOutcome {
    Imported {
        file_id: i64,
    },
    /// A file with the same sha256 was already imported for this account.
    Duplicate {
        file_id: i64,
    },
    /// The kind is recognized but not imported (yet, or ever, for personal data).
    Unsupported,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RowCounts {
    pub total: i32,
    pub inserted: i32,
    pub merged: i32,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

pub async fn existing_file(
    pool: &PgPool,
    account_id: i64,
    sha256: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT id FROM export_files WHERE account_id = $1 AND sha256 = $2",
        account_id,
        sha256,
    )
    .fetch_optional(pool)
    .await
}

pub async fn import_file(
    pool: &PgPool,
    upload: FileUpload<'_>,
) -> Result<ImportOutcome, ImportError> {
    let actual = sha256_hex(upload.bytes);
    if !actual.eq_ignore_ascii_case(upload.sha256) {
        return Err(ImportError::Checksum {
            expected: upload.sha256.to_owned(),
            actual,
        });
    }

    if !upload.kind.is_importable() {
        return Ok(ImportOutcome::Unsupported);
    }

    if let Some(file_id) = existing_file(pool, upload.account_id, &actual).await? {
        return Ok(ImportOutcome::Duplicate { file_id });
    }

    let kind = upload.kind;
    let bytes = upload.bytes.to_vec();
    let parsed = tokio::task::spawn_blocking(move || Parsed::parse(kind, &bytes))
        .await
        .expect("parse task panicked")
        .map_err(|source| ImportError::Parse {
            name: upload.name.to_owned(),
            source,
        })?;

    let mut tx = pool.begin().await?;

    // A concurrent upload of the same file loses the race here instead of double-importing.
    let file_id = sqlx::query_scalar!(
        "INSERT INTO export_files (export_id, account_id, name, kind, sha256)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (account_id, sha256) DO NOTHING
         RETURNING id",
        upload.export_id,
        upload.account_id,
        upload.name,
        upload.kind.as_str(),
        actual,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(file_id) = file_id else {
        tx.rollback().await?;
        let file_id = existing_file(pool, upload.account_id, &actual)
            .await?
            .expect("conflicting file exists");
        return Ok(ImportOutcome::Duplicate { file_id });
    };

    let (account_id, as_of) = (upload.account_id, upload.as_of);
    let counts = match parsed {
        Parsed::Extended(plays) => {
            extended::merge(&mut tx, account_id, file_id, as_of, &plays).await?
        }
        Parsed::BasicHistory(plays) => {
            account::basic_history(&mut tx, account_id, file_id, &plays).await?
        }
        Parsed::Searches(searches) => {
            account::searches(&mut tx, account_id, file_id, &searches).await?
        }
        Parsed::Library(items) => account::library(&mut tx, account_id, file_id, &items).await?,
        Parsed::Playlists(playlists) => {
            account::playlists(&mut tx, account_id, file_id, &playlists).await?
        }
        Parsed::Follow(follow) => account::follow(&mut tx, account_id, file_id, follow).await?,
        Parsed::Marquee(rows) => account::marquee(&mut tx, account_id, file_id, &rows).await?,
        Parsed::Inferences(values) => {
            account::inferences(&mut tx, account_id, file_id, &values).await?
        }
        Parsed::Username(username) => {
            account::username(&mut tx, account_id, username.as_deref()).await?
        }
    };

    sqlx::query!(
        "UPDATE export_files SET rows_total = $2, rows_inserted = $3, rows_merged = $4 WHERE id = $1",
        file_id,
        counts.total,
        counts.inserted,
        counts.merged,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        name = upload.name,
        total = counts.total,
        inserted = counts.inserted,
        merged = counts.merged,
        "imported file"
    );

    Ok(ImportOutcome::Imported { file_id })
}
