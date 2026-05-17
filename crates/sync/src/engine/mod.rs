//! Ubicación: `crates/sync/src/engine/mod.rs`
//!
//! Descripción: Motor de sincronización offline-first.
//!
//! ADRs relacionados: 0021 (Local-First Sync Offline)

use crate::queue::SyncQueueItem;

pub struct SyncEngine;

impl SyncEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn sync(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    pub async fn push(&self, item: SyncQueueItem) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}