//! Ubicación: `crates/infrastructure/src/handlers/device_handler.rs`
//!
//! Descripción: Handlers HTTP para operaciones con dispositivos.
//!
//! ADRs relacionados: 0003 (Axum), 0020, 0016 (OpenAPI)

use axum::{
    extract::{Path, State},
    response::Json,
};
use std::sync::Arc;

use crate::dto::{device::{CreateDeviceRequest, UpdateDeviceRequest, DeviceResponse}, error::ApiErrorResponse};
use crate::state::AppState;
use domain::entities::{Device, DeviceType, DeviceStatus};
use domain::ports::DeviceRepository;

#[utoipa::path(
    get,
    path = "/api/v1/devices",
    responses(
        (status = 200, description = "Lista de dispositivos", body = Vec<DeviceResponse>)
    ),
    tag = "devices"
)]
pub async fn list_devices(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DeviceResponse>>, ApiErrorResponse> {
    let devices = state.device_repository.list()
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    let response: Vec<DeviceResponse> = devices.into_iter().map(|d| d.into()).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/devices/{id}",
    params(
        ("id" = String, Path, description = "UUID del dispositivo")
    ),
    responses(
        (status = 200, description = "Dispositivo encontrado", body = DeviceResponse),
        (status = 404, description = "Dispositivo no encontrado")
    ),
    tag = "devices"
)]
pub async fn get_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<DeviceResponse>, ApiErrorResponse> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    let device = state.device_repository.find_by_id(uuid)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiErrorResponse::not_found("Device"))?;

    Ok(Json(device.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/devices",
    request_body = CreateDeviceRequest,
    responses(
        (status = 201, description = "Dispositivo creado", body = DeviceResponse),
        (status = 400, description = "Datos inválidos")
    ),
    tag = "devices"
)]
pub async fn create_device(
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

#[utoipa::path(
    put,
    path = "/api/v1/devices/{id}",
    params(
        ("id" = String, Path, description = "UUID del dispositivo")
    ),
    request_body = UpdateDeviceRequest,
    responses(
        (status = 200, description = "Dispositivo actualizado", body = DeviceResponse),
        (status = 404, description = "Dispositivo no encontrado")
    ),
    tag = "devices"
)]
pub async fn update_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
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

#[utoipa::path(
    delete,
    path = "/api/v1/devices/{id}",
    params(
        ("id" = String, Path, description = "UUID del dispositivo")
    ),
    responses(
        (status = 200, description = "Dispositivo eliminado"),
        (status = 404, description = "Dispositivo no encontrado")
    ),
    tag = "devices"
)]
pub async fn delete_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    state.device_repository.soft_delete(uuid)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(serde_json::json!({ "message": "Device deleted" })))
}

impl From<Device> for DeviceResponse {
    fn from(d: Device) -> Self {
        Self {
            id: d.id.to_string(),
            hostname: d.hostname,
            ip_address: d.ip_address,
            mac_address: d.mac_address,
            device_type: format!("{:?}", d.device_type),
            status: format!("{:?}", d.status),
            sede_id: d.sede_id.to_string(),
            last_seen_at: d.last_seen_at.map(|t| t.to_string()),
            created_at: d.created_at.to_string(),
        }
    }
}