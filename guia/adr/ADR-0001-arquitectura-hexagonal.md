# ADR 0001 — Arquitectura Hexagonal y Monolito Modular

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Revisado en** | ADR 0003 (Stack Backend Axum), ADR 0004 (Persistencia PostgreSQL), ADR 0018 (Sintonía CLI), ADR 0020 (Monitoreo Regional) |

---

## Contexto

El primer problema de todo proyecto es elegir cómo organizar el código antes de escribir la primera línea. La tentación de los microservicios es real, pero para un equipo pequeño con un VPS de bajo costo introduce complejidad operacional innecesaria.

El proyecto de monitoreo de infraestructura de red para la Gobernación del Beni necesita:

- Simplicidad operacional
- Bajo consumo de recursos
- Despliegue rápido
- Escalabilidad estructural
- Código mantenible por un solo desarrollador
- Separación estricta entre negocio e infraestructura

Necesitábamos una arquitectura que permitiera evolucionar el sistema sin reescribir el núcleo.

---

## Decisión

Usar un **monolito modular** como unidad de despliegue con **arquitectura hexagonal** como disciplina interna de organización del código.

> Un monorepo bien estructurado vence a microservicios mal diseñados para equipos pequeños.

---

## Las tres capas del modelo

```txt
┌────────────────────────────────────────────┐
│             Adaptadores (afuera)           │
│ HTTP · CLI · Workers · SNMP · Email · S3  │
│                                            │
│ ┌────────────────────────────────────────┐ │
│ │         Puertos (contratos)            │ │
│ │ EscanearRed · GuardarMétrica · Alertar │ │
│ │                                        │ │
│ │ ┌────────────────────────────────────┐ │ │
│ │ │       Dominio (el corazón)         │ │ │
│ │ │ Entidades · Reglas de negocio      │ │ │
│ │ │ No conoce HTTP ni SQL              │ │ │
│ │ │ No conoce frameworks               │ │ │
│ │ │ No conoce async/await              │ │ │
│ │ └────────────────────────────────────┘ │ │
│ └────────────────────────────────────────┘ │
└────────────────────────────────────────────┘
```

Las dependencias siempre apuntan hacia adentro.

El dominio no conoce:

- Axum
- PostgreSQL
- Docker
- HTTP
- SNMP
- SvelteKit
- Tokio
- SQLx

El dominio solo conoce reglas de negocio.

---

## Estructura del monorepo

```txt
redes/
├── apps/
│   ├── api/           # Backend Axum (ADR 0003)
│   ├── web/           # Dashboard SvelteKit 2 + Svelte 5 (ADR 0017)
│   └── agent/         # Agente de monitoreo en sedes remotas (ADR 0022)
│
├── crates/
│   ├── domain/        # Núcleo puro — sin dependencias externas
│   ├── application/   # Casos de uso — solo domain
│   ├── infrastructure/# HTTP, config, utoipa, mailer (Resend)
│   ├── database/      # SQLx + PostgreSQL + repositorios
│   ├── auth/          # PASETO v4 + argon2id (ADR 0008)
│   ├── inventory/     # Inventario de dispositivos (ADR 0020)
│   ├── jobs/          # Apalis — background jobs (ADR 0015)
│   ├── sync/          # Sincronización offline (ADR 0021)
│   ├── snmp/          # Recolección SNMP + ICMP (ADR 0020)
│   ├── topology/      # Grafo de red + análisis (ADR 0020)
│   └── storage/       # S3/assets (ADR 0020)
│
├── data/
│   ├── migrations/    # SQLx migrations (ADR 0005)
│   └── seeds/         # Datos iniciales
│
├── infra/
│   ├── docker/        # Docker Compose (ADR 0013)
│   └── coolify/       # Deploy (ADR 0019)
│
└── justfile           # Comandos de desarrollo (ADR 0012)
```

---

## Diagrama de dependencias entre crates

```txt
                         ┌─────────────┐
                         │   domain    │
                         │ (sin deps   │
                         │  externas)  │
                         └──────┬──────┘
                                │
                                ▼
                         ┌─────────────┐
                         │ application │
                         │ (solo domain)│
                         └──────┬──────┘
                                │
              ┌─────────────────┼─────────────────┐
              │                 │                 │
              ▼                 ▼                 ▼
        ┌─────────┐      ┌──────────┐      ┌──────────┐
        │ database│      │   auth   │      │  storage │
        │(sqlx)   │      │(PASETO)  │      │  (S3)    │
        └────┬────┘      └────┬─────┘      └────┬─────┘
             │                │                 │
             │    ┌───────────┼─────────────────┘
             │    │           │
             ▼    ▼           ▼
        ┌──────────────────────────┐
        │     infrastructure       │
        │ (axum + utoipa + config) │
        └────────────┬─────────────┘
                     │
                     ▼
              ┌─────────────┐
              │  apps/api   │
              │ (ensambla    │
              │  todo)       │
              └─────────────┘
```

