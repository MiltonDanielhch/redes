//! Ubicación: `crates/domain/src/ports/sede_repository.rs`
//!
//! Descripción: Puerto (trait) para repositorio de Sedes. Define las operaciones
//!              de persistencia que debe implementar cualquier adaptador de base de datos.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0004 (PostgreSQL)

use crate::entities::Sede;
use crate::errors::DomainError;

/// Trait que define las operaciones del repositorio de Sedes
///
/// Implementado por: `PostgresSedeRepository` en `crates/database`
pub trait SedeRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<Sede>, DomainError>;
    fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Sede>, DomainError>;
    fn save(&self, sede: &Sede) -> Result<Sede, DomainError>;
    fn delete(&self, id: uuid::Uuid) -> Result<(), DomainError>;
}