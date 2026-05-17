//! Ubicación: `crates/sync/src/lib.rs`
//!
//! Descripción: Módulo de sincronización offline-first. Gestiona cola de sync
//!              para operación en sedes con conectividad inestable.
//!
//! ADRs relacionados: 0021 (Local-First Sync Offline), 0020 (Monitoreo Regional)

pub mod queue;
pub mod engine;