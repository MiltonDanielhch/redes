//! Ubicación: `crates/observability/src/metrics.rs`
//!
//! Descripción: Métricas Prometheus con metrics crate.

use std::sync::Arc;
use metrics::{Counter, Gauge, Histogram, Register};
use metrics_exporter_prometheus::PrometheusBuilder;
use axum::{
    Router,
    routing::get,
    response::IntoResponse,
    extract::State,
};
use std::sync::RwLock;

#[derive(Clone)]
pub struct MetricsService {
    request_count: Counter,
    request_duration: Histogram,
    active_requests: Gauge,
    error_count: Counter,
}

impl MetricsService {
    pub fn new() -> Self {
        Self {
            request_count: Counter::from_parts(
                metrics::Descriptor::new(
                    "http_requests_total".into(),
                    "Total number of HTTP requests".into(),
                    metrics::Unit::Count,
                ),
                metrics::CounterValue::Unsigned(0),
            ),
            request_duration: Histogram::from_parts(
                metrics::Descriptor::new(
                    "http_request_duration_seconds".into(),
                    "HTTP request duration in seconds".into(),
                    metrics::Unit::Seconds,
                ),
                metrics::HistogramValue::new(&[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            ),
            active_requests: Gauge::from_parts(
                metrics::Descriptor::new(
                    "http_active_requests".into(),
                    "Number of active HTTP requests".into(),
                    metrics::Unit::Count,
                ),
                metrics::GaugeValue::Absolute(0.0),
            ),
            error_count: Counter::from_parts(
                metrics::Descriptor::new(
                    "http_errors_total".into(),
                    "Total number of HTTP errors".into(),
                    metrics::Unit::Count,
                ),
                metrics::CounterValue::Unsigned(0),
            ),
        }
    }

    pub fn increment_request_count(&self, method: &str, path: &str, status: u16) {
        let labels = [
            ("method".into(), method.into()),
            ("path".into(), path.into()),
            ("status".into(), status.to_string()),
        ];
        self.request_count.increment(&labels);
    }

    pub fn record_request_duration(&self, method: &str, path: &str, duration_secs: f64) {
        let labels = [
            ("method".into(), method.into()),
            ("path".into(), path.into()),
        ];
        self.request_duration.record(duration_secs, &labels);
    }

    pub fn increment_active_requests(&self) {
        self.active_requests.increment(&[]);
    }

    pub fn decrement_active_requests(&self) {
        self.active_requests.decrement(&[]);
    }

    pub fn increment_error_count(&self, error_type: &str) {
        let labels = [("type".into(), error_type.into())];
        self.error_count.increment(&labels);
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
    use metrics_exporter_prometheus::PrometheusHandle;
    let handle = PrometheusHandle::current();
    axum::response::Html(handle.render())
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