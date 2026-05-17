//! Ubicación: `crates/infrastructure/src/handlers/alert_handler.rs`
//!
//! Descripción: Handlers HTTP para operaciones con alertas.
//!
//! ADRs relacionados: 0003 (Axum), 0020, 0016 (OpenAPI)

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use std::sync::Arc;
use serde::Deserialize;
use serde;

use crate::dto::{alert::{CreateAlertRequest, AcknowledgeAlertRequest, AlertResponse, AlertListResponse}, error::ApiErrorResponse};
use crate::state::AppState;
use domain::entities::{AlertStatus, AlertSeverity};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAlertsQuery {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub device_id: Option<String>,
    pub limit: Option<usize>,
}

#[utoipa::path(
    get,
    path = "/api/v1/alerts",
    params(
        ("status" = Option<String>, Query, description = "Filter by status (Active, Acknowledged, Resolved)"),
        ("severity" = Option<String>, Query, description = "Filter by severity (Critical, High, Medium, Low)"),
        ("device_id" = Option<String>, Query, description = "Filter by device UUID"),
        ("limit" = Option<usize>, Query, description = "Maximum number of results")
    ),
    responses(
        (status = 200, description = "Lista de alertas", body = AlertListResponse)
    ),
    tag = "alerts"
)]
pub async fn list_alerts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAlertsQuery>,
) -> Result<Json<AlertListResponse>, ApiErrorResponse> {
    let alerts = if let Some(status_str) = query.status {
        let status = match status_str.as_str() {
            "Active" => AlertStatus::Active,
            "Acknowledged" => AlertStatus::Acknowledged,
            "Resolved" => AlertStatus::Resolved,
            _ => return Err(ApiErrorResponse::bad_request("Invalid status")),
        };
        state.alert_repository.find_by_status(status)
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
    } else if let Some(device_id_str) = query.device_id {
        let device_id = Uuid::parse_str(&device_id_str)
            .map_err(|_| ApiErrorResponse::bad_request("Invalid device_id format"))?;
        let limit = query.limit.unwrap_or(100);
        state.alert_repository.find_by_device(device_id, limit)
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
    } else {
        state.alert_repository.find_active()
            .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?
    };

    let response = AlertListResponse::from(alerts);
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/alerts/{id}",
    params(
        ("id" = String, Path, description = "UUID de la alerta")
    ),
    responses(
        (status = 200, description = "Alerta encontrada", body = AlertResponse),
        (status = 404, description = "Alerta no encontrada")
    ),
    tag = "alerts"
)]
pub async fn get_alert(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<AlertResponse>, ApiErrorResponse> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    let alerts = state.alert_repository.find_active()
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    let alert = alerts.into_iter()
        .find(|a| a.id == uuid)
        .ok_or_else(|| ApiErrorResponse::not_found("Alert"))?;

    Ok(Json(alert.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/alerts",
    request_body = CreateAlertRequest,
    responses(
        (status = 201, description = "Alerta creada", body = AlertResponse),
        (status = 400, description = "Datos inválidos")
    ),
    tag = "alerts"
)]
pub async fn create_alert(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAlertRequest>,
) -> Result<Json<AlertResponse>, ApiErrorResponse> {
    let alert_type = payload.to_alert_type()
        .ok_or_else(|| ApiErrorResponse::bad_request("Invalid alert_type"))?;
    let severity = payload.to_severity()
        .ok_or_else(|| ApiErrorResponse::bad_request("Invalid severity"))?;
    let device_id = if let Some(ref did) = payload.device_id {
        Some(Uuid::parse_str(did)
            .map_err(|_| ApiErrorResponse::bad_request("Invalid device_id format"))?)
    } else {
        None
    };

    let alert = domain::entities::Alert::new(
        alert_type,
        severity,
        device_id,
        payload.message,
        payload.details,
    );

    let created = state.alert_repository.save(&alert)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(created.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/alerts/{id}/acknowledge",
    request_body = AcknowledgeAlertRequest,
    params(
        ("id" = String, Path, description = "UUID de la alerta")
    ),
    responses(
        (status = 200, description = "Alerta reconocida"),
        (status = 404, description = "Alerta no encontrada")
    ),
    tag = "alerts"
)]
pub async fn acknowledge_alert(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<AcknowledgeAlertRequest>,
) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
    let alert_id = Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;
    let user_id = Uuid::parse_str(&payload.user_id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid user_id format"))?;

    state.alert_repository.acknowledge(alert_id, user_id)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Alert acknowledged successfully",
        "alert_id": alert_id.to_string()
    })))
}

#[utoipa::path(
    put,
    path = "/api/v1/alerts/{id}/resolve",
    params(
        ("id" = String, Path, description = "UUID de la alerta")
    ),
    responses(
        (status = 200, description = "Alerta resuelta"),
        (status = 404, description = "Alerta no encontrada")
    ),
    tag = "alerts"
)]
pub async fn resolve_alert(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
    let alert_id = Uuid::parse_str(&id)
        .map_err(|_| ApiErrorResponse::bad_request("Invalid UUID format"))?;

    state.alert_repository.resolve(alert_id)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Alert resolved successfully",
        "alert_id": alert_id.to_string()
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/alerts/stats",
    responses(
        (status = 200, description = "Estadísticas de alertas")
    ),
    tag = "alerts"
)]
pub async fn get_alert_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
    let active = state.alert_repository.find_by_status(AlertStatus::Active)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;
    let acknowledged = state.alert_repository.find_by_status(AlertStatus::Acknowledged)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;
    let resolved = state.alert_repository.find_by_status(AlertStatus::Resolved)
        .map_err(|e| ApiErrorResponse::new(format!("Database error: {}", e)))?;

    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;

    for alert in &active {
        match alert.severity {
            AlertSeverity::Critical => critical_count += 1,
            AlertSeverity::High => high_count += 1,
            AlertSeverity::Medium => medium_count += 1,
            AlertSeverity::Low => low_count += 1,
        }
    }

    Ok(Json(serde_json::json!({
        "total_active": active.len(),
        "total_acknowledged": acknowledged.len(),
        "total_resolved": resolved.len(),
        "active_by_severity": {
            "critical": critical_count,
            "high": high_count,
            "medium": medium_count,
            "low": low_count
        }
    })))
}