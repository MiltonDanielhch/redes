//! Ubicación: `crates/domain/src/entities/intrusion.rs`
//!
//! Descripción: Entidad IntrusionEvent para detección de intrusiones.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntrusionStatus {
    Detected,
    Investigating,
    Resolved,
    FalsePositive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrusionEvent {
    pub id: uuid::Uuid,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub device_id: Option<uuid::Uuid>,
    pub status: IntrusionStatus,
    pub detected_at: OffsetDateTime,
    pub resolved_at: Option<OffsetDateTime>,
    pub notes: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl IntrusionEvent {
    pub fn new(
        mac_address: String,
        ip_address: Option<String>,
        device_id: Option<uuid::Uuid>,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: uuid::Uuid::new_v4(),
            mac_address,
            ip_address,
            device_id,
            status: IntrusionStatus::Detected,
            detected_at: now,
            resolved_at: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn investigate(&mut self) {
        self.status = IntrusionStatus::Investigating;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn resolve(&mut self) {
        self.status = IntrusionStatus::Resolved;
        self.resolved_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn mark_false_positive(&mut self) {
        self.status = IntrusionStatus::FalsePositive;
        self.resolved_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }
}