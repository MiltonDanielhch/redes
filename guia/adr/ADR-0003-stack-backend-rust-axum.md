# ADR 0003 — Stack Backend: Rust 2024 + Axum 0.8 + Tokio

> **Última revisión de versiones:** 2026-05-16  
> Se actualizaron las versiones de dependencias tras auditoría contra crates.io y docs.rs.

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0002 (Configuración), ADR 0004 (PostgreSQL), ADR 0008 (PASETO), ADR 0009 (Rate Limiting), ADR 0013 (Docker Deploy), ADR 0016 (OpenAPI), ADR 0020 (Monitoreo Regional) |

---

## Contexto

El sistema de monitoreo de infraestructura de red de la Gobernación del Beni
necesita un backend capaz de:

* manejar múltiples conexiones concurrentes,
* consumir pocos recursos,
* responder rápidamente,
* ejecutarse en VPS pequeños,
* y mantenerse estable durante largos periodos de operación.

El stack backend debe cumplir:

* alto rendimiento,
* seguridad de memoria,
* bajo consumo de RAM,
* concurrencia eficiente,
* binario único,
* despliegue simple,
* compatibilidad con Docker.

---

## Decisión

Usar:

* Rust Edition 2024
* Axum 0.8
* Tokio 1.52+

como stack principal del backend.

Protocolos de comunicación aprobados:
* **REST** — API principal (JSON)
* **SSE** — Realtime push al frontend (ADR 0017)
* **HTTP/1.1** — Comunicación agente→servidor (ADR 0022)

**No usar:** gRPC, WebSocket (salvo excepción justificada), GraphQL, Kafka, NATS.

---

## Motivos principales

### Rust 2024

Rust permite:

* seguridad de memoria sin garbage collector,
* ausencia de data races,
* alta performance,
* bajo consumo de recursos,
* binarios pequeños y estáticos.

Esto es importante para infraestructura regional
con recursos limitados.

---

### Axum 0.8

Axum proporciona:

* integración nativa con Tokio,
* middleware basado en Tower (composable, ordenable),
* tipado fuerte,
* handlers simples,
* arquitectura modular,
* excelente compatibilidad con Rust moderno,
* soporte nativo para SSE (Server-Sent Events).

Encaja correctamente con la arquitectura hexagonal del ADR 0001.

---

### Tokio 1.52+

Tokio proporciona:

* runtime asíncrono eficiente,
* concurrencia cooperativa,
* tareas ligeras (green threads),
* networking de alto rendimiento,
* ecosistema maduro y estable.

---

## Stack aprobado (Cargo.toml)

```toml
[dependencies]
# ── Runtime ──────────────────────────────────────────────
tokio = { version = "1.52", features = [
    "rt-multi-thread",
    "macros",
    "signal",
    "time",
] }

# ── Web Framework ─────────────────────────────────────────
axum = { version = "0.8", features = ["macros"] }
axum-extra = { version = "0.12", features = [
    "typed-header",
    "query",
    "cookie",
] }

# ── Middleware ────────────────────────────────────────────
tower = "0.5.2"
tower-http = { version = "0.6.10", features = [
    "cors",
    "trace",
    "timeout",
    "compression-gzip",
    "compression-brotli",
    "request-id",
    "limit",
    "validate-request",
] }

# ── Serialización ─────────────────────────────────────────
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# ── Observabilidad ────────────────────────────────────────
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = [
    "env-filter",
    "json",
    "fmt",
] }
tracing-opentelemetry = { version = "0.30", optional = true }

# ── Base de datos ───────────────────────────────────────
sqlx = { version = "0.8.6", features = [
    "runtime-tokio-rustls",
    "postgres",
    "macros",
    "migrate",
    "chrono",
    "uuid",
] }

# ── Cache ─────────────────────────────────────────────────
moka = { version = "0.12.15", features = ["future"] }

# ── Auth / Seguridad ────────────────────────────────────
pasetors = "0.7"
argon2 = "0.5"
secrecy = "0.10"

# ── HTTP Client (para servicios externos) ─────────────────
# NOTA: reqwest 0.13 introduce rustls como default.
# La feature "rustls-tls" fue renombrada a "rustls".
# "query" y "form" son ahora features opcionales.
reqwest = { version = "0.13", features = [
    "json",
    "gzip",
    "rustls",
] }

# ── OpenAPI / Documentación ─────────────────────────────
utoipa = { version = "5.4", features = ["axum_extras"] }
utoipa-scalar = { version = "0.4", features = ["axum"] }

# ── Validación ───────────────────────────────────────────
validator = { version = "0.20", features = ["derive"] }

# ── Email ─────────────────────────────────────────────────
# Resend se usa vía reqwest (HTTP API), no necesita crate adicional

# ── Utilidades ────────────────────────────────────────────
uuid = { version = "1.15", features = ["v7", "serde"] }
time = { version = "0.3", features = ["serde", "formatting", "parsing"] }
thiserror = "2"
anyhow = "1"  # Solo para bins, no para libs
config = "0.15"
dotenvy = "0.15"

# ── Background Jobs ──────────────────────────────────────
# NOTA: apalis 1.0 se encuentra en release candidate (rc.9).
# Para producción estable, validar antes de migrar desde 0.7.
apalis = { version = "1.0", features = ["sqlx", "postgres"] }

# ── Testing ───────────────────────────────────────────────
[dev-dependencies]
cargo-nextest = "0.9.135"
axum-test = "17"  # Helper para testing de handlers Axum
mockall = "0.13"
```

