//! Ubicación: `crates/domain/src/ports/metrics_repository.rs`
//!
//! Descripción: Puerto para repositorio de métricas.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use crate::entities::MetricReading;
use crate::errors::DomainError;

pub trait MetricsRepository: Send + Sync {
    fn save(&self, metric: &MetricReading) -> Result<MetricReading, DomainError>;
    fn find_by_device(&self, device_id: uuid::Uuid, limit: usize) -> Result<Vec<MetricReading>, DomainError>;
    fn find_by_date_range(
        &self,
        device_id: uuid::Uuid,
        from: time::OffsetDateTime,
        to: time::OffsetDateTime,
    ) -> Result<Vec<MetricReading>, DomainError>;
    fn get_recent(&self, limit: usize) -> Result<Vec<MetricReading>, DomainError>;
}