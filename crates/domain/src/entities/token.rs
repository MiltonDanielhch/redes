//! Ubicación: `crates/domain/src/entities/token.rs`
//!
//! Descripción: Entidad Token para email_verification y password_reset.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0008 (PASETO)

use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenPurpose {
    EmailVerification,
    PasswordReset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub token_hash: String,
    pub purpose: TokenPurpose,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl Token {
    pub fn new(user_id: uuid::Uuid, token_hash: String, purpose: TokenPurpose, expires_at: OffsetDateTime) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            user_id,
            token_hash,
            purpose,
            expires_at,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() > self.expires_at
    }
}