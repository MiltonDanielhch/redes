//! Ubicación: `crates/domain/src/errors/mod.rs`
//!
//! Descripción: Tipos de errores del dominio. Errores de negocio puro sin dependencias.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0007 (Manejo de Errores)

/// Errores de dominio - representan fallos en reglas de negocio
#[derive(Debug)]
pub enum DomainError {
    NotFound(String),
    Validation(String),
    Concurrency,
    Internal(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::NotFound(msg) => write!(f, "NotFound: {}", msg),
            DomainError::Validation(msg) => write!(f, "Validation: {}", msg),
            DomainError::Concurrency => write!(f, "Concurrency conflict"),
            DomainError::Internal(msg) => write!(f, "Internal: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}