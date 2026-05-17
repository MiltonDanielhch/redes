//! Ubicación: `crates/domain/src/ports/mod.rs`
//!
//! Descripción: Módulo de puertos (traits) que definen contratos entre capas.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0006 (RBAC), 0008 (PASETO), 0020 (Monitoreo Regional)

pub mod alert_repository;
pub mod audit_repository;
pub mod device_repository;
pub mod intrusion_repository;
pub mod metrics_repository;
pub mod sede_repository;
pub mod session_repository;
pub mod token_repository;
pub mod user_repository;

pub use alert_repository::AlertRepository;
pub use audit_repository::AuditRepository;
pub use device_repository::DeviceRepository;
pub use intrusion_repository::IntrusionRepository;
pub use metrics_repository::MetricsRepository;
pub use sede_repository::SedeRepository;
pub use session_repository::SessionRepository;
pub use token_repository::TokenRepository;
pub use user_repository::UserRepository;