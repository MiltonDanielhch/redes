# ADR 0005 — Migraciones SQLx y Seeding Idempotente

| Campo               | Valor                                                             |
| ------------------- | ----------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                        |
| **Fecha**           | 2026                                                              |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                  |
| **Relacionado con** | ADR 0004 (PostgreSQL + Docker), ADR 0001 (Arquitectura Hexagonal) |

---

## Contexto

A medida que el sistema de monitoreo crece,
modificar la base de datos manualmente
introduce riesgos operacionales y pérdida de consistencia.

Necesitamos una estrategia que permita:

* evolucionar el esquema de forma segura,
* mantener sincronizados los entornos,
* automatizar despliegues,
* poblar datos de desarrollo,
* y garantizar reproducibilidad.

El sistema debe poder reconstruirse completamente
desde cero mediante comandos automatizados.

---

## Decisión

Usar:

* SQLx Migrations
* migraciones versionadas,
* seeds idempotentes,
* y automatización mediante CLI/Justfile.

Las migraciones serán ejecutadas automáticamente
durante el arranque de la aplicación.

---

## Estructura aprobada

```txt id="gqtgkp"
data/
├── migrations/
│   ├── 20260101000001_create_users.sql
│   ├── 20260101000002_create_devices.sql
│   └── 20260101000003_create_metrics.sql
│
└── seeds/
    └── development/
```

---

## Migraciones SQLx

```bash id="iibdrx"
# Crear migración
sqlx migrate add create_devices_table

# Ejecutar migraciones
sqlx migrate run

# Ver estado
sqlx migrate info
```

---

## Ejemplo de migración

```sql id="brrjmc"
CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY,
    ip_address TEXT NOT NULL UNIQUE,
    hostname TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_devices_active
ON devices(id)
WHERE deleted_at IS NULL;
```

---

## Migraciones automáticas

```rust id="dgmbcz"
sqlx::migrate!("../../data/migrations")
    .run(&pool)
    .await
    .expect("migraciones fallaron");
```

La aplicación no inicia
si las migraciones fallan.

Esto evita:

* esquemas inconsistentes,
* despliegues corruptos,
* y errores silenciosos.

---

## Seeds de desarrollo

Los seeds existen únicamente para:

* desarrollo,
* testing,
* demos locales.

Nunca se ejecutan automáticamente
en producción.

---

## Ejemplo de seed idempotente

```rust id="xjlwmv"
pub async fn seed_development(
    pool: &PgPool,
) -> Result<(), AppError> {

    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users"
    )
    .fetch_one(pool)
    .await?;

    if count > 0 {
        tracing::info!("seed omitido");
        return Ok(());
    }

    sqlx::query!(
        r#"
        INSERT INTO users (
            id,
            email
        )
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
        Uuid::new_v4(),
        "admin@redes.local"
    )
    .execute(pool)
    .await?;

    Ok(())
}
```

---

## Automatización recomendada

```makefile id="ubrlgo"
db-reset:
    docker compose down -v
    docker compose up -d

    sqlx migrate run

    cargo run --bin cli seed
```

---

## Principios adoptados

| Principio              | Descripción                               |
| ---------------------- | ----------------------------------------- |
| Migraciones inmutables | Nunca modificar migraciones ejecutadas    |
| Reproducibilidad       | La DB puede reconstruirse automáticamente |
| Fail-fast              | El sistema falla si la migración falla    |
| Seeds idempotentes     | Ejecutables múltiples veces               |
| SQL explícito          | Máximo control y visibilidad              |

---

## Corrección de migraciones

Si una migración en producción contiene errores:

```txt id="fjlwmk"
❌ Nunca modificar migraciones ejecutadas

✅ Crear una nueva migración correctiva
```

Esto preserva:

* trazabilidad,
* auditoría,
* consistencia histórica.

---

## Simplificaciones aplicadas

Para mantener simplicidad operacional
NO se incluyen:

* generación automática de schemas,
* ORMs complejos,
* migraciones dinámicas,
* abstracciones enterprise,
* sincronización automática multi-entorno.

El sistema utiliza:

* SQL explícito,
* SQLx,
* PostgreSQL,
* migraciones simples.

---

## Herramientas aprobadas

| Herramienta      | Propósito      |
| ---------------- | -------------- |
| `sqlx-cli`       | Migraciones    |
| `just`           | Automatización |
| `cargo-nextest`  | Tests          |
| `pg_dump`        | Backups        |
| `docker compose` | Entorno local  |

---

## Alternativas descartadas

| Opción               | Motivo                    |
| -------------------- | ------------------------- |
| Prisma ORM           | Abstracción excesiva      |
| Diesel               | Mayor fricción async      |
| Migraciones manuales | Riesgo operacional        |
| Auto-sync de schema  | Riesgo de inconsistencias |

---

## Consecuencias

### ✅ Positivas

* Esquema versionado
* Deploy reproducible
* Evolución controlada
* Integración fuerte con Rust
* Fácil reconstrucción del entorno
* Mayor seguridad operativa

---

### ⚠️ Trade-offs

* Requiere conocer SQL
* Las migraciones incorrectas requieren corrección adicional
* Mayor disciplina en cambios de esquema

---

## Impacto regional

La automatización reduce:

* errores humanos,
* configuraciones inconsistentes,
* problemas en reinstalaciones,
* tiempo de recuperación,
* y soporte remoto innecesario.

Esto es importante para:

* oficinas regionales,
* infraestructura distribuida,
* y mantenimiento simplificado.

---

## Resultado esperado

Un sistema de persistencia:

* reproducible,
* seguro,
* mantenible,
* auditable,
* fácil de desplegar,
* y preparado para evolucionar sin pérdida de consistencia.