**Regla:** Flechas suben. Ningún crate importa uno que esté por encima.

---

## Crates implementados

| Crate | Responsabilidad | Dependencias directas |
|-------|----------------|----------------------|
| `domain` | Entidades, value objects, ports (traits), errores | `thiserror`, `uuid`, `time`, `serde` |
| `application` | Casos de uso (use cases) | `domain` |
| `database` | SQLx, repositorios, migraciones | `domain`, `sqlx`, `moka` |
| `auth` | PASETO v4, argon2id, password hashing | `domain`, `pasetors`, `argon2`, `secrecy` |
| `inventory` | Lógica de inventario de dispositivos | `domain` |
| `jobs` | Apalis — background jobs (alertas, agregación, cleanup) | `domain`, `apalis` |
| `sync` | Sincronización offline, sync queue | `domain`, `tokio`, `serde` |
| `snmp` | Recolección SNMP, ICMP ping, descubrimiento | `domain`, `snmp`, `surge-ping`, `tokio` |
| `topology` | Grafo de red, análisis de conectividad, SPOF | `domain` |
| `storage` | S3/assets, almacenamiento de archivos | `domain`, `aws-config`, `aws-sdk-s3` |
| `infrastructure` | Axum handlers, middleware, config, utoipa, mailer | `application`, `database`, `auth`, `inventory`, `jobs`, `sync`, `snmp`, `topology`, `storage`, `axum`, `utoipa`, `tower`, `tower-http` |

---

## Apps implementadas

| App | Responsabilidad | Dependencias |
|-----|----------------|-------------|
| `api` | Servidor Axum — ensambla todos los crates | `infrastructure`, `database`, `auth`, `storage`, `domain`, `application` |
| `web` | Dashboard SvelteKit 2 SSR con Svelte 5 | Frontend (no depende de crates Rust) |
| `agent` | Agente de monitoreo en sedes remotas | `domain` (tipos compartidos), `snmp`, `sync`, `tokio`, `reqwest`, `rusqlite` |

---

## Reglas de frontera arquitectónica

| Regla | Descripción | Verificación |
|-------|-------------|------------|
| **R1** | `crates/domain` sin dependencias externas | `cargo tree -p domain --depth 1` → solo `thiserror`, `uuid`, `time`, `serde` |
| **R2** | SQL únicamente en `crates/database/repositories/` | `grep -r "sqlx" crates/domain/ --include="*.rs"` → cero resultados |
| **R3** | Lógica de negocio fuera de handlers Axum | Handlers solo delegan a use cases |
| **R4** | Traits del dominio (ports) son síncronos | Sin `async-trait` en `crates/domain` |
| **R5** | Ningún crate depende de uno que esté por encima en el diagrama | `cargo check --workspace` lo valida |

---

## Ejemplo: puerto y adaptador en Rust

```rust
// crates/domain/src/ports/device_repository.rs
// Puerto: contrato del dominio. SIN async.

pub trait DeviceRepository: Send + Sync {
    fn find_by_id(&self, id: &DeviceId) -> Result<Option<Device>, DomainError>;
    fn find_by_sede(&self, sede_id: &SedeId) -> Result<Vec<Device>, DomainError>;
    fn save(&self, device: &Device) -> Result<(), DomainError>;
    fn soft_delete(&self, id: &DeviceId) -> Result<(), DomainError>;
}
```

```rust
// crates/database/src/repositories/device_repository.rs
// Adaptador: implementación con SQLx + PostgreSQL. Async aquí es OK.

use domain::ports::DeviceRepository;

pub struct PostgresDeviceRepository {
    pool: PgPool,
}

impl DeviceRepository for PostgresDeviceRepository {
    async fn find_by_id(&self, id: &DeviceId) -> Result<Option<Device>, DomainError> {
        sqlx::query_as::<_, DeviceRow>(
            "SELECT * FROM devices WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .map(|row| Device::try_from(row))
        .transpose()
    }

    async fn soft_delete(&self, id: &DeviceId) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE devices SET deleted_at = NOW() WHERE id = $1"
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
```

Si mañana PostgreSQL cambia, solo se reemplaza el adaptador. El dominio permanece intacto.

---

## Fases del desarrollo

### Diseño

