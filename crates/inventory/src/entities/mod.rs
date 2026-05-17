//! Ubicación: `crates/inventory/src/entities/mod.rs`
//!
//! Descripción: Entidades del módulo de inventario.
//!
//! ADRs relacionados: 0020 (Monitoreo Regional)

mod device;

pub use device::{Device, DeviceType, DeviceStatus};