---

## Middleware en orden correcto

> **Regla:** El orden de los layers en Tower es **de abajo hacia arriba** (el último `.layer()` es el primero en procesar la request).
> 
> Orden recomendado:
> 1. `trace` — logging base con request_id
> 2. `request_id` — generar/propagar X-Request-Id
> 3. `cors` — Cross-Origin Resource Sharing (antes de auth para preflight)
> 4. `rate_limit` — protección contra abuse (ADR 0009)
> 5. `timeout` — límite de tiempo por request
> 6. `compression` — compresión de respuesta (último, sobre body final)

```rust
use axum::{
    Router,
    ServiceExt,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
    compression::CompressionLayer,
    request_id::SetRequestIdLayer,
};
use std::time::Duration;

let app = Router::new()
    .merge(api_router())
    .with_state(state)
    .layer(
        ServiceBuilder::new()
            // 1. Tracing — logging base con request_id
            .layer(TraceLayer::new_for_http())

            // 2. Request ID — generar/propagar X-Request-Id
            .layer(SetRequestIdLayer::x_request_id(
                MakeRequestUuid
            ))

            // 3. CORS — antes de rate limit para preflight requests
            .layer(cors_layer)

            // 4. Rate Limiting — protección contra abuse (ADR 0009)
            // NOTA: RateLimitLayer requiere implementación custom o crate externo
            // Ejemplo con tower_governor o implementación propia:
            .layer(rate_limit_layer)

            // 5. Timeout — límite de tiempo por request
            .layer(
                TimeoutLayer::new(
                    Duration::from_secs(30)
                )
            )

            // 6. Compresión — último, sobre el body final
            .layer(CompressionLayer::new())
    );
```

---

## Rate Limiting (ADR 0009)

```rust
use std::num::NonZeroU32;
use tower_governor::{GovernorLayer, GovernorConfigBuilder};

// Configuración por endpoint
let governor_config = GovernorConfigBuilder::default()
    .per_second(60)           // 60 requests por minuto
    .burst_size(NonZeroU32::new(10).unwrap())  // burst de 10
    .finish()
    .expect("Configuración de rate limiting inválida");

let rate_limit_layer = GovernorLayer {
    config: Arc::new(governor_config),
};
```

**Headers de respuesta:**
- `X-RateLimit-Limit`: límite por ventana
- `X-RateLimit-Remaining`: requests restantes
- `X-RateLimit-Reset`: timestamp de reset
- `Retry-After`: segundos hasta próximo request permitido (en 429)

---

## Principios adoptados

| Principio | Descripción |
|-----------|-------------|
| Binario único | Deploy simple (un binario + PostgreSQL) |
| Runtime eficiente | Menor consumo de RAM (< 512MB para API) |
| Async real | Miles de conexiones concurrentes con green threads |
| Seguridad | Memory safety sin GC, sin data races |
| Middleware composable | Capas desacopladas, ordenables, testeables |
| Compile-time safety | Errores detectados antes de producción |
| REST + SSE | Protocolos simples, compatibles con firewalls institucionales |
| Fail-fast | Panic en startup si config inválida (ADR 0002) |

---

## Graceful shutdown completo

```rust
use tokio::signal;
use tracing::info;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info("Recibido Ctrl+C, iniciando shutdown graceful..."),
        _ = terminate => info("Recibido SIGTERM, iniciando shutdown graceful..."),
    }
}

// En main.rs:
let shutdown = shutdown_signal();

axum::serve(listener, app)
    .with_graceful_shutdown(shutdown)
    .await?;

// Post-shutdown: cerrar recursos
info!("Cerrando pool PostgreSQL...");
state.pool.close().await;

info!("Esperando jobs de Apalis...");
// jobs_monitor.shutdown().await;

info!("Cerrando conexiones SSE...");
// sse_broadcaster.close_all().await;

info!("Shutdown completo.");
```

---

## Simplificaciones aplicadas

Para mantener simplicidad operacional
NO se incluyen en el MVP:

