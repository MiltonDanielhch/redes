//! Ubicación: `crates/infrastructure/src/handlers/sede_handler.rs`
//!
//! Descripción: Handlers HTTP para operaciones con sedes.
//!
//! ADRs relacionados: 0003 (Axum), 0020, 0016 (OpenAPI)

use axum::{
    extract::{Path, State},
    response::Json,
};
use std::sync::Arc;

use crate::dto::{sede::{CreateSedeRequest, UpdateSedeRequest, SedeResponse}, error::ApiErrorResponse};
use crate::state::AppState;
use domain::entities::Sede;

#[utoipa::path(
    get,
    path = "/api/v1/sedes",
    responses(
        (status = 200, description = "Lista de sedes", body = Vec<SedeResponse>)
    ),
    tag = "sedes"
)]
pub async fn list_sedes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SedeResponse>>, ApiErrorResponse> {
    let sedes = state.sede_repository.find_all()
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    let response: Vec<SedeResponse> = sedes.into_iter().map(|s| s.into()).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/sedes/{id}",
    params(
        ("id" = String, Path, description = "UUID de la sede")
    ),
    responses(
        (status = 200, description = "Sede encontrada", body = SedeResponse),
        (status = 404, description = "Sede no encontrada")
    ),
    tag = "sedes"
)]
pub async fn get_sede(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SedeResponse>, ApiErrorResponse> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    let sede = state.sede_repository.find_by_id(uuid)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiErrorResponse::not_found("Sede"))?;

    Ok(Json(sede.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/sedes",
    request_body = CreateSedeRequest,
    responses(
        (status = 201, description = "Sede creada", body = SedeResponse),
        (status = 400, description = "Datos inválidos")
    ),
    tag = "sedes"
)]
pub async fn create_sede(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSedeRequest>,
) -> Result<Json<SedeResponse>, ApiErrorResponse> {
    let sede = Sede::new(
        payload.nombre,
        payload.ubicacion,
        payload.secretaria,
    );

    let created = state.sede_repository.save(&sede)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(created.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/sedes/{id}",
    params(
        ("id" = String, Path, description = "UUID de la sede")
    ),
    request_body = UpdateSedeRequest,
    responses(
        (status = 200, description = "Sede actualizada", body = SedeResponse),
        (status = 404, description = "Sede no encontrada")
    ),
    tag = "sedes"
)]
pub async fn update_sede(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSedeRequest>,
) -> Result<Json<SedeResponse>, ApiErrorResponse> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    let mut sede = state.sede_repository.find_by_id(uuid)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiErrorResponse::not_found("Sede"))?;

    if let Some(nombre) = payload.nombre {
        sede.nombre = nombre;
    }
    if let Some(ubicacion) = payload.ubicacion {
        sede.ubicacion = ubicacion;
    }
    if let Some(secretaria) = payload.secretaria {
        sede.secretaria = secretaria;
    }

    let updated = state.sede_repository.save(&sede)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(updated.into()))
}

impl From<Sede> for SedeResponse {
    fn from(sede: Sede) -> Self {
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