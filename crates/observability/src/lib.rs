//! Ubicación: `crates/observability/src/lib.rs`
//!
//! Descripción: Módulo de observabilidad - tracing, métricas y logging.
//!
//! ADRs relacionados: 0011 (Estándares de desarrollo)

pub mod tracing;
pub mod metrics;
pub mod logging;

pub use tracing::{init_tracing, shutdown_tracing, add_trace_id_to_response};
pub use metrics::MetricsService;
pub use logging::init_logging;