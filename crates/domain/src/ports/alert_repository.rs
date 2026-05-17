//! Ubicación: `crates/domain/src/ports/alert_repository.rs`
//!
//! Descripción: Puerto para repositorio de alertas.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use crate::entities::{Alert, AlertStatus};
use crate::errors::DomainError;

pub trait AlertRepository: Send + Sync {
    fn save(&self, alert: &Alert) -> Result<Alert, DomainError>;
    fn find_active(&self) -> Result<Vec<Alert>, DomainError>;
    fn acknowledge(&self, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<(), DomainError>;
    fn find_by_device(&self, device_id: uuid::Uuid, limit: usize) -> Result<Vec<Alert>, DomainError>;
    fn find_by_status(&self, status: AlertStatus) -> Result<Vec<Alert>, DomainError>;
}