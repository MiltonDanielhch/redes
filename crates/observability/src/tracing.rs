//! Ubicación: `crates/observability/src/tracing.rs`
//!
//! Descripción: Tracing con soporte OpenTelemetry y headers X-Trace-ID.

use tracing::Span;
use axum::extract::Request;

pub fn init_tracing() -> tracing_subscriber::reload::Handle {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    use std::sync::Arc;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json();

    let (subscriber, handle) = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(open_telemetry_layer())
        .reload();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");

    handle
}

fn open_telemetry_layer() -> impl tracing_subscriber::layer::Layer<tracing_subscriber::Registry> {
    tracing_opentelemetry::layer()
}

pub fn get_current_span() -> Option<String> {
    use opentelemetry::trace::TraceContextExt;
    tracing::info_span!("test").in_scope(|| {
        opentelemetry::trace::Span::current().span_context().trace_id().to_hex().into()
    })
}

pub fn add_trace_id_to_response<B>(request: &Request<B>) -> Option<String> {
    use opentelemetry::propagation::TextMapPropagator;
    use std::collections::HashMap;

    let headers = request.headers();
    let mut map = HashMap::new();

    for (key, value) in headers {
        if let Ok(v) = value.to_str() {
            map.insert(key.as_str(), v);
        }
    }

    let propagator = opentelemetry::global::get_text_map_propagator(
        |p| p.clone()
    );

    let extracted = propagator.extract(&map);

    extracted.and_then(|ctx| {
        ctx.span().span_context().trace_id().to_hex().into()
    })
}

pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}