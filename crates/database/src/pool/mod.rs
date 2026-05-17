//! Ubicación: `crates/database/src/pool/mod.rs`
//!
//! Descripción: Módulo de configuración del pool de conexiones PostgreSQL.
//!
//! ADRs relacionados: 0004 (PostgreSQL)

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}