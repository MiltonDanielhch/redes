//! Ubicación: `crates/infrastructure/src/config/mod.rs`
//!
//! Descripción: Configuración tipada de la aplicación. Lee variables de entorno
//!              y archivos de configuración. Fail-fast si falta alguna variable required.
//!
//! ADRs relacionados: 0002 (Config), 0008 (PASETO)

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: SocketAddr,
    pub port: u16,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub paseto_secret: String,
    pub token_expiry_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub bucket: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub healthcheck_url: Option<String>,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, anyhow::Error> {
        let contents = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}