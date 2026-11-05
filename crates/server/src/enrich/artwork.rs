//! Artwork storage: a Garage (S3) bucket in production, memory in tests.

use std::collections::HashMap;
use std::sync::Mutex;

use s3::creds::Credentials;
use s3::{Bucket, Region};

pub enum ArtworkStore {
    S3(Box<Bucket>),
    Memory(Mutex<HashMap<String, Vec<u8>>>),
}

#[derive(Debug, thiserror::Error)]
pub enum ArtworkError {
    #[error("artwork bucket: {0}")]
    S3(#[from] s3::error::S3Error),
    #[error("artwork bucket credentials: {0}")]
    Credentials(#[from] s3::creds::error::CredentialsError),
    #[error("artwork bucket returned {0}")]
    Status(u16),
}

impl ArtworkStore {
    /// From the bucket secret's variables (S3_BUCKET, S3_ENDPOINT, S3_REGION,
    /// S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY); None when no bucket is configured.
    pub fn from_env() -> Result<Option<Self>, ArtworkError> {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        let (Some(bucket), Some(endpoint)) = (var("S3_BUCKET"), var("S3_ENDPOINT")) else {
            return Ok(None);
        };

        Ok(Some(Self::s3(
            &bucket,
            &endpoint,
            var("S3_REGION").as_deref().unwrap_or("garage"),
            var("S3_ACCESS_KEY_ID").as_deref(),
            var("S3_SECRET_ACCESS_KEY").as_deref(),
        )?))
    }

    /// A path-style bucket (as Garage needs) at `endpoint`.
    pub fn s3(
        bucket: &str,
        endpoint: &str,
        region: &str,
        access_key: Option<&str>,
        secret_key: Option<&str>,
    ) -> Result<Self, ArtworkError> {
        let region = Region::Custom {
            region: region.to_owned(),
            endpoint: endpoint.to_owned(),
        };
        let credentials = Credentials::new(access_key, secret_key, None, None, None)?;
        Ok(Self::S3(
            Bucket::new(bucket, region, credentials)?.with_path_style(),
        ))
    }

    pub fn memory() -> Self {
        Self::Memory(Mutex::new(HashMap::new()))
    }

    pub async fn put(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
    ) -> Result<(), ArtworkError> {
        match self {
            Self::S3(bucket) => {
                let response = bucket
                    .put_object_with_content_type(key, bytes, content_type)
                    .await?;
                match response.status_code() {
                    200..=299 => Ok(()),
                    status => Err(ArtworkError::Status(status)),
                }
            }
            Self::Memory(objects) => {
                objects
                    .lock()
                    .expect("artwork store poisoned")
                    .insert(key.to_owned(), bytes.to_vec());
                Ok(())
            }
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ArtworkError> {
        match self {
            Self::S3(bucket) => {
                let response = bucket.get_object(key).await?;
                match response.status_code() {
                    200..=299 => Ok(Some(response.to_vec())),
                    404 => Ok(None),
                    status => Err(ArtworkError::Status(status)),
                }
            }
            Self::Memory(objects) => Ok(objects
                .lock()
                .expect("artwork store poisoned")
                .get(key)
                .cloned()),
        }
    }
}
