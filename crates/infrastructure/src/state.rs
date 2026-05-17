//! Ubicación: `crates/infrastructure/src/state.rs`
//!
//! Descripción: Estado de la aplicación con repositories compartidos.
//!
//! ADRs relacionados: 0003 (Axum)

use std::sync::Arc;

pub struct AppState {
    pub sede_repository: Arc<dyn domain::ports::SedeRepository>,
    pub device_repository: Arc<dyn domain::ports::DeviceRepository>,
}

impl AppState {
    pub fn new(
        sede_repository: Arc<dyn domain::ports::SedeRepository>,
        device_repository: Arc<dyn domain::ports::DeviceRepository>,
    ) -> Self {
        Self {
            sede_repository,
            device_repository,
        }
    }
}