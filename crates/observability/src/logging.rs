//! Ubicación: `crates/observability/src/logging.rs`
//!
//! Descripción: Logging estructurado JSON con redaction de datos sensibles.

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use serde::Serialize;
use std::collections::HashMap;

const SENSITIVE_KEYS: &[&str] = &[
    "password",
    "secret",
    "token",
    "api_key",
    "apikey",
    "authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
];

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

pub fn redact_sensitive_data(data: &mut HashMap<String, serde_json::Value>) {
    for key in SENSITIVE_KEYS.iter() {
        if data.contains_key(*key) {
            data.insert(key.to_string(), serde_json::Value::String("[REDACTED]".to_string()));
        }
    }
}

pub fn is_sensitive_key(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    SENSITIVE_KEYS.iter().any(|s| key_lower.contains(&s.to_lowercase()))
}

pub struct RequestLogContext {
    pub request_id: String,
    pub trace_id: Option<String>,
    pub user_id: Option<String>,
    pub path: String,
    pub method: String,
}

impl RequestLogContext {
    pub fn new(request_id: String, path: String, method: String) -> Self {
        Self {
            request_id,
            trace_id: None,
            user_id: None,
            path,
            method,
        }
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}