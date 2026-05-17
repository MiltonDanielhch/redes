//! Ubicación: `crates/infrastructure/src/dto/device.rs`
//!
//! Descripción: DTOs para operaciones con dispositivos.
//!
//! ADRs relacionados: 0003 (Axum), 0020

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateDeviceRequest {
    pub hostname: String,
    pub ip_address: String,
    pub device_type: String,
    pub sede_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub hostname: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub id: String,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: Option<String>,
    pub device_type: String,
    pub status: String,
    pub sede_id: String,
    pub last_seen_at: Option<String>,
    pub created_at: String,
}