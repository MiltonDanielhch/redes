//! Ubicación: `crates/domain/src/entities/audit_log.rs`
//!
//! Descripción: Entidad AuditLog para auditoría de acciones.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0006 (RBAC)

use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: OffsetDateTime,
}

impl AuditLog {
    pub fn new(
        user_id: Option<uuid::Uuid>,
        action: String,
        resource: String,
        resource_id: Option<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            user_id,
            action,
            resource,
            resource_id,
            details,
            ip_address: None,
            user_agent: None,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}