| Patrón | Rol |
|--------|-----|
| Arquitectura Hexagonal | Separación entre negocio e infraestructura |
| DDD táctico | Entidades y Value Objects |
| Puertos | Contratos del dominio (traits síncronos) |
| Onion Architecture | Dependencias hacia el centro |

### Construcción

| Elemento | Rol |
|----------|-----|
| Adaptadores | Implementaciones reales (async permitido) |
| Traits | Contratos tipados |
| Tipos fuertes | Prevención de errores (Newtype pattern) |
| Rust 2024 | Seguridad y performance |

### Evolución

| Cambio | Impacto |
|--------|---------|
| Cambiar PostgreSQL | Solo cambia `database/` |
| Cambiar Axum | Solo cambia `infrastructure/` y `apps/api/` |
| Cambiar SNMP | Solo cambia `snmp/` |
| Cambiar reglas de negocio | Solo cambia `domain/` |
| Añadir nuevo módulo | Nuevo crate + adaptador en `infrastructure/` |

---

## Herramientas aprobadas

| Herramienta | Propósito |
|-------------|-----------|
| `cargo-deny` | Validación de licencias y seguridad (ADR 0012) |
| `cargo-audit` | Detección de vulnerabilidades en dependencias (ADR 0012) |
| `cargo-nextest` | Tests rápidos y paralelos (ADR 0010) |
| `cargo-watch` | Recompilación automática en desarrollo (ADR 0012) |
| `just` | Comandos de desarrollo (ADR 0012) |
| `lefthook` | Git hooks (pre-commit, pre-push) (ADR 0012) |
| `taplo` | Orden del workspace TOML (ADR 0011) |
| `insta` | Snapshot testing (ADR 0010) |
| `cargo-expand` | Debug de macros |

---

## Mandamientos del proyecto

| Mandamiento | Descripción |
|-------------|-------------|
| Protección del dominio | El negocio nunca depende de infraestructura |
| Simplicidad operacional | Menos servicios, menos problemas |
| Independencia tecnológica | Frameworks reemplazables |
| Código entendible | Diseñado para humanos |
| Escalabilidad estructural | El sistema puede crecer sin reescritura |
| Compilador como guardián | `cargo check` valida la arquitectura |

---

## Alternativas descartadas

| Opción | Motivo |
|--------|--------|
| Microservicios | Complejidad operacional excesiva para equipo pequeño |
| Monolito sin capas | Difícil mantenimiento, acoplamiento total |
| Kubernetes temprano | Innecesario para el tamaño actual (Coolify es suficiente) |
| Event sourcing | Complejidad sin beneficio inmediato |
| NATS JetStream | No requerido; Apalis + PostgreSQL cubre jobs |

---

## Consecuencias

### ✅ Positivas

- Deploy simple (un binario + PostgreSQL)
- Bajo consumo de RAM (< 512MB para API)
- Fácil mantenimiento
- Escalable estructuralmente (añadir crates sin tocar existentes)
- Código testeable (domain sin I/O)
- Separación clara de responsabilidades
- El compilador hace cumplir la arquitectura

### ⚠️ Trade-offs

- Más archivos iniciales que un monolito plano
- Curva de aprendizaje en hexagonal
- Requiere disciplina arquitectónica (no hay policia humana, solo el compilador)
- Agente standalone requiere compilar `domain` dos veces (en API y en agente)

---

## Impacto regional

La arquitectura permite adaptar el sistema a distintos equipos de red sin modificar el núcleo del negocio.

Si la Gobernación cambia:

- switches,
- proveedores,
- protocolos,
- infraestructura,

solo cambian adaptadores externos.

La lógica principal permanece estable.

---

## Resultado esperado

Una base sólida para un sistema de monitoreo regional:

- mantenible,
- simple de operar,
- barata de desplegar,
- fácil de evolucionar,
- resistente a cambios tecnológicos.

---

## Notas de actualización (2026-05-16)

- **Frontend:** Corregida la referencia a "SvelteKit 5" → `SvelteKit 2` con `Svelte 5`. SvelteKit 2.x es el framework full-stack actual; Svelte 5 es la versión del compilador de componentes.
- **Rust 2024 + Axum 0.8:** Con la edición 2024 y Axum 0.8, los traits async nativos de Rust reemplazan la necesidad de `async-trait` en extractores y handlers. Se mantiene `async-trait` solo donde sea estrictamente necesario por compatibilidad con crates que aún lo requieran.
- **SQLx:** Actualizado a la serie 0.8.x estable.
- **Utoipa:** Serie 5.x con soporte OpenAPI 3.1.
- **PASETO v4:** `pasetors` / `paseto-rs` implementan PASETO v4 con XChaCha20-Poly1305 + Ed25519.
