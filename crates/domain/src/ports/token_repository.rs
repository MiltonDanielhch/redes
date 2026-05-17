//! Ubicación: `crates/domain/src/ports/token_repository.rs`
//!
//! Descripción: Puerto para repositorio de tokens.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0008 (PASETO)

use crate::entities::Token;
use crate::errors::DomainError;

pub trait TokenRepository: Send + Sync {
    fn create(&self, token: &Token) -> Result<Token, DomainError>;
    fn find_by_hash(&self, hash: &str) -> Result<Option<Token>, DomainError>;
    fn delete_by_user_id(&self, user_id: uuid::Uuid) -> Result<(), DomainError>;
    fn delete_expired(&self) -> Result<u64, DomainError>;
}