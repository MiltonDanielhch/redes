//! Ubicación: `crates/domain/src/lib.rs`
//!
//! Descripción: Capa de dominio - lógica de negocio pura sin dependencias externas.
//!              Contiene entidades, value objects, errores de dominio y puertos (traits).
//!              Implementa Arquitectura Hexagonal siguiendo ADR 0001.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0004 (PostgreSQL), 0006 (RBAC), 0008 (PASETO)

pub mod entities;
pub mod errors;
pub mod ports;
pub mod value_objects;