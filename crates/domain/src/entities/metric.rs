//! Ubicación: `crates/domain/src/entities/metric.rs`
//!
//! Descripción: Entidad MetricReading para métricas de monitoreo.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReading {
    pub id: uuid::Uuid,
    pub device_id: uuid::Uuid,
    pub bandwidth_rx_bytes: i64,
    pub bandwidth_tx_bytes: i64,
    pub latency_ms: Option<i32>,
    pub packet_loss_percent: Option<f64>,
    pub anomaly_detected: bool,
    pub created_at: OffsetDateTime,
}

impl MetricReading {
    pub fn new(
        device_id: uuid::Uuid,
        bandwidth_rx_bytes: i64,
        bandwidth_tx_bytes: i64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            device_id,
            bandwidth_rx_bytes,
            bandwidth_tx_bytes,
            latency_ms: None,
            packet_loss_percent: None,
            anomaly_detected: false,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn with_latency(mut self, latency_ms: i32) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    pub fn with_packet_loss(mut self, packet_loss: f64) -> Self {
        self.packet_loss_percent = Some(packet_loss);
        self
    }

    pub fn detect_anomaly(&mut self) {
        if let Some(latency) = self.latency_ms {
            if latency > 100 {
                self.anomaly_detected = true;
            }
        }
        if let Some(packet_loss) = self.packet_loss_percent {
            if packet_loss > 5.0 {
                self.anomaly_detected = true;
            }
        }
    }
}