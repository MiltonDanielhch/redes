//! Ubicación: `crates/domain/src/entities/mod.rs`
//!
//! Descripción: Módulo de entidades de dominio.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0006 (RBAC), 0008 (PASETO), 0020 (Monitoreo Regional)

mod alert;
mod audit_log;
mod device;
mod device_link;
mod intrusion;
mod metric;
mod role;
mod sede;
mod session;
mod token;
mod user;

pub use alert::{Alert, AlertSeverity, AlertStatus, AlertType};
pub use audit_log::AuditLog;
pub use device::{Device, DeviceStatus, DeviceType};
pub use device_link::{DeviceLink, LinkType};
pub use intrusion::{IntrusionEvent, IntrusionStatus};
pub use metric::MetricReading;
pub use role::Role;
pub use sede::Sede;
pub use session::Session;
pub use token::{Token, TokenPurpose};
pub use user::User;