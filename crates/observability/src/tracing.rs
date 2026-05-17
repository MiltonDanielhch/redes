//! Ubicación: `crates/observability/src/tracing.rs`
//!
//! Descripción: Tracing con soporte OpenTelemetry y headers X-Trace-ID.

use axum::extract::Request;

pub fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json();

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");
}

pub fn get_current_span() -> Option<String> {
    // Simplificado - retorna el trace ID del span actual si existe
    let span = tracing::Span::current();
    let span_id: Option<tracing::Id> = span.into();
    span_id.map(|id| format!("span-{:?}", id))
}

pub fn add_trace_id_to_response<B>(request: &Request<B>) -> Option<String> {
    // Simplificado - en producción esto extraería el trace ID de los headers
    request.headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}