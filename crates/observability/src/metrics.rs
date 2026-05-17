//! Ubicación: `crates/observability/src/metrics.rs`
//!
//! Descripción: Métricas Prometheus con metrics crate.

use std::sync::Arc;
use metrics_exporter_prometheus::PrometheusBuilder;
use axum::{
    Router,
    routing::get,
    response::IntoResponse,
};

#[derive(Clone)]
pub struct MetricsService {
    // Placeholder - métricas implementadas vía macros globales
}

impl MetricsService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn increment_request_count(&self, _method: &str, _path: &str, _status: u16) {
        // Implementación simplificada - en producción usar macros de metrics
    }

    pub fn record_request_duration(&self, _method: &str, _path: &str, _duration_secs: f64) {
        // Implementación simplificada - en producción usar macros de metrics
    }

    pub fn increment_active_requests(&self) {
        // Implementación simplificada - en producción usar macros de metrics
    }

    pub fn decrement_active_requests(&self) {
        // Implementación simplificada - en producción usar macros de metrics
    }

    pub fn increment_error_count(&self, _error_type: &str) {
        // Implementación simplificada - en producción usar macros de metrics
    }
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn init_metrics() -> anyhow::Result<Arc<MetricsService>> {
    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 9090))
        .install()?;

    Ok(Arc::new(MetricsService::new()))
}

pub async fn metrics_handler() -> impl IntoResponse {
    // Implementación simplificada - en producción usar PrometheusHandle
    axum::response::Html("<html><body>Prometheus metrics endpoint</body></html>")
}

#[derive(Clone)]
pub struct AppMetrics {
    pub service: Arc<MetricsService>,
}

impl AppMetrics {
    pub fn new() -> Self {
        Self {
            service: Arc::new(MetricsService::new()),
        }
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(self.service.clone())
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new()
    }
}