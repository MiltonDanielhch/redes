//! Ubicación: `crates/domain/src/ports/session_repository.rs`
//!
//! Descripción: Puerto para repositorio de sesiones.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0008 (PASETO)

use crate::entities::Session;
use crate::errors::DomainError;

pub trait SessionRepository: Send + Sync {
    fn create(&self, session: &Session) -> Result<Session, DomainError>;
    fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, DomainError>;
    fn delete_by_user_id(&self, user_id: uuid::Uuid) -> Result<(), DomainError>;
    fn delete_expired(&self) -> Result<u64, DomainError>;
}