//! Ubicación: `crates/application/src/services/sede_service.rs`
//!
//! Descripción: Servicio de aplicación para operaciones con Sedes.
//!             Orquesta las operaciones del repositorio de Sedes.
//!
//! ADRs relacionados: 0001 (Hexagonal), 0020 (Monitoreo Regional)

use domain::entities::Sede;
use domain::ports::sede_repository::SedeRepository;
use domain::errors::DomainError;
use uuid::Uuid;

pub struct SedeService<R: SedeRepository> {
    repository: R,
}

impl<R: SedeRepository> SedeService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn get_all_sedes(&self) -> Result<Vec<Sede>, DomainError> {
        self.repository.find_all()
    }

    pub fn get_sede_by_id(&self, id: Uuid) -> Result<Option<Sede>, DomainError> {
        self.repository.find_by_id(id)
    }

    pub fn create_sede(&self, sede: &Sede) -> Result<Sede, DomainError> {
        self.repository.save(sede)
    }
}