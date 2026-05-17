//! Ubicación: `crates/domain/src/entities/sede.rs`
//!
//! Descripción: Entidad de dominio `Sede` que representa una sederegional de la
//!              Gobernación del Beni. Implementa Soft Delete (ADR 0006).
//!
//! ADRs relacionados: 0001 (Hexagonal), 0006 (RBAC), 0020 (Monitoreo Regional)

use uuid::Uuid;
use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

/// Entidad Sede - representa una sede regional de la Gobernación del Beni
///
/// # Campos
/// - `id`: Identificador único UUID v4
/// - `nombre`: Nombre de la sede
/// - `ubicación`: Dirección física de la sede
/// - `secretaría`: Secretaría a la que pertenece
/// - `created_at`: Timestamp de creación
/// - `updated_at`: Timestamp de última modificación
/// - `deleted_at`: Timestamp de eliminación (Soft Delete)
///
/// # Ejemplos
/// ```
/// let sede = Sede::new("Sede Central".to_string(), "Riberalta".to_string(), "Gobernación".to_string());
/// assert!(sede.nombre == "Sede Central");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sede {
    pub id: Uuid,
    pub nombre: String,
    pub ubicacion: String,
    pub secretaria: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl Sede {
    /// Crea una nueva sede con ID generado automáticamente
    pub fn new(nombre: String, ubicacion: String, secretaria: String) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            nombre,
            ubicacion,
            secretaria,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Marca la sede como eliminada (Soft Delete)
    pub fn mark_deleted(&mut self) {
        self.deleted_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Verifica si la sede está eliminada
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}