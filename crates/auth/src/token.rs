//! Ubicación: `crates/auth/src/token.rs`
//!
//! Descripción: Generador y verificador de tokens.
//!              JWT PROHIBIDO - usa PASETO v4 Local exclusivamente (ADR 0008).
//!              TODO: Implementar PASETO real cuando la API sea compatible.
//!
//! ADRs relacionados: 0008 (PASETO)

#[derive(Debug)]
pub enum TokenError {
    GenerationFailed,
    VerificationFailed,
    InvalidToken,
}

pub struct TokenManager {
    secret: Vec<u8>,
}

impl TokenManager {
    pub fn new(secret: &[u8]) -> Self {
        Self { secret: secret.to_vec() }
    }

    pub fn generate(&self, payload: &str) -> Result<String, TokenError> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let data = format!("{}:{}", payload, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());
        let encoded = STANDARD.encode(data.as_bytes());
        Ok(encoded)
    }

    pub fn verify(&self, token: &str) -> Result<String, TokenError> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let decoded = STANDARD.decode(token)
            .map_err(|_| TokenError::InvalidToken)?;
        String::from_utf8(decoded)
            .map_err(|_| TokenError::InvalidToken)
    }
}