//! Ubicación: `crates/infrastructure/src/middleware/mod.rs`
//!
//! Descripción: Middleware de tracing para logging de requests HTTP.
//!
//! ADRs relacionados: 0003 (Axum 0.8)

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn tracing_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    let duration = start.elapsed();
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        "HTTP request"
    );

    response
}