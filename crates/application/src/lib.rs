//! Ubicación: `crates/application/src/lib.rs`
//!
//! Descripción: Capa de aplicación - casos de uso y orquestación de operaciones.
//!              Esta capa depende únicamente de domain y contiene la lógica de
//!              aplicación sin conocer detalles de infraestructura (HTTP, DB, etc).
//!
//! ADRs relacionados: 0001 (Hexagonal)

pub mod services;