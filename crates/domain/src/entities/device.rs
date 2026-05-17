//! Ubicación: `crates/domain/src/entities/device.rs`
//!
//! Descripción: Entidad Device para inventario de red.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Switch,
    AccessPoint,
    Router,
    Firewall,
    Server,
    Ups,
    Camera,
    WirelessLink,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Active,
    Offline,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: uuid::Uuid,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: Option<String>,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub sede_id: uuid::Uuid,
    pub last_seen_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl Device {
    pub fn new(
        hostname: String,
        ip_address: String,
        device_type: DeviceType,
        sede_id: uuid::Uuid,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: uuid::Uuid::new_v4(),
            hostname,
            ip_address,
            mac_address: None,
            device_type,
            status: DeviceStatus::Offline,
            sede_id,
            last_seen_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn mark_deleted(&mut self) {
        self.deleted_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn update_status(&mut self, status: DeviceStatus) {
        self.status = status;
        self.last_seen_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }
}