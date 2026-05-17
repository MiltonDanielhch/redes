//! Ubicación: `crates/infrastructure/src/lib.rs`
//!
//! Descripción: Capa de infraestructura - adaptadores HTTP (Axum), configuración,
//!              middleware y manejo de errores. orquesta toda la aplicación.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0002 (Config)

pub mod config;
pub mod dto;
pub mod handlers;
pub mod routes;
pub mod middleware;
pub mod error;