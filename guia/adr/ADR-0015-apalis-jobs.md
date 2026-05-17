# ADR 0015 — Jobs Asíncronos con Tokio JoinSet

> **Última revisión:** 2026-05-17
> **Estado:** ✅ Actualizado - Reemplazado Apalis por Tokio JoinSet

| Campo               | Valor                                                                 |
| ------------------- | --------------------------------------------------------------------- |
| **Estado**          | ✅ Actualizado (antes: Aceptado)                                      |
| **Fecha**           | 2026-05-17 (original: 2026-05-16)                                     |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                     |
| **Relacionado con** | ADR 0014 (Monitoreo), ADR 0003 (Stack Backend), ADR 0020 (Monitoreo Regional), ADR 0021 (Local-First Sync) |

---

## Contexto

El sistema necesita procesar tareas largas de forma asíncrona:

* Envío de emails transaccionales
* Procesamiento de alertas SNMP
* Jobs de mantenimiento programados
* Procesamiento de datos de red

El handler HTTP no debe bloquearse esperando estos procesos.

---

## Decisión Original (2026-05-16)

~~Usar **Apalis 1.0** (release candidate) como layer de jobs asíncronos en Rust.~~

## Decisión Actual (2026-05-17)

**Usar Tokio JoinSet + mpsc channel + Semaphore** como alternativa zero-dependency a Apalis.

**Razón del cambio:** Apalis 1.0 RC no tiene features postgres disponibles en la versión rc.9, bloqueando la implementación.

**Ventajas de Tokio JoinSet:**
- Zero-dependency extra (solo usa tokio)
- Zero-cost abstraction
- Back-pressure nativo con Semaphore
- Integración directa con Tokio runtime
- Simplicidad - sin capas de abstracción innecesarias

**Limitaciones:**
- Solo para jobs locales (no distribuido)
- No tiene retry built-in (se puede agregar manualmente)
- No tiene scheduling built-in (se puede usar tokio::time)

> **Nota:** Para sistemas que requieran jobs distribuidos entre múltiples nodos, considerar pgmq o rediq (cola Postgres).

---

## Implementación

**Dependencias en `crates/jobs/Cargo.toml`:**

```toml
[dependencies]
domain = { path = "../domain" }
uuid = { workspace = true }
serde = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }
```

---

## Arquitectura

```text
Request HTTP
     ↓
Enqueue Job → JobQueue (mpsc channel + in-memory)
     ↓
Worker Loop (tokio::spawn + JoinSet)
     ↓
Semaphore (back-pressure, max_concurrent)
     ↓
Procesamiento async
```

---

## Jobs del Sistema

| Job | Prioridad | Retry | Timeout | Descripción |
| ----------------- | --------- | ----- | ------- | ------------------------------ |
| EmailJob | Alta | 3 | 30s | Envío de emails transaccionales |
| CleanupJob | Baja | 1 | 5min | Mantenimiento y limpieza |

## Jobs de Monitoreo (ADR 0020)

| Job | Prioridad | Retry | Timeout | Descripción |
| ---------------------- | --------- | ----- | ------- | ---------------------------------------- |
| MetricsAggregationJob | Alta | 3 | 2min | Agregación de métricas por hora/día |
| AlertDispatchJob | Crítica | 5 | 30s | Envío de alertas por email/Telegram |
| IntrusionDetectionJob | Crítica | 5 | 2min | Análisis de anomalías y detección |
| CleanupMetricsJob | Baja | 1 | 5min | Limpieza de métricas antiguas (30 días) |
| SyncOfflineJob | Media | 2 | 5min | Sincronización offline (ADR 0021) |

---

## Configuración del Worker

