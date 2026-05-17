//! Ubicación: `crates/domain/src/entities/mod.rs`
//!
//! Descripción: Módulo de entidades de dominio. Contiene todas las entidades del
//!              negocio como Sede, Device, MetricReading, Alert, etc.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

mod sede;

pub use sede::Sede;