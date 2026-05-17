//! Ubicación: `crates/infrastructure/src/dto/sede.rs`
//!
//! Descripción: DTOs para operaciones con sedes.
//!
//! ADRs relacionados: 0003 (Axum), 0020

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSedeRequest {
    #[schema(example = "Sede Central")]
    pub nombre: String,
    #[schema(example = "Riberalta")]
    pub ubicacion: String,
    #[schema(example = "Gobernación")]
    pub secretaria: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSedeRequest {
    #[schema(example = "Sede Nueva")]
    pub nombre: Option<String>,
    #[schema(example = "Guayaramerín")]
    pub ubicacion: Option<String>,
    #[schema(example = "Secretaría de Infraestructura")]
    pub secretaria: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SedeResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    #[schema(example = "Sede Central")]
    pub nombre: String,
    #[schema(example = "Riberalta")]
    pub ubicacion: String,
    #[schema(example = "Gobernación")]
    pub secretaria: String,
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub created_at: String,
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub updated_at: String,
}