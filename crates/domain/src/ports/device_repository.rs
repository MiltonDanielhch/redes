//! Ubicación: `crates/domain/src/ports/device_repository.rs`
//!
//! Descripción: Puerto para repositorio de dispositivos.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use crate::entities::{Device, DeviceStatus};
use crate::errors::DomainError;

pub trait DeviceRepository: Send + Sync {
    fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Device>, DomainError>;
    fn find_by_sede(&self, sede_id: uuid::Uuid) -> Result<Vec<Device>, DomainError>;
    fn find_by_type(&self, device_type: &str) -> Result<Vec<Device>, DomainError>;
    fn save(&self, device: &Device) -> Result<Device, DomainError>;
    fn update_status(&self, id: uuid::Uuid, status: DeviceStatus) -> Result<(), DomainError>;
    fn soft_delete(&self, id: uuid::Uuid) -> Result<(), DomainError>;
    fn list(&self) -> Result<Vec<Device>, DomainError>;
}