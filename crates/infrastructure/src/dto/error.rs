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
use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ApiErrorResponse {
    #[schema(example = "Resource not found")]
    pub error: String,
    #[schema(example = "The requested item does not exist", nullable = true)]
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