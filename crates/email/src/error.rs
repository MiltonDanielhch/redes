//! Ubicación: `crates/email/src/error.rs`
//!
//! Descripción: Tipos de error para el módulo de email.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("Failed to send email: {0}")]
    SendFailed(String),

    #[error("Invalid email address: {0}")]
    InvalidAddress(String),

    #[error("Template rendering failed: {0}")]
    TemplateError(String),

    #[error("Queue error: {0}")]
    QueueError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}