//! Ubicación: `crates/database/src/repositories/sede_repository.rs`
//!
//! Descripción: Implementación PostgreSQL del repositorio de Sedes.
//!              Utiliza SQLx para operaciones de base de datos.
//!
//! ADRs relacionados: 0004 (PostgreSQL), 0020 (Monitoreo Regional)

use domain::entities::Sede;
use domain::ports::sede_repository::SedeRepository;
use domain::errors::DomainError;
use uuid::Uuid;

pub struct PostgresSedeRepository {
    pool: sqlx::PgPool,
}

impl PostgresSedeRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

impl SedeRepository for PostgresSedeRepository {
    fn find_all(&self) -> Result<Vec<Sede>, DomainError> {
        todo!("Implementar con sqlx - SELECT * FROM sedes WHERE deleted_at IS NULL")
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<Sede>, DomainError> {
        todo!("Implementar con sqlx - SELECT * FROM sedes WHERE id = $1 AND deleted_at IS NULL")
    }

    fn save(&self, sede: &Sede) -> Result<Sede, DomainError> {
        todo!("Implementar con sqlx - INSERT INTO sedes")
    }

    fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        todo!("Implementar con sqlx - UPDATE sedes SET deleted_at = NOW() WHERE id = $1 (Soft Delete)")
    }
}