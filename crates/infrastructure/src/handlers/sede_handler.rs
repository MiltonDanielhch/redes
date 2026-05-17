//! Ubicación: `crates/infrastructure/src/handlers/sede_handler.rs`
//!
//! Descripción: Handlers HTTP para operaciones con sedes.
//!
//! ADRs relacionados: 0003 (Axum), 0020

use axum::{
    extract::{Path, State},
    response::Json,
};
use std::sync::Arc;

use crate::dto::{sede::{CreateSedeRequest, UpdateSedeRequest, SedeResponse}, error::ApiErrorResponse};
use domain::entities::Sede;
use domain::ports::SedeRepository;

pub struct SedeHandlers<R: SedeRepository> {
    repository: Arc<R>,
}

impl<R: SedeRepository> SedeHandlers<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn list(
        State(repository): State<Arc<R>>,
    ) -> Result<Json<Vec<SedeResponse>>, ApiErrorResponse> {
        let sedes = repository.find_all()
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

        let response: Vec<SedeResponse> = sedes.into_iter().map(|s| s.into()).collect();
        Ok(Json(response))
    }

    pub async fn get(
        State(repository): State<Arc<R>>,
        Path(id): Path<String>,
    ) -> Result<Json<SedeResponse>, ApiErrorResponse> {
        let uuid = uuid::Uuid::parse_str(&id)
            .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

        let sede = repository.find_by_id(uuid)
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
            .ok_or_else(|| ApiErrorResponse::not_found("Sede"))?;

        Ok(Json(sede.into()))
    }

    pub async fn create(
        State(repository): State<Arc<R>>,
        Json(payload): Json<CreateSedeRequest>,
    ) -> Result<Json<SedeResponse>, ApiErrorResponse> {
        let sede = Sede::new(
            payload.nombre,
            payload.ubicacion,
            payload.secretaria,
        );

        let created = repository.save(&sede)
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

        Ok(Json(created.into()))
    }

    pub async fn update(
        State(repository): State<Arc<R>>,
        Path(id): Path<String>,
        Json(payload): Json<UpdateSedeRequest>,
    ) -> Result<Json<SedeResponse>, ApiErrorResponse> {
        let uuid = uuid::Uuid::parse_str(&id)
            .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

        let mut sede = repository.find_by_id(uuid)
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

        let updated = repository.save(&sede)
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

        Ok(Json(updated.into()))
    }
}