//! Ubicación: `crates/jobs/src/workers/metrics_worker.rs`
//!
//! Descripción: Job para procesar métricas de dispositivos.
//!
//! ADRs relacionados: 0015 (Apalis Jobs), 0020 (Monitoreo Regional)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsJob {
    pub device_id: uuid::Uuid,
}

impl MetricsJob {
    pub fn new(device_id: uuid::Uuid) -> Self {
        Self { device_id }
    }

    pub async fn process(&self) {
        tracing::info!("Processing metrics for device: {}", self.device_id);
    }
}