```rust
//! Ubicación: `crates/jobs/src/worker.rs`
//!
//! Descripción: Worker de Apalis con PostgreSQL storage, Tower middleware,
//!              tracing integration y graceful shutdown.
//!
//! ADRs: 0015, 0014, 0020

use apalis::prelude::*;
use apalis_postgres::PostgresStorage;
use tower::{retry::RetryLayer, timeout::TimeoutLayer, limit::ConcurrencyLimitLayer};
use std::time::Duration;
use tracing::{info, span, Level, Instrument};

pub async fn run_worker(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Storage PostgreSQL para jobs
    let storage = PostgresStorage::new(pool).await?;

    // Monitor que gestiona todos los workers
    let monitor = Monitor::new()
        // Worker de emails
        .register(
            WorkerBuilder::new("email-worker")
                .concurrency(2)
                .backend(storage.clone())
                .layer(RetryLayer::new(Default::default()))
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(ConcurrencyLimitLayer::new(5))
                .build_fn(email_job_handler)
        )
        // Worker de métricas (cron cada hora)
        .register(
            WorkerBuilder::new("metrics-worker")
                .concurrency(1)
                .backend(storage.clone())
                .layer(RetryLayer::new(Default::default()))
                .layer(TimeoutLayer::new(Duration::from_secs(120)))
                .build_fn(metrics_aggregation_handler)
        )
        // Worker de alertas
        .register(
            WorkerBuilder::new("alert-worker")
                .concurrency(2)
                .backend(storage.clone())
                .layer(RetryLayer::new(Default::default()))
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(RateLimitLayer::new(10, Duration::from_secs(60))) // 10/min para Resend
                .build_fn(alert_dispatch_handler)
        )
        // Worker de intrusiones
        .register(
            WorkerBuilder::new("intrusion-worker")
                .concurrency(1)
                .backend(storage.clone())
                .layer(RetryLayer::new(Default::default()))
                .layer(TimeoutLayer::new(Duration::from_secs(120)))
                .build_fn(intrusion_detection_handler)
        )
        // Worker de cleanup
        .register(
            WorkerBuilder::new("cleanup-worker")
                .concurrency(1)
                .backend(storage.clone())
                .layer(TimeoutLayer::new(Duration::from_secs(300)))
                .build_fn(cleanup_metrics_handler)
        )
        // Worker de sync offline (ADR 0021)
        .register(
            WorkerBuilder::new("sync-worker")
                .concurrency(1)
                .backend(storage.clone())
                .layer(RetryLayer::new(Default::default()))
                .layer(TimeoutLayer::new(Duration::from_secs(300)))
                .build_fn(sync_offline_handler)
        );

    // Graceful shutdown con señales
    monitor.run().await?;

    Ok(())
}
```

---

## Handlers de Jobs

```rust
//! Ubicación: `crates/jobs/src/handlers/email.rs`
//!
//! Descripción: Handler para envío de emails transaccionales.
//!
//! ADRs: 0015, 0016

use apalis::prelude::*;
use tracing::{info, error, instrument};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailJob {
    pub to: String,
    pub template: String,
    pub vars: serde_json::Value,
}

#[instrument(skip(job, ctx), fields(job_id = %ctx.id()))]
pub async fn email_job_handler(job: EmailJob, ctx: JobContext) -> Result<(), Error> {
    info!(to = %job.to, template = %job.template, "processing email job");

    // Lógica de envío via Resend (ADR 0016)
    // ...

    info!("email sent successfully");
    Ok(())
}
```

---

## Cron Jobs (apalis-cron)

```rust
//! Ubicación: `crates/jobs/src/cron.rs`
//!
//! Descripción: Jobs programados con cron expressions.
//!
//! ADRs: 0015, 0020

use apalis_cron::{CronJob, Schedule};
use chrono::Utc;
use std::str::FromStr;

// Métricas: cada hora en el minuto 0
pub fn metrics_schedule() -> Schedule {
    Schedule::from_str("0 0 * * * *").unwrap() // Cada hora
}

// Cleanup: cada día a las 3 AM
pub fn cleanup_schedule() -> Schedule {
    Schedule::from_str("0 0 3 * * *").unwrap() // 3 AM diario
}

// Backup verification: cada hora
pub fn backup_schedule() -> Schedule {
    Schedule::from_str("0 0 * * * *").unwrap()
}
```

---

## Enqueue de Jobs desde HTTP

```rust
//! Ubicación: `crates/jobs/src/enqueue.rs`
//!
//! Descripción: Helper para encolar jobs desde handlers HTTP.
//!
//! ADRs: 0015, 0003

use apalis::prelude::*;
use apalis_postgres::PostgresStorage;

pub async fn enqueue_email(
    storage: &PostgresStorage<EmailJob>,
    job: EmailJob,
) -> Result<String, Error> {
    let job_id = storage.push(job).await?;
    Ok(job_id.to_string())
}
```

---

## Monitoreo con Healthchecks.io (ADR 0014)

```rust
//! Ubicación: `crates/jobs/src/healthcheck.rs`
//!
//! Descripción: Heartbeat de worker para Healthchecks.io.
//!
//! ADRs: 0015, 0014

use std::time::Duration;
use tokio::time::interval;
use crate::monitoring::healthchecks;

pub async fn worker_heartbeat(hc_url: String) {
    let mut ticker = interval(Duration::from_secs(300)); // 5 min

    loop {
        ticker.tick().await;
        healthchecks::ping(&hc_url, "apalis_worker").await;
    }
}
```

---

## Graceful Shutdown

```rust
//! Ubicación: `apps/api/src/main.rs` (extracto)
//!
//! Descripción: Manejo de señales para shutdown graceful del worker.
//!
//! ADRs: 0015

use tokio::signal;

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received, stopping workers...");
}
```

---

## Justfile (extracto)

```makefile
# Jobs / Worker
worker:
    cargo run --bin worker

worker-logs:
    docker compose logs -f worker

# Encolar job de prueba
enqueue-test:
    curl -X POST http://localhost:8080/api/v1/jobs/test         -H "Content-Type: application/json"         -d '{"type":"email","to":"test@example.com"}'
```

---

## Tabla de Jobs vs Storage

