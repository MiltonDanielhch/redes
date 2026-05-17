//! Ubicación: `crates/infrastructure/src/dto/alert.rs`
//!
//! Descripción: DTOs para requests y responses de alertas.
//!
//! ADRs relacionados: 0003 (Axum), 0020 (Monitoreo Regional)

use domain::entities::{Alert, AlertType, AlertSeverity};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlertRequest {
    pub alert_type: String,
    pub severity: String,
    pub device_id: Option<String>,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcknowledgeAlertRequest {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlertResponse {
    pub id: String,
    pub alert_type: String,
    pub severity: String,
    pub device_id: Option<String>,
    pub message: String,
    pub details: Option<String>,
    pub status: String,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Alert> for AlertResponse {
    fn from(alert: Alert) -> Self {
        Self {
            id: alert.id.to_string(),
            alert_type: format!("{:?}", alert.alert_type),
            severity: format!("{:?}", alert.severity),
            device_id: alert.device_id.map(|id| id.to_string()),
            message: alert.message,
            details: alert.details,
            status: format!("{:?}", alert.status),
            acknowledged_by: alert.acknowledged_by.map(|id| id.to_string()),
            acknowledged_at: alert.acknowledged_at.map(|dt| dt.to_string()),
            created_at: alert.created_at.to_string(),
            updated_at: alert.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlertListResponse {
    pub alerts: Vec<AlertResponse>,
    pub total: usize,
}

impl From<Vec<Alert>> for AlertListResponse {
    fn from(alerts: Vec<Alert>) -> Self {
        let total = alerts.len();
        Self {
            alerts: alerts.into_iter().map(|a| a.into()).collect(),
            total,
        }
    }
}

impl CreateAlertRequest {
    pub fn to_alert_type(&self) -> Option<AlertType> {
        match self.alert_type.as_str() {
            "DeviceOffline" => Some(AlertType::DeviceOffline),
            "BandwidthSaturation" => Some(AlertType::BandwidthSaturation),
            "PacketLoss" => Some(AlertType::PacketLoss),
            "Intrusion" => Some(AlertType::Intrusion),
            "TopologyChange" => Some(AlertType::TopologyChange),
            "HighTraffic" => Some(AlertType::HighTraffic),
            _ => None,
        }
    }

    pub fn to_severity(&self) -> Option<AlertSeverity> {
        match self.severity.as_str() {
            "Critical" => Some(AlertSeverity::Critical),
            "High" => Some(AlertSeverity::High),
            "Medium" => Some(AlertSeverity::Medium),
            "Low" => Some(AlertSeverity::Low),
            _ => None,
        }
    }
}