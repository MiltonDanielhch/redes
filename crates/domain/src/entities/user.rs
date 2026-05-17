//! Ubicación: `crates/domain/src/entities/user.rs`
//!
//! Descripción: Entidad de dominio User con Soft Delete.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0006 (RBAC), 0008 (PASETO)

use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl User {
    pub fn new(email: String, password_hash: String, name: String) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: uuid::Uuid::new_v4(),
            email,
            password_hash,
            name,
            is_active: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn mark_deleted(&mut self) {
        self.deleted_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn validate_email(email: &str) -> bool {
        email.contains('@') && email.contains('.')
    }

    pub fn validate_name(name: &str) -> bool {
        !name.trim().is_empty()
    }
}