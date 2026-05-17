//! Ubicación: `crates/infrastructure/src/routes/mod.rs`
//!
//! Descripción: Definición de rutas HTTP de la API usando Axum 0.8.
//!
//! ADRs relacionados: 0003 (Axum 0.8)

use axum::{
    Router,
    routing::get,
};
use tower_http::cors::{CorsLayer, Any};

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn ready_handler() -> &'static str {
    "READY"
}