//! Ubicación: `crates/monitoring/src/lib.rs`
//!
//! Descripción: Módulo de monitoreo - métricas de red, health checks y detección
//!              de anomalías. Recolecta bandwidth, latencia, packet loss.
//!
//! ADRs relacionados: 0014 (Healthchecks), 0020 (Monitoreo Regional)

pub mod metrics;
pub mod health;