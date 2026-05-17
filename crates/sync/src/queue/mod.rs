//! Ubicación: `crates/sync/src/queue/mod.rs`
//!
//! Descripción: Cola de sincronización para operaciones offline.
//!
//! ADRs relacionados: 0021 (Local-First Sync Offline)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncQueueItem {
    pub id: String,
    pub operation: SyncOperation,
    pub entity_type: String,
    pub entity_id: String,
    pub payload: String,
    pub timestamp: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncOperation {
    Create,
    Update,
    Delete,
}