| Job | Storage | Cron | Priority | Concurrency |
| ---------------------- | --------- | ---- | ---------- | ----------- |
| EmailJob | PostgreSQL | No | Normal | 2 |
| MetricsAggregationJob | PostgreSQL | Sí (hora) | High | 1 |
| AlertDispatchJob | PostgreSQL | No | Critical | 2 |
| IntrusionDetectionJob | PostgreSQL | Sí (5min) | Critical | 1 |
| CleanupMetricsJob | PostgreSQL | Sí (3AM) | Low | 1 |
| SyncOfflineJob | PostgreSQL | Sí (5min) | Normal | 1 |

---

## Alternativas consideradas

| Opción | Motivo de descarte |
| --------------------- | --------------------------- |
| Implementación manual | Complejidad innecesaria |
| Celery (Python) | Stack incompatible |
| Sidekiq (Ruby) | Stack incompatible |
| Bull (Node.js) | Stack incompatible |
| Faktory | Requiere infra adicional |
| Beanstalkd | Menor ecosistema Rust |

---

## Herramientas y Librerías (Edición 2026)

| Herramienta | Propósito | Versión | Estado |
| ---------------- | -------------------------------- | ------------ | ------ |
| `apalis` | Framework de jobs | 1.0.0-rc.7 | ✅ Activa (RC) |
| `apalis-postgres` | Storage PostgreSQL | 1.0.0-rc.7 | ✅ Activa (RC) |
| `apalis-sqlite` | Storage SQLite (dev) | 1.0.0-rc.7 | 🟡 Dev only |
| `apalis-cron` | Scheduling cron | 1.0.0-rc.7 | ✅ Activa (RC) |
| `apalis-board` | Web UI para monitoreo | 1.0.0-rc.8 | 🟡 Opcional |
| `tower` | Middleware (retry, timeout) | 0.5.3 | ✅ Activa |
| `tokio` | Runtime async | 1.52 | ✅ Activa |
| `tracing` | Observabilidad | workspace | ✅ Activa |

---

## Consecuencias

### ✅ Positivas

* Procesamiento asíncrono sin bloquear HTTP
* Retry automático con backoff exponencial
* Priorities integradas
* Cron jobs nativos
* Web UI opcional para monitoreo
* Compatible con PostgreSQL del proyecto
* Tower middleware (retry, timeout, rate limit, concurrency)
* Tracing integration nativa
* Graceful shutdown

### ⚠️ Negativas / Trade-offs

**API inestable (RC)**

Apalis 1.0 aún no es estable. Puede haber breaking changes entre RCs.

**Mitigación**

* Fijar versión exacta en `Cargo.toml` (`=1.0.0-rc.7`)
* Revisar changelog antes de actualizar
* Encapsular toda la lógica en `crates/jobs/`

**Mayor complejidad que procesamiento síncrono**

Mitigación:
* Tests exhaustivos
* Monitoreo con Healthchecks.io
* Logs estructurados

---

## Decisiones derivadas

* `apalis 1.0.0-rc.7` es la versión oficial de jobs
* `apalis-postgres` es el storage de producción
* `apalis-sqlite` para desarrollo local
* `apalis-cron` para jobs programados
* Cada worker tiene nombre descriptivo (`email-worker`, `metrics-worker`)
* Concurrency limitada a 1-2 por worker (VPS $5)
* Timeout obligatorio para todos los jobs
* Retry con backoff exponencial
* Rate limit para `AlertDispatchJob` (Resend)
* Tracing span por job con `job_id`, `job_type`, `attempt`
* Healthchecks.io heartbeat cada 5 minutos
* Graceful shutdown con señales SIGTERM/SIGINT
* `crates/jobs/` encapsula toda la lógica de jobs
* `tokio 1.52` es la versión mínima para el runtime de jobs
* `tower 0.5.3` es la versión mínima para middleware

---

## Registro de cambios de versiones

| Fecha | Componente | Anterior | Actual | Notas |
|-------|------------|----------|--------|-------|
| 2026-05-16 | apalis | 1.0.0-rc.9 | **1.0.0-rc.7** | Última RC real (abr 2026). rc.9 no existe en crates.io. |
| 2026-05-16 | apalis-postgres | 1.0.0-rc.9 | **1.0.0-rc.7** | Alineado con apalis core. |
| 2026-05-16 | apalis-sqlite | 1.0.0-rc.9 | **1.0.0-rc.7** | Alineado con apalis core. |
| 2026-05-16 | apalis-cron | 1.0.0-rc.9 | **1.0.0-rc.7** | Alineado con apalis core. |
| 2026-05-16 | apalis-board | 1.0.0-rc.9 | **1.0.0-rc.8** | Última RC (may 2026). |
| 2026-05-16 | tokio | 1.45 | **1.52** | Runtime actualizado. Última: 1.52.3 (may 2026). |
| 2026-05-16 | tower | 0.5 | **0.5.3** | Última estable. Middleware retry/timeout/limit. |
