//! Ubicación: `crates/infrastructure/src/routes/mod.rs`
//!
//! Descripción: Definición de rutas HTTP de la API usando Axum 0.8.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0020, 0016

use axum::{
    Router,
    extract::State,
    routing::{get, put},
    response::IntoResponse,
};
use std::sync::Arc;

use crate::state::AppState;
use crate::handlers::sede_handler::{list_sedes, get_sede, create_sede, update_sede};
use crate::handlers::device_handler::{list_devices, get_device, create_device, update_device, delete_device};
use crate::handlers::alert_handler::{list_alerts, get_alert, create_alert, acknowledge_alert, resolve_alert, get_alert_stats};
use crate::openapi::ApiDoc;
use monitoring::health::HealthRegistry;
use utoipa::OpenApi;

pub fn create_router() -> Router<Arc<AppState>> {
    let _api_doc = ApiDoc::openapi();
    let health_registry = Arc::new(HealthRegistry::new());

    Router::new()
        .route("/health/live", get(liveness_handler))
        .route("/health/ready", get(readiness_handler))
        .route("/docs", get(scalar_docs))
        .route("/api-docs.json", get(openapi_json))
        .route("/api/v1/sedes", get(list_sedes).post(create_sede))
        .route("/api/v1/sedes/:id", get(get_sede).put(update_sede))
        .route("/api/v1/devices", get(list_devices).post(create_device))
        .route("/api/v1/devices/:id", get(get_device).put(update_device).delete(delete_device))
        .route("/api/v1/alerts", get(list_alerts).post(create_alert))
        .route("/api/v1/alerts/stats", get(get_alert_stats))
        .route("/api/v1/alerts/:id", get(get_alert))
        .route("/api/v1/alerts/:id/acknowledge", put(acknowledge_alert))
        .route("/api/v1/alerts/:id/resolve", put(resolve_alert))
        .layer(crate::middleware::cors_layer())
        .layer(crate::middleware::tracing_layer())
        .with_state(Arc::new(AppState::new(
            Arc::new(MockSedeRepository),
            Arc::new(MockDeviceRepository),
            Arc::new(MockAlertRepository),
            health_registry,
        )))
}

async fn scalar_docs() -> impl IntoResponse {
    use axum::response::Html;
    let openapi = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi).unwrap_or_default();
    Html(format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>API Documentation</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        pre {{ background: #f4f4f4; padding: 20px; overflow: auto; }}
    </style>
</head>
<body>
    <h1>API Documentation (OpenAPI JSON)</h1>
    <p>Get the raw JSON: <a href="/api-docs.json">/api-docs.json</a></p>
    <pre>{}</pre>
</body>
</html>"#, json))
}

async fn openapi_json() -> impl IntoResponse {
    axum::Json(ApiDoc::openapi())
}

async fn liveness_handler() -> &'static str {
    "OK"
}

async fn readiness_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let health = state.health_registry.check_all().await;
    if health.overall_healthy {
        (axum::http::StatusCode::OK, axum::Json(health))
    } else {
        (axum::http::StatusCode::SERVICE_UNAVAILABLE, axum::Json(health))
    }
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

struct MockAlertRepository;

impl domain::ports::AlertRepository for MockAlertRepository {
    fn save(&self, _alert: &domain::entities::Alert) -> Result<domain::entities::Alert, domain::errors::DomainError> {
        Ok(_alert.clone())
    }

    fn find_active(&self) -> Result<Vec<domain::entities::Alert>, domain::errors::DomainError> {
        Ok(vec![])
    }

    fn acknowledge(&self, _id: uuid::Uuid, _user_id: uuid::Uuid) -> Result<(), domain::errors::DomainError> {
        Ok(())
    }

    fn find_by_device(&self, _device_id: uuid::Uuid, _limit: usize) -> Result<Vec<domain::entities::Alert>, domain::errors::DomainError> {
        Ok(vec![])
    }

    fn find_by_status(&self, _status: domain::entities::AlertStatus) -> Result<Vec<domain::entities::Alert>, domain::errors::DomainError> {
        Ok(vec![])
    }

    fn resolve(&self, _id: uuid::Uuid) -> Result<(), domain::errors::DomainError> {
        Ok(())
    }
}