//! Ubicación: `crates/domain/src/entities/session.rs`
//!
//! Descripción: Entidad Session para gestión de sesiones de usuario.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0008 (PASETO)

use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl Session {
    pub fn new(user_id: uuid::Uuid, token_hash: String, expires_at: OffsetDateTime) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: uuid::Uuid::new_v4(),
            user_id,
            token_hash,
            ip_address: None,
            user_agent: None,
            expires_at,
            created_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() > self.expires_at
    }
}