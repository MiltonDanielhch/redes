//! Ubicación: `crates/domain/src/entities/alert.rs`
//!
//! Descripción: Entidad Alert para notificaciones de monitoreo.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    DeviceOffline,
    BandwidthSaturation,
    PacketLoss,
    Intrusion,
    TopologyChange,
    HighTraffic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: uuid::Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub device_id: Option<uuid::Uuid>,
    pub message: String,
    pub details: Option<String>,
    pub status: AlertStatus,
    pub acknowledged_by: Option<uuid::Uuid>,
    pub acknowledged_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Alert {
    pub fn new(
        alert_type: AlertType,
        severity: AlertSeverity,
        device_id: Option<uuid::Uuid>,
        message: String,
        details: Option<String>,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: uuid::Uuid::new_v4(),
            alert_type,
            severity,
            device_id,
            message,
            details,
            status: AlertStatus::Active,
            acknowledged_by: None,
            acknowledged_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn acknowledge(&mut self, user_id: uuid::Uuid) {
        self.status = AlertStatus::Acknowledged;
        self.acknowledged_by = Some(user_id);
        self.acknowledged_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved;
        self.updated_at = OffsetDateTime::now_utc();
    }
}