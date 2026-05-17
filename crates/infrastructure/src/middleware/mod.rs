//! Ubicación: `crates/infrastructure/src/middleware/mod.rs`
//!
//! Descripción: Middleware de tracing, CORS y auditoría para requests HTTP.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0007, 0006

use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub mod audit_channel;
pub mod action_logger;
pub mod rbac;

pub fn tracing_layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}