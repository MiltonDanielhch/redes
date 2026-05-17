//! Ubicación: `crates/infrastructure/src/dto/error.rs`
//!
//! Descripción: DTOs para errores de API.
//!
//! ADRs relacionados: 0003 (Axum), 0007

use axum::{
    http::StatusCode,
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

    pub fn bad_request(msg: &str) -> Self {
        Self {
            error: "Bad request".to_string(),
            details: Some(msg.to_string()),
        }
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            error: "Internal server error".to_string(),
            details: Some(msg.into()),
        }
    }

    pub fn unauthorized(msg: &str) -> Self {
        Self {
            error: "Unauthorized".to_string(),
            details: Some(msg.to_string()),
        }
    }

    pub fn forbidden(msg: &str) -> Self {
        Self {
            error: "Forbidden".to_string(),
            details: Some(msg.to_string()),
        }
    }

    pub fn into_response_with_status(self, status: StatusCode) -> axum::response::Response {
        let body = Json(json!({
            "error": self.error,
            "details": self.details,
        }));
        (status, body).into_response()
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> axum::response::Response {
        self.into_response_with_status(StatusCode::INTERNAL_SERVER_ERROR)
    }
}