//! Imports export files into Postgres. Each file is imported in a single transaction.

mod extended;

use chrono::{DateTime, Utc};
use spotify_export::FileKind;
use sqlx::PgPool;

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
    sqlx::query_scalar("SELECT id FROM export_files WHERE account_id = $1 AND sha256 = $2")
        .bind(account_id)
        .bind(sha256)
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

    let is_video = upload.kind == FileKind::ExtendedVideo;
    let bytes = upload.bytes.to_vec();
    let plays =
        tokio::task::spawn_blocking(move || spotify_export::parse_extended(&bytes, is_video))
            .await
            .expect("parse task panicked")
            .map_err(|source| ImportError::Parse {
                name: upload.name.to_owned(),
                source,
            })?;

    let mut tx = pool.begin().await?;

    // A concurrent upload of the same file loses the race here instead of double-importing.
    let file_id: Option<i64> = sqlx::query_scalar(
        "INSERT INTO export_files (export_id, account_id, name, kind, sha256)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (account_id, sha256) DO NOTHING
         RETURNING id",
    )
    .bind(upload.export_id)
    .bind(upload.account_id)
    .bind(upload.name)
    .bind(upload.kind.as_str())
    .bind(&actual)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(file_id) = file_id else {
        tx.rollback().await?;
        let file_id = existing_file(pool, upload.account_id, &actual)
            .await?
            .expect("conflicting file exists");
        return Ok(ImportOutcome::Duplicate { file_id });
    };

    let counts = extended::merge(&mut tx, upload.account_id, file_id, upload.as_of, &plays).await?;

    sqlx::query(
        "UPDATE export_files SET rows_total = $2, rows_inserted = $3, rows_merged = $4 WHERE id = $1",
    )
    .bind(file_id)
    .bind(counts.total)
    .bind(counts.inserted)
    .bind(counts.merged)
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
