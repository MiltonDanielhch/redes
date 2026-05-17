//! Ubicación: `crates/domain/src/entities/device_link.rs`
//!
//! Descripción: Entidad DeviceLink para conexiones entre dispositivos.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    Ethernet,
    Fiber,
    Wireless,
    Serial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLink {
    pub id: uuid::Uuid,
    pub source_device_id: uuid::Uuid,
    pub target_device_id: uuid::Uuid,
    pub link_type: LinkType,
    pub bandwidth_mbps: Option<i32>,
    pub status: String,
    pub created_at: time::OffsetDateTime,
}

impl DeviceLink {
    pub fn new(
        source_device_id: uuid::Uuid,
        target_device_id: uuid::Uuid,
        link_type: LinkType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            source_device_id,
            target_device_id,
            link_type,
            bandwidth_mbps: None,
            status: "active".to_string(),
            created_at: time::OffsetDateTime::now_utc(),
        }
    }
}