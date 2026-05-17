//! Ubicación: `crates/infrastructure/src/routes/mod.rs`
//!
//! Descripción: Definición de rutas HTTP de la API usando Axum 0.8.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0020, 0016

use axum::{
    Router,
    routing::{get, post, put, delete},
};
use std::sync::Arc;

use crate::state::AppState;
use crate::handlers::sede_handler::{list_sedes, get_sede, create_sede, update_sede};
use crate::handlers::device_handler::{list_devices, get_device, create_device, update_device, delete_device};

pub fn create_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/api/v1/sedes", get(list_sedes).post(create_sede))
        .route("/api/v1/sedes/:id", get(get_sede).put(update_sede))
        .route("/api/v1/devices", get(list_devices).post(create_device))
        .route("/api/v1/devices/:id", get(get_device).put(update_device).delete(delete_device))
        .layer(crate::middleware::cors_layer())
        .layer(crate::middleware::tracing_layer())
        .with_state(Arc::new(AppState::new(
            Arc::new(MockSedeRepository),
            Arc::new(MockDeviceRepository),
        )))
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn ready_handler() -> &'static str {
    "READY"
}

struct MockSedeRepository;

impl domain::ports::SedeRepository for MockSedeRepository {
    fn find_all(&self) -> Result<Vec<domain::entities::Sede>, domain::errors::DomainError> {
        Ok(vec![])
    }

    fn find_by_id(&self, _id: uuid::Uuid) -> Result<Option<domain::entities::Sede>, domain::errors::DomainError> {
        Ok(None)
    }

    fn save(&self, _sede: &domain::entities::Sede) -> Result<domain::entities::Sede, domain::errors::DomainError> {
        unimplemented!()
    }
}

struct MockDeviceRepository;

impl domain::ports::DeviceRepository for MockDeviceRepository {
    fn find_by_id(&self, _id: uuid::Uuid) -> Result<Option<domain::entities::Device>, domain::errors::DomainError> {
        Ok(None)
    }

    fn find_by_sede(&self, _sede_id: uuid::Uuid) -> Result<Vec<domain::entities::Device>, domain::errors::DomainError> {
        Ok(vec![])
    }

    fn find_by_type(&self, _device_type: &str) -> Result<Vec<domain::entities::Device>, domain::errors::DomainError> {
        Ok(vec![])
    }

    fn save(&self, _device: &domain::entities::Device) -> Result<domain::entities::Device, domain::errors::DomainError> {
        unimplemented!()
    }

    fn update_status(&self, _id: uuid::Uuid, _status: domain::entities::DeviceStatus) -> Result<(), domain::errors::DomainError> {
        unimplemented!()
    }

    fn soft_delete(&self, _id: uuid::Uuid) -> Result<(), domain::errors::DomainError> {
        Ok(())
    }

    fn list(&self) -> Result<Vec<domain::entities::Device>, domain::errors::DomainError> {
        Ok(vec![])
    }
}