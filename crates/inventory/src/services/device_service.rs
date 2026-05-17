//! Ubicación: `crates/inventory/src/services/device_service.rs`
//!
//! Descripción: Servicio para gestionar dispositivos de inventario.
//!
//! ADRs relacionados: 0020 (Monitoreo Regional)

use crate::entities::{Device, DeviceStatus};

pub struct InventoryService;

impl InventoryService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_device_status(&self, device: &Device) -> DeviceStatus {
        device.status.clone()
    }

    pub fn is_online(&self, device: &Device) -> bool {
        device.status == DeviceStatus::Active
    }
}