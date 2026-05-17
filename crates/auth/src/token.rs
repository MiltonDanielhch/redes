//! Ubicación: `crates/auth/src/token.rs`
//!
//! Descripción: Generador y verificador de tokens PASETO v4 Local.
//!              JWT PROHIBIDO - usa PASETO v4 Local exclusivamente (ADR 0008).
//!
//! ADRs relacionados: 0008 (PASETO)

use rusty_paseto::prelude::{
    PasetoParser, PasetoBuilder, PasetoSymmetricKey, V4, Local, Key,
};
use serde::{Deserialize, Serialize};

const SECRET_SIZE: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Failed to generate token: {0}")]
    GenerationFailed(String),
    #[error("Failed to verify token: {0}")]
    VerificationFailed(String),
    #[error("Token expired")]
    Expired,
    #[error("Invalid token")]
    Invalid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: uuid::Uuid,
    pub role: String,
    pub exp: Option<i64>,
}

pub struct TokenManager {
    key: PasetoSymmetricKey<V4, Local>,
}

impl TokenManager {
    pub fn new(secret: &[u8]) -> Self {
        let mut key_bytes = [0u8; SECRET_SIZE];
        let len = secret.len().min(SECRET_SIZE);
        key_bytes[..len].copy_from_slice(&secret[..len]);
        let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(key_bytes));
        Self { key }
    }

    pub fn generate(&self, user_id: uuid::Uuid, subject: &str, role: &str) -> Result<String, TokenError> {
        let now = time::OffsetDateTime::now_utc();
        let expiry = now + time::Duration::hours(24);

        let claims = serde_json::json!({
            "sub": subject,
            "user_id": user_id.to_string(),
            "role": role,
            "iat": now.unix_timestamp(),
            "exp": expiry.unix_timestamp()
        });

        let token = PasetoBuilder::<V4, Local>::default()
            .claim("data", claims)
            .map_err(|e| TokenError::GenerationFailed(e.to_string()))?
            .build(&self.key)
            .map_err(|e| TokenError::GenerationFailed(e.to_string()))?;

        Ok(token)
    }

    pub fn verify(&self, token: &str) -> Result<Claims, TokenError> {
        let mut parser = PasetoParser::<V4, Local>::default();
        let parsed = parser
            .parse(token, &self.key)
            .map_err(|e| TokenError::VerificationFailed(e.to_string()))?;

        let data = parsed.get("data")
            .ok_or_else(|| TokenError::Invalid)?;

        let sub = data.get("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TokenError::Invalid)?;

        let user_id_str = data.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TokenError::Invalid)?;

        let user_id = uuid::Uuid::parse_str(user_id_str)
            .map_err(|_| TokenError::Invalid)?;

        let role = data.get("role")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TokenError::Invalid)?;

        Ok(Claims {
            sub: sub.to_string(),
            user_id,
            role: role.to_string(),
            exp: None,
        })
    }
}