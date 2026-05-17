//! Ubicación: `crates/infrastructure/src/state.rs`
//!
//! Descripción: Estado de la aplicación con repositories compartidos.
//!
//! ADRs relacionados: 0003 (Axum)

use std::sync::Arc;
use monitoring::health::HealthRegistry;

pub struct AppState {
    pub sede_repository: Arc<dyn domain::ports::SedeRepository>,
    pub device_repository: Arc<dyn domain::ports::DeviceRepository>,
    pub alert_repository: Arc<dyn domain::ports::AlertRepository>,
    pub health_registry: Arc<HealthRegistry>,
}

impl AppState {
    pub fn new(
        sede_repository: Arc<dyn domain::ports::SedeRepository>,
        device_repository: Arc<dyn domain::ports::DeviceRepository>,
        alert_repository: Arc<dyn domain::ports::AlertRepository>,
        health_registry: Arc<HealthRegistry>,
    ) -> Self {
        Self {
            sede_repository,
            device_repository,
            alert_repository,
            health_registry,
        }
    }
}