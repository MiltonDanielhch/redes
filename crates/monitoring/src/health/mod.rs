//! Ubicación: `crates/monitoring/src/health/mod.rs`
//!
//! Descripción: Health checker para componentes del sistema.
//!              Implementa /health/live y /health/ready.
//!
//! ADRs relacionados: 0014 (Healthchecks)

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub component: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemHealth {
    pub overall_healthy: bool,
    pub components: Vec<HealthStatus>,
    pub timestamp: time::OffsetDateTime,
}

#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthStatus;
}

pub struct HealthRegistry {
    checks: Arc<RwLock<Vec<Arc<dyn HealthCheck>>>>,
}

impl HealthRegistry {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register(&self, check: Arc<dyn HealthCheck>) {
        let mut checks = self.checks.write().await;
        checks.push(check);
    }

    pub async fn check_all(&self) -> SystemHealth {
        let checks = self.checks.read().await;
        let mut components = Vec::new();
        let mut all_healthy = true;

        for check in checks.iter() {
            let status = check.check().await;
            if !status.healthy {
                all_healthy = false;
            }
            components.push(status);
        }

        SystemHealth {
            overall_healthy: all_healthy,
            components,
            timestamp: time::OffsetDateTime::now_utc(),
        }
    }
}

impl Default for HealthRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DatabaseHealthCheck {
    pool: sqlx::PgPool,
}

impl DatabaseHealthCheck {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        "database"
    }

    async fn check(&self) -> HealthStatus {
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => HealthStatus {
                healthy: true,
                component: "database".to_string(),
                message: Some("Connected".to_string()),
            },
            Err(e) => HealthStatus {
                healthy: false,
                component: "database".to_string(),
                message: Some(e.to_string()),
            },
        }
    }
}

pub struct HttpHealthCheck {
    name: String,
    url: String,
}

impl HttpHealthCheck {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
        }
    }
}

#[async_trait]
impl HealthCheck for HttpHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthStatus {
        match reqwest::get(&self.url).await {
            Ok(response) if response.status().is_success() => HealthStatus {
                healthy: true,
                component: self.name.clone(),
                message: Some(format!("HTTP {}", response.status())),
            },
            Ok(response) => HealthStatus {
                healthy: false,
                component: self.name.clone(),
                message: Some(format!("HTTP {}", response.status())),
            },
            Err(e) => HealthStatus {
                healthy: false,
                component: self.name.clone(),
                message: Some(e.to_string()),
            },
        }
    }
}

pub struct HealthChecker;

impl HealthChecker {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_device(&self, _ip: &str) -> bool {
        true
    }

    pub async fn ping(&self, _host: &str) -> Result<f64, std::net::IpAddr> {
        Ok(0.0)
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}
