//! Ubicación: `crates/domain/src/ports/user_repository.rs`
//!
//! Descripción: Puerto para repositorio de usuarios.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0006 (RBAC)

use crate::entities::User;
use crate::errors::DomainError;

pub trait UserRepository: Send + Sync {
    fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, DomainError>;
    fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    fn save(&self, user: &User) -> Result<User, DomainError>;
    fn soft_delete(&self, id: uuid::Uuid) -> Result<(), DomainError>;
    fn list(&self) -> Result<Vec<User>, DomainError>;
}