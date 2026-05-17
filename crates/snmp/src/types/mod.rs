//! Ubicación: `crates/snmp/src/types/mod.rs`
//!
//! Descripción: Tipos de datos SNMP.
//!
//! ADRs relacionados: 0020 (Monitoreo Regional)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpMetric {
    pub oid: String,
    pub value: String,
    pub timestamp: time::OffsetDateTime,
}