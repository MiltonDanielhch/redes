//! Ubicación: `crates/auth/src/errors.rs`
//!
//! Descripción: Tipos de errores para la capa de autenticación.
//!
//! ADRs relacionados: 0008 (PASETO)

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    TokenExpired,
    TokenInvalid,
    PasswordHashingFailed,
    Internal(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::TokenExpired => write!(f, "Token expired"),
            AuthError::TokenInvalid => write!(f, "Token invalid"),
            AuthError::PasswordHashingFailed => write!(f, "Password hashing failed"),
            AuthError::Internal(msg) => write!(f, "Internal: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}