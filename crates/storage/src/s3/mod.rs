//! Ubicación: `crates/storage/src/s3/mod.rs`
//!
//! Descripción: Storage S3 para assets y archivos.
//!
//! ADRs relacionados: 0020 (Monitoreo Regional)

use aws_sdk_s3::Client;

pub struct S3Storage {
    _client: Client,
    _bucket: String,
}

impl S3Storage {
    pub fn new(client: Client, bucket: String) -> Self {
        Self { _client: client, _bucket: bucket }
    }

    pub async fn upload(&self, _key: &str, _data: &[u8]) -> Result<(), aws_sdk_s3::Error> {
        Ok(())
    }

    pub async fn download(&self, _key: &str) -> Result<Vec<u8>, aws_sdk_s3::Error> {
        Ok(vec![])
    }

    pub async fn delete(&self, _key: &str) -> Result<(), aws_sdk_s3::Error> {
        Ok(())
    }
}