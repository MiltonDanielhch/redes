//! Ubicación: `crates/infrastructure/src/error/mod.rs`
//!
//! Descripción: Tipos de errores de la capa de infraestructura.
//!
//! ADRs relacionados: 0007 (Manejo de Errores)

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
    Unauthorized,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "NotFound: {}", msg),
            AppError::BadRequest(msg) => write!(f, "BadRequest: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "NotFound: {}", msg),
            AppError::BadRequest(msg) => write!(f, "BadRequest: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (axum::http::StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Unauthorized => (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}