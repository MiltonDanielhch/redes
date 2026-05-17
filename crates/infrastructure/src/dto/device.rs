//! Ubicación: `crates/infrastructure/src/dto/device.rs`
//!
//! Descripción: DTOs para operaciones con dispositivos.
//!
//! ADRs relacionados: 0003 (Axum), 0020

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDeviceRequest {
    #[schema(example = "SW-CENTRAL-01")]
    pub hostname: String,
    #[schema(example = "192.168.1.10")]
    pub ip_address: String,
    #[schema(example = "Switch")]
    pub device_type: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub sede_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDeviceRequest {
    #[schema(example = "SW-CENTRAL-02")]
    pub hostname: Option<String>,
    #[schema(example = "192.168.1.11")]
    pub ip_address: Option<String>,
    #[schema(example = "Router")]
    pub device_type: Option<String>,
    #[schema(example = "Active")]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001")]
    pub id: String,
    #[schema(example = "SW-CENTRAL-01")]
    pub hostname: String,
    #[schema(example = "192.168.1.10")]
    pub ip_address: String,
    #[schema(example = "AA:BB:CC:DD:EE:FF")]
    pub mac_address: Option<String>,
    #[schema(example = "Switch")]
    pub device_type: String,
    #[schema(example = "Active")]
    pub status: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub sede_id: String,
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub last_seen_at: Option<String>,
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub created_at: String,
}