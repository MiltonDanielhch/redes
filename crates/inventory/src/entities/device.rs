//! Ubicación: `crates/inventory/src/entities/device.rs`
//!
//! Descripción: Entidad Device - representa un dispositivo de red en una sede.
//!              Incluye switches, routers, access points, firewalls, servers, UPS, cámaras.
//!
//! ADRs relacionados: 0020 (Monitoreo Regional)

use uuid::Uuid;
use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

/// Tipo de dispositivo de red
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Switch,
    AccessPoint,
    Router,
    Firewall,
    Server,
    Ups,
    Camera,
}

/// Estado del dispositivo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Active,
    Offline,
    Maintenance,
}

/// Entidad Device - dispositivo de red en una sede
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: Uuid,
    pub sede_id: Uuid,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: Option<String>,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl Device {
    pub fn new(
        sede_id: Uuid,
        hostname: String,
        ip_address: String,
        device_type: DeviceType,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            sede_id,
            hostname,
            ip_address,
            mac_address: None,
            device_type,
            status: DeviceStatus::Offline,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn mark_deleted(&mut self) {
        self.deleted_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }
}