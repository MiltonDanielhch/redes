//! Ubicación: `apps/api/src/main.rs`
//!
//! Descripción: Punto de entrada del servidor API Axum 0.8.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0020 (Monitoreo Regional)

use axum::{Router, routing::get};
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting API server...");

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn ready() -> &'static str {
    "READY"
}