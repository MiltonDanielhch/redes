//! Ubicación: `crates/domain/src/ports/audit_repository.rs`
//!
//! Descripción: Puerto para repositorio de auditoría.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0006 (RBAC)

use crate::entities::AuditLog;
use crate::errors::DomainError;

pub trait AuditRepository: Send + Sync {
    fn log(&self, entry: &AuditLog) -> Result<AuditLog, DomainError>;
    fn find_by_resource(&self, resource: &str, limit: usize) -> Result<Vec<AuditLog>, DomainError>;
    fn find_by_user(&self, user_id: uuid::Uuid, limit: usize) -> Result<Vec<AuditLog>, DomainError>;
}