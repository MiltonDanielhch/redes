//! Ubicación: `crates/infrastructure/src/dto/error.rs`
//!
//! Descripción: DTOs para errores de API.
//!
//! ADRs relacionados: 0003 (Axum), 0007

use axum::{
    response::IntoResponse,
    Json,
};
use serde_json::json;

#[derive(Debug, serde::Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let body = Json(json!({
            "error": self.error,
            "details": self.details,
        }));

        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl ApiErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }

    pub fn with_details(error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: Some(details.into()),
        }
    }

    pub fn not_found(resource: &str) -> Self {
        Self {
            error: format!("{} not found", resource),
            details: None,
        }
    }

    pub fn bad_request(message: &str) -> Self {
        Self {
            error: message.to_string(),
            details: None,
        }
    }
}