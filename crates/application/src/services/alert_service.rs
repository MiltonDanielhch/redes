//! Ubicación: `crates/application/src/services/alert_service.rs`
//!
//! Descripción: Servicio de aplicación para operaciones con Alertas.
//!              Orquesta la lógica de detección y gestión de alertas.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use domain::entities::{Alert, AlertType, AlertSeverity, AlertStatus, Device, DeviceStatus};
use domain::ports::AlertRepository;
use domain::errors::DomainError;
use uuid::Uuid;
use std::sync::Arc;

pub struct AlertService<R: AlertRepository> {
    repository: R,
}

impl<R: AlertRepository> AlertService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn create_alert(
        &self,
        alert_type: AlertType,
        severity: AlertSeverity,
        device_id: Option<Uuid>,
        message: String,
        details: Option<String>,
    ) -> Result<Alert, DomainError> {
        let alert = Alert::new(alert_type, severity, device_id, message, details);
        self.repository.save(&alert)?;
        Ok(alert)
    }

    pub fn get_active_alerts(&self) -> Result<Vec<Alert>, DomainError> {
        self.repository.find_active()
    }

    pub fn acknowledge_alert(&self, alert_id: Uuid, user_id: Uuid) -> Result<(), DomainError> {
        self.repository.acknowledge(alert_id, user_id)
    }

    pub fn resolve_alert(&self, alert_id: Uuid) -> Result<(), DomainError> {
        self.repository.resolve(alert_id)
    }

    pub fn get_alerts_by_device(&self, device_id: Uuid, limit: usize) -> Result<Vec<Alert>, DomainError> {
        self.repository.find_by_device(device_id, limit)
    }

    pub fn get_alerts_by_status(&self, status: AlertStatus) -> Result<Vec<Alert>, DomainError> {
        self.repository.find_by_status(status)
    }
}

pub struct AlertDetector {
    device_offline_threshold_secs: i64,
    bandwidth_saturation_threshold: f64,
    packet_loss_threshold: f64,
}

impl Default for AlertDetector {
    fn default() -> Self {
        Self {
            device_offline_threshold_secs: 300,
            bandwidth_saturation_threshold: 0.90,
            packet_loss_threshold: 0.05,
        }
    }
}

impl AlertDetector {
    pub fn new(
        device_offline_threshold_secs: i64,
        bandwidth_saturation_threshold: f64,
        packet_loss_threshold: f64,
    ) -> Self {
        Self {
            device_offline_threshold_secs,
            bandwidth_saturation_threshold,
            packet_loss_threshold,
        }
    }

    pub fn check_device_offline(&self, device: &Device, last_seen_secs_ago: i64) -> Option<Alert> {
        if last_seen_secs_ago > self.device_offline_threshold_secs && device.status != DeviceStatus::Offline {
            Some(Alert::new(
                AlertType::DeviceOffline,
                AlertSeverity::High,
                Some(device.id),
                format!("Device {} is offline", device.hostname),
                Some(format!("Last seen {} seconds ago", last_seen_secs_ago)),
            ))
        } else {
            None
        }
    }

    pub fn check_bandwidth_saturation(&self, device: &Device, utilization: f64) -> Option<Alert> {
        if utilization > self.bandwidth_saturation_threshold {
            Some(Alert::new(
                AlertType::BandwidthSaturation,
                AlertSeverity::Medium,
                Some(device.id),
                format!("Device {} bandwidth saturation at {:.1}%", device.hostname, utilization * 100.0),
                Some(format!("Utilization: {:.1}%", utilization * 100.0)),
            ))
        } else {
            None
        }
    }

    pub fn check_packet_loss(&self, device: &Device, packet_loss: f64) -> Option<Alert> {
        if packet_loss > self.packet_loss_threshold {
            let severity = if packet_loss > 0.20 {
                AlertSeverity::Critical
            } else if packet_loss > 0.10 {
                AlertSeverity::High
            } else {
                AlertSeverity::Medium
            };

            Some(Alert::new(
                AlertType::PacketLoss,
                severity,
                Some(device.id),
                format!("Device {} packet loss detected: {:.1}%", device.hostname, packet_loss * 100.0),
                Some(format!("Packet loss: {:.1}%", packet_loss * 100.0)),
            ))
        } else {
            None
        }
    }

    pub fn check_intrusion(&self, device: &Device, unknown_macs: &[String]) -> Option<Alert> {
        if !unknown_macs.is_empty() {
            Some(Alert::new(
                AlertType::Intrusion,
                AlertSeverity::Critical,
                Some(device.id),
                format!("Possible intrusion on device {}: {} unknown MACs", device.hostname, unknown_macs.len()),
                Some(format!("Unknown MACs: {:?}", unknown_macs)),
            ))
        } else {
            None
        }
    }
}

pub struct AlertServiceWithDetector<R: AlertRepository> {
    inner: AlertService<R>,
    detector: Arc<AlertDetector>,
}

impl<R: AlertRepository> AlertServiceWithDetector<R> {
    pub fn new(repository: R, detector: AlertDetector) -> Self {
        Self {
            inner: AlertService::new(repository),
            detector: Arc::new(detector),
        }
    }

    pub fn create_alert(&self, alert: Alert) -> Result<Alert, DomainError> {
        self.inner.repository.save(&alert)?;
        Ok(alert)
    }

    pub fn get_active_alerts(&self) -> Result<Vec<Alert>, DomainError> {
        self.inner.get_active_alerts()
    }

    pub fn acknowledge_alert(&self, alert_id: Uuid, user_id: Uuid) -> Result<(), DomainError> {
        self.inner.acknowledge_alert(alert_id, user_id)
    }

    pub fn resolve_alert(&self, alert_id: Uuid) -> Result<(), DomainError> {
        self.inner.resolve_alert(alert_id)
    }

    pub fn detect_and_create_offline_alert(&self, device: &Device, last_seen_secs_ago: i64) -> Result<Option<Alert>, DomainError> {
        if let Some(alert) = self.detector.check_device_offline(device, last_seen_secs_ago) {
            self.inner.repository.save(&alert)?;
            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    pub fn detect_and_create_bandwidth_alert(&self, device: &Device, utilization: f64) -> Result<Option<Alert>, DomainError> {
        if let Some(alert) = self.detector.check_bandwidth_saturation(device, utilization) {
            self.inner.repository.save(&alert)?;
            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    pub fn detect_and_create_packet_loss_alert(&self, device: &Device, packet_loss: f64) -> Result<Option<Alert>, DomainError> {
        if let Some(alert) = self.detector.check_packet_loss(device, packet_loss) {
            self.inner.repository.save(&alert)?;
            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }
}