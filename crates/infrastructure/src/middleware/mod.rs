//! Ubicación: `crates/infrastructure/src/middleware/mod.rs`
//!
//! Descripción: Middleware de tracing y CORS para requests HTTP.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0007

use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;

pub fn tracing_layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}