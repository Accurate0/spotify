//! Shared fixture for import tests. Requires DATABASE_URL (docker compose up db); sqlx::test
//! creates and migrates a fresh database per test.

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use spotify_export::FileKind;
use spotify_server::import::{FileUpload, ImportOutcome, import_file, sha256_hex};
use sqlx::PgPool;

pub struct Fixture {
    pub pool: PgPool,
    pub account_id: i64,
}

impl Fixture {
    pub async fn new(pool: PgPool) -> Self {
        let account_id = sqlx::query_scalar!(
            "INSERT INTO accounts (slug, display_name) VALUES ('me', 'me') RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        Self { pool, account_id }
    }

    pub async fn export(&self, as_of: &str) -> (i64, DateTime<Utc>) {
        let as_of: DateTime<Utc> = as_of.parse().unwrap();
        let export_id = sqlx::query_scalar!(
            "INSERT INTO exports (account_id, label, as_of) VALUES ($1, 'test', $2) RETURNING id",
            self.account_id,
            as_of
        )
        .fetch_one(&self.pool)
        .await
        .unwrap();
        (export_id, as_of)
    }

    /// Imports one file as its own export.
    pub async fn import_file(&self, as_of: &str, name: &str, bytes: &[u8]) -> ImportOutcome {
        let (export_id, as_of) = self.export(as_of).await;
        self.import_into(export_id, as_of, name, bytes).await
    }

    pub async fn import_into(
        &self,
        export_id: i64,
        as_of: DateTime<Utc>,
        name: &str,
        bytes: &[u8],
    ) -> ImportOutcome {
        let sha256 = sha256_hex(bytes);
        import_file(
            &self.pool,
            FileUpload {
                account_id: self.account_id,
                export_id,
                as_of,
                name,
                kind: FileKind::classify(name).expect("test file name is classifiable"),
                sha256: &sha256,
                bytes,
            },
        )
        .await
        .unwrap()
    }

    /// Imports extended streaming history rows.
    pub async fn import(&self, as_of: &str, rows: &[serde_json::Value]) -> ImportOutcome {
        let bytes = serde_json::to_vec(rows).unwrap();
        self.import_file(as_of, "Streaming_History_Audio_2023_0.json", &bytes)
            .await
    }

    /// (total, inserted, merged) for an imported file.
    pub async fn counts(&self, outcome: ImportOutcome) -> (i32, i32, i32) {
        let ImportOutcome::Imported { file_id } = outcome else {
            panic!("expected Imported, got {outcome:?}");
        };
        let row = sqlx::query!(
            "SELECT rows_total, rows_inserted, rows_merged FROM export_files WHERE id = $1",
            file_id
        )
        .fetch_one(&self.pool)
        .await
        .unwrap();
        (row.rows_total, row.rows_inserted, row.rows_merged)
    }
}
