//! Ubicación: `crates/monitoring/src/health/mod.rs`
//!
//! Descripción: Health checker para dispositivos de red.
//!
//! ADRs relacionados: 0014 (Healthchecks)

pub struct HealthChecker;

impl HealthChecker {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_device(&self, ip: &str) -> bool {
        true
    }

    pub async fn ping(&self, host: &str) -> Result<f64, std::net::IpAddr> {
        Ok(0.0)
    }
}