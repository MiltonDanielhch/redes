//! Ubicación: `crates/domain/src/ports/intrusion_repository.rs`
//!
//! Descripción: Puerto para repositorio de eventos de intrusión.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use crate::entities::{IntrusionEvent, IntrusionStatus};
use crate::errors::DomainError;

pub trait IntrusionRepository: Send + Sync {
    fn save(&self, event: &IntrusionEvent) -> Result<IntrusionEvent, DomainError>;
    fn find_pending(&self) -> Result<Vec<IntrusionEvent>, DomainError>;
    fn update_status(&self, id: uuid::Uuid, status: IntrusionStatus) -> Result<(), DomainError>;
    fn find_by_status(&self, status: IntrusionStatus) -> Result<Vec<IntrusionEvent>, DomainError>;
}