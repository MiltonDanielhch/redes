//! Ubicación: `crates/infrastructure/src/middleware/mod.rs`
//!
//! Descripción: Middleware de tracing para logging de requests HTTP.
//!
//! ADRs relacionados: 0003 (Axum 0.8)

use tower_http::trace::TraceLayer;

pub fn tracing_layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}