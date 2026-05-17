//! Ubicación: `crates/database/src/lib.rs`
//!
//! Descripción: Capa de base de datos - acceso a datos y repositorios SQLx.
//!              TODO Implementar repositorios concretos para PostgreSQL.
//!
//! ADRs relacionados: 0004 (PostgreSQL), 0020 (Monitoreo Regional)

pub mod repositories;
pub mod pool;