* Kubernetes
* gRPC interno
* WebSocket (salvo excepción justificada — usar SSE preferido)
* Service Mesh
* Event sourcing
* CQRS
* Microservicios
* Runtime distribuido
* Kafka
* NATS
* GraphQL

El sistema actual es:

* un backend Axum (REST + SSE),
* un frontend SvelteKit SSR,
* PostgreSQL,
* agentes Rust ligeros,
* Docker Compose.

---

## Herramientas aprobadas

| Herramienta | Propósito | Versión |
|-------------|-----------|---------|
| `tower-http` | Middleware HTTP (cors, trace, timeout, compression) | `0.6.10` |
| `tracing` | Observabilidad estructurada | `0.1` |
| `tracing-subscriber` | Subscripción a logs (JSON, fmt) | `0.3` |
| `utoipa` | Generación OpenAPI | `5.4` |
| `utoipa-scalar` | UI de documentación | `0.4` |
| `cargo-nextest` | Tests rápidos y paralelos | `0.9.135` |
| `tokio-console` | Debug de tareas async (dev only) | `0.1` |
| `axum-test` | Helpers para testing de handlers | `17` |
| `mockall` | Mocking para tests | `0.13` |

---

## Alternativas descartadas

| Opción | Motivo |
|--------|--------|
| Node.js + Express | Mayor consumo de RAM, sin compile-time safety |
| FastAPI + Python | Menor performance, GIL limita concurrencia |
| Spring Boot + JVM | JVM demasiado pesada (> 512MB RAM) |
| Go + Fiber | Menor seguridad de tipos, sin ownership |
| Microservicios | Complejidad operacional excesiva para equipo pequeño |
| gRPC + Protobuf | Overkill para MVP, problemas con firewalls institucionales |

---

## Consecuencias

### ✅ Positivas

* Bajo consumo de recursos (< 512MB RAM para API)
* Excelente performance (Rust zero-cost abstractions)
* Alta concurrencia (miles de conexiones con < 1MB cada una)
* Seguridad de memoria (sin GC pauses, sin segfaults)
* Binario pequeño (< 50MB con dependencias)
* Deploy simple (binario único + systemd/Docker)
* Excelente integración con Docker (imagen scratch/alpine)
* Compatible con VPS económicos (1 vCPU, 1GB RAM)
* SSE nativo en Axum (sin crates adicionales)

---

### ⚠️ Trade-offs

* Curva de aprendizaje alta (borrow checker, lifetimes)
* Compilaciones más lentas que Node/Python
* Ecosistema más joven que Java/Node (pero maduro en web)
* Async debugging más complejo (tokio-console ayuda)
* Menos bibliotecas de machine learning que Python

---

## Impacto regional

El stack backend permite desplegar
el sistema de monitoreo en infraestructura limitada.

Beneficios directos:

* menor costo operativo (VPS de $5-10/mes),
* menor consumo de RAM (deja espacio para PostgreSQL),
* menor consumo de CPU (más sedes monitoreadas con mismo hardware),
* menos reinicios (estabilidad de Rust),
* menos fallos por concurrencia (data race safety),
* mejor estabilidad en producción (compile-time correctness).

Esto es importante para oficinas regionales
con soporte técnico limitado y presupuesto ajustado.

---

## Resultado esperado

Un backend:

* rápido (latencia < 50ms p95),
* estable (uptime > 99.9%),
* seguro (memory safety + PASETO auth),
* eficiente (< 512MB RAM),
* fácil de desplegar (binario + Docker),
* barato de operar (VPS compartido),
* y preparado para crecer sin reescritura (añadir crates, no modificar existentes).

---

## Registro de cambios de versiones

| Fecha | Crate | Anterior | Actual | Notas |
|-------|-------|----------|--------|-------|
| 2026-05-16 | tokio | 1.45 | **1.52** | Runtime actualizado a última estable |
| 2026-05-16 | reqwest | 0.12 | **0.13** | Breaking change: `rustls-tls` → `rustls`; `query`/`form` opcionales |
| 2026-05-16 | axum-extra | 0.10 | **0.12** | Actualización de compatibilidad con axum 0.8 |
| 2026-05-16 | tower-http | 0.6.2 | **0.6.10** | Patches de seguridad y fixes acumulados |
| 2026-05-16 | sqlx | 0.8.5 | **0.8.6** | Patch release con fixes |
| 2026-05-16 | moka | 0.12 | **0.12.15** | Actualización de patch |
| 2026-05-16 | utoipa | 5 | **5.4** | Nuevas features y fixes |
| 2026-05-16 | cargo-nextest | 0.9 | **0.9.135** | Actualización de runner de tests |
| 2026-05-16 | apalis | 0.7 | **1.0** | Salto a versión 1.0 (validar RC antes de producción) |
