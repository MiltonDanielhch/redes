//! Ubicación: `crates/domain/src/entities/role.rs`
//!
//! Descripción: Entidad Role para RBAC.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0006 (RBAC)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl Role {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            description,
        }
    }
}