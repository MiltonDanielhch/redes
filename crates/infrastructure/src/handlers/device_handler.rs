//! Ubicación: `crates/infrastructure/src/handlers/device_handler.rs`
//!
//! Descripción: Handlers HTTP para operaciones con dispositivos.
//!
//! ADRs relacionados: 0003 (Axum), 0020

use axum::{
    extract::{Path, State},
    response::Json,
};
use std::sync::Arc;

use crate::dto::{device::{CreateDeviceRequest, UpdateDeviceRequest, DeviceResponse}, error::ApiErrorResponse};
use domain::entities::{Device, DeviceType, DeviceStatus};
use domain::ports::DeviceRepository;

pub struct DeviceHandlers<R: DeviceRepository> {
    repository: Arc<R>,
}

impl<R: DeviceRepository> DeviceHandlers<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn list(
        State(repository): State<Arc<R>>,
    ) -> Result<Json<Vec<DeviceResponse>>, ApiErrorResponse> {
        let devices = repository.list()
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

        let response: Vec<DeviceResponse> = devices.into_iter().map(|d| d.into()).collect();
        Ok(Json(response))
    }

    pub async fn get(
        State(repository): State<Arc<R>>,
        Path(id): Path<String>,
    ) -> Result<Json<DeviceResponse>, ApiErrorResponse> {
        let uuid = uuid::Uuid::parse_str(&id)
            .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

        let device = repository.find_by_id(uuid)
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
            .ok_or_else(|| ApiErrorResponse::not_found("Device"))?;

        Ok(Json(device.into()))
    }

    pub async fn create(
        State(repository): State<Arc<R>>,
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

        let created = repository.save(&device)
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

        Ok(Json(created.into()))
    }

    pub async fn update(
        State(repository): State<Arc<R>>,
        Path(id): Path<String>,
        Json(payload): Json<UpdateDeviceRequest>,
    ) -> Result<Json<DeviceResponse>, ApiErrorResponse> {
        let uuid = uuid::Uuid::parse_str(&id)
            .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

        let mut device = repository.find_by_id(uuid)
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

        let updated = repository.save(&device)
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

        Ok(Json(updated.into()))
    }

    pub async fn delete(
        State(repository): State<Arc<R>>,
        Path(id): Path<String>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        let uuid = uuid::Uuid::parse_str(&id)
            .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

        repository.soft_delete(uuid)
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

        Ok(Json(serde_json::json!({ "message": "Device deleted" })))
    }
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