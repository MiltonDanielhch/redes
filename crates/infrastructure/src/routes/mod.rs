//! Ubicación: `crates/infrastructure/src/routes/mod.rs`
//!
//! Descripción: Definición de rutas HTTP de la API usando Axum 0.8.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0020

use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::State,
    Json,
};
use std::sync::Arc;

use crate::state::AppState;
use crate::dto::{sede::{CreateSedeRequest, UpdateSedeRequest, SedeResponse}, device::{CreateDeviceRequest, UpdateDeviceRequest, DeviceResponse}, error::ApiErrorResponse};
use domain::entities::{Device, DeviceType, DeviceStatus};
use domain::ports::{SedeRepository, DeviceRepository};

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

async fn list_sedes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SedeResponse>>, ApiErrorResponse> {
    let sedes = state.sede_repository.find_all()
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    let response: Vec<SedeResponse> = sedes.into_iter().map(|s| s.into()).collect();
    Ok(Json(response))
}

async fn get_sede(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<SedeResponse>, ApiErrorResponse> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    let sede = state.sede_repository.find_by_id(uuid)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiErrorResponse::not_found("Sede"))?;

    Ok(Json(sede.into()))
}

async fn create_sede(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSedeRequest>,
) -> Result<Json<SedeResponse>, ApiErrorResponse> {
    let sede = domain::entities::Sede::new(
        payload.nombre,
        payload.ubicacion,
        payload.secretaria,
    );

    let created = state.sede_repository.save(&sede)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(created.into()))
}

async fn update_sede(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateSedeRequest>,
) -> Result<Json<SedeResponse>, ApiErrorResponse> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    let mut sede = state.sede_repository.find_by_id(uuid)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiErrorResponse::not_found("Sede"))?;

    if let Some(nombre) = payload.nombre {
        sede.nombre = nombre;
    }
    if let Some(ubicacion) = payload.ubicacion {
        sede.ubicacion = ubicacion;
    }
    if let Some(secretaria) = payload.secretaria {
        sede.secretaria = secretaria;
    }

    let updated = state.sede_repository.save(&sede)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(updated.into()))
}

async fn list_devices(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DeviceResponse>>, ApiErrorResponse> {
    let devices = state.device_repository.list()
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    let response: Vec<DeviceResponse> = devices.into_iter().map(|d| d.into()).collect();
    Ok(Json(response))
}

async fn get_device(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<DeviceResponse>, ApiErrorResponse> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    let device = state.device_repository.find_by_id(uuid)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiErrorResponse::not_found("Device"))?;

    Ok(Json(device.into()))
}

async fn create_device(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateDeviceRequest>,
) -> Result<Json<DeviceResponse>, ApiErrorResponse> {
    let device_type = match payload.device_type.as_str() {
        "Switch" => DeviceType::Switch,
        "AccessPoint" => DeviceType::AccessPoint,
        "Router" => DeviceType::Router,
        "Firewall" => DeviceType::Firewall,
        "Server" => DeviceType::Server,
        "Ups" => DeviceType::Ups,
        "Camera" => DeviceType::Camera,
        "WirelessLink" => DeviceType::WirelessLink,
        _ => return Err(ApiErrorResponse::bad_request("Invalid device_type")),
    };

    let sede_id = uuid::Uuid::parse_str(&payload.sede_id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid sede_id format"))?;

    let device = Device::new(
        payload.hostname,
        payload.ip_address,
        device_type,
        sede_id,
    );

    let created = state.device_repository.save(&device)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(created.into()))
}

async fn update_device(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateDeviceRequest>,
) -> Result<Json<DeviceResponse>, ApiErrorResponse> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    let mut device = state.device_repository.find_by_id(uuid)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiErrorResponse::not_found("Device"))?;

    if let Some(hostname) = payload.hostname {
        device.hostname = hostname;
    }
    if let Some(ip_address) = payload.ip_address {
        device.ip_address = ip_address;
    }
    if let Some(device_type_str) = payload.device_type {
        device.device_type = match device_type_str.as_str() {
            "Switch" => DeviceType::Switch,
            "AccessPoint" => DeviceType::AccessPoint,
            "Router" => DeviceType::Router,
            "Firewall" => DeviceType::Firewall,
            "Server" => DeviceType::Server,
            "Ups" => DeviceType::Ups,
            "Camera" => DeviceType::Camera,
            "WirelessLink" => DeviceType::WirelessLink,
            _ => return Err(ApiErrorResponse::bad_request("Invalid device_type")),
        };
    }
    if let Some(status_str) = payload.status {
        device.status = match status_str.as_str() {
            "Active" => DeviceStatus::Active,
            "Offline" => DeviceStatus::Offline,
            "Maintenance" => DeviceStatus::Maintenance,
            _ => return Err(ApiErrorResponse::bad_request("Invalid status")),
        };
    }

    let updated = state.device_repository.save(&device)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(updated.into()))
}

async fn delete_device(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    state.device_repository.soft_delete(uuid)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(serde_json::json!({ "message": "Device deleted" })))
}

struct MockSedeRepository;

impl SedeRepository for MockSedeRepository {
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

impl DeviceRepository for MockDeviceRepository {
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