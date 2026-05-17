//! Ubicación: `crates/domain/src/ports/sede_repository.rs`
//!
//! Descripción: Puerto para repositorio de sedes.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use crate::entities::Sede;
use crate::errors::DomainError;

pub trait SedeRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<Sede>, DomainError>;
    fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Sede>, DomainError>;
    fn save(&self, sede: &Sede) -> Result<Sede, DomainError>;
}