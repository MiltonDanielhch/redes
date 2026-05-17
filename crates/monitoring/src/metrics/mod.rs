//! Ubicación: `crates/monitoring/src/metrics/mod.rs`
//!
//! Descripción: Estructuras de datos para métricas de red.
//!
//! ADRs relacionados: 0020 (Monitoreo Regional)

use uuid::Uuid;
use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

/// Lectura de métricas de un dispositivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReading {
    pub id: Uuid,
    pub device_id: Uuid,
    pub bandwidth_rx_kbps: f64,
    pub bandwidth_tx_kbps: f64,
    pub latency_ms: f64,
    pub packet_loss: f64,
    pub anomaly: bool,
    pub recorded_at: OffsetDateTime,
}

impl MetricReading {
    pub fn new(device_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            device_id,
            bandwidth_rx_kbps: 0.0,
            bandwidth_tx_kbps: 0.0,
            latency_ms: 0.0,
            packet_loss: 0.0,
            anomaly: false,
            recorded_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn is_anomaly(&self) -> bool {
        self.anomaly || self.latency_ms > 100.0 || self.packet_loss > 5.0
    }
}