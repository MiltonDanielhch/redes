//! Ubicación: `crates/infrastructure/src/dto/sede.rs`
//!
//! Descripción: DTOs para operaciones con sedes.
//!
//! ADRs relacionados: 0003 (Axum), 0020

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateSedeRequest {
    pub nombre: String,
    pub ubicacion: String,
    pub secretaria: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSedeRequest {
    pub nombre: Option<String>,
    pub ubicacion: Option<String>,
    pub secretaria: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SedeResponse {
    pub id: String,
    pub nombre: String,
    pub ubicacion: String,
    pub secretaria: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<domain::entities::Sede> for SedeResponse {
    fn from(sede: domain::entities::Sede) -> Self {
        Self {
            id: sede.id.to_string(),
            nombre: sede.nombre,
            ubicacion: sede.ubicacion,
            secretaria: sede.secretaria,
            created_at: sede.created_at.to_string(),
            updated_at: sede.updated_at.to_string(),
        }
    }
}