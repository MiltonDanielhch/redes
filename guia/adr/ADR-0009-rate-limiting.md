# ADR 0009 — Rate Limiting con tower-governor

| Campo               | Valor                                                                                                           |
| ------------------- | --------------------------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                                                      |
| **Fecha**           | 2026-05-16                                                                                                      |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                |
| **Relacionado con** | ADR 0003 (Axum + Tower middleware), ADR 0008 (Auth — límites por endpoint), ADR 0014 (Monitoreo) |
| **Última revisión** | 2026-05-16 — Corrección de versiones + alineación con ROADMAP-AUTH |

---

## Contexto

Exponer una API pública sin límites de peticiones la vuelve vulnerable a:

* **Fuerza bruta** en endpoints sensibles (`/auth/login`, `/auth/register`)
* **Spam automatizado** en formularios y endpoints públicos
* **Scraping masivo** de información
* **Abuso accidental o intencional** que degrade el servidor
* **Consumo innecesario de CPU** debido a `argon2id` y operaciones costosas

Necesitamos una estrategia de rate limiting que:

* Proteja endpoints críticos sin afectar usuarios legítimos
* Sea simple de operar
* Funcione en una sola instancia y pueda evolucionar a múltiples nodos
* Mantenga bajo overhead y mínima latencia

---

## Decisión

Usar **`tower-governor`** como middleware de rate limiting integrado con Axum y Tower.

La estrategia se divide en:

* Límites estrictos para autenticación (por minuto, no por segundo)
* Límites generales para API autenticada
* Exclusión explícita de endpoints internos (`/health`, `/docs`, `/openapi.json`)

---

## Dependencias

```toml
# apps/api/Cargo.toml

# Middleware Tower para rate limiting con GCRA
tower-governor = { version = "0.8.0", features = ["tracing"] }
# Extracción segura de IP real detrás de proxies
axum-client-ip = "1.3.1"
```

**Notas de versión:**
- `tower-governor 0.8.0` es la última estable (ago 2025). Requiere Tower 0.5+. Compatible con Axum 0.8 del proyecto.citeweb_search:19#6
- `axum-client-ip 1.3.1` es la última estable (ene 2026). Requiere Axum 0.7+. Compatible con Axum 0.8.citeweb_search:19#4web_search:19#7
- `governor 0.10.4` es la dependencia subyacente (GCRA). Estado en `AtomicU64`, thread-safe vía CAS.citeweb_search:19#2web_search:19#9

---

## Arquitectura del Rate Limiting

```text
Cliente
   │
   ▼
Caddy / Reverse Proxy
   │
   ├── X-Real-IP
   └── X-Forwarded-For
   │
   ▼
tower-governor
   │
   ├── Auth Limits (por minuto, burst bajo)
   ├── API Limits (por segundo, burst alto)
   └── Excluded routes
   │
   ▼
Handlers Axum
```

---

## Configuración de límites

Alineada con ROADMAP-AUTH-FULLSTACK.md (A.9):

### 1 — Auth: protección contra fuerza bruta

Endpoints:

* `/auth/register`
* `/auth/login`
* `/auth/refresh`
* `/auth/forgot-password`
* `/auth/reset-password`

Configuración (por minuto para evitar bloqueo de operadores legítimos):

```rust
//! Ubicación: `apps/api/src/http/middleware/rate_limit.rs`
//!
//! Descripción: Configuraciones de rate limiting por categoría de endpoint.
//!              Usa GCRA (Generic Cell Rate Algorithm) via tower-governor.
//!
//! ADRs: 0009, 0003, 0008

use std::sync::Arc;
use std::time::Duration;
use tower_governor::{
    governor::{GovernorConfig, GovernorConfigBuilder},
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};

/// Auth — protección anti fuerza bruta
/// 
/// Límite: 10 requests / 60 segundos (equivale a ~1 cada 6s)
/// Burst: 5 (permite ráfaga inicial pequeña)
/// 
/// Combinado con argon2id (~200ms) hace costoso el ataque automatizado.
pub fn auth_rate_limit() -> GovernorLayer<SmartIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_second(6)      // 1 cada 6 segundos = 10 por minuto
        .burst_size(5)
        .use_headers()      // x-ratelimit-limit, x-ratelimit-remaining
        .finish()
        .expect("invalid auth rate limit config");

    GovernorLayer::new(Arc::new(config))
}

/// Auth register — más estricto
/// 
/// Límite: 5 requests / 15 minutos
pub fn auth_register_rate_limit() -> GovernorLayer<SmartIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_second(180)    // 1 cada 180s = 5 por 15min
        .burst_size(3)
        .use_headers()
        .finish()
        .expect("invalid register rate limit config");

    GovernorLayer::new(Arc::new(config))
}

/// Auth forgot-password — muy estricto
/// 
/// Límite: 3 requests / 60 minutos
pub fn auth_forgot_rate_limit() -> GovernorLayer<SmartIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_second(1200)   // 1 cada 1200s = 3 por hora
        .burst_size(2)
        .use_headers()
        .finish()
        .expect("invalid forgot-password rate limit config");

    GovernorLayer::new(Arc::new(config))
}
```

---

### 2 — API autenticada general

```rust
/// API general autenticada — uso normal de dashboard
/// 
/// Límite: 30 requests / 60 segundos (equivale a ~1 cada 2s)
/// Burst: 20 (permite carga inicial de página)
pub fn api_rate_limit() -> GovernorLayer<SmartIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_second(2)      // 1 cada 2 segundos = 30 por minuto
        .burst_size(20)
        .use_headers()
        .finish()
        .expect("invalid api rate limit config");

    GovernorLayer::new(Arc::new(config))
}
```

---

## Integración en Axum

```rust
//! Ubicación: `apps/api/src/router.rs`
//!
//! Descripción: Router Axum con rate limiting aplicado por grupo de rutas.
//!
//! ADRs: 0003, 0009

use axum::{
    routing::{get, post},
    Router,
};
use axum_client_ip::ClientIpSource;

pub fn build_router(state: AppState) -> Router {
    // Auth — límites estrictos
    let auth_routes = Router::new()
        .route("/auth/register", post(register_handler))
        .layer(auth_register_rate_limit())
        .route("/auth/login", post(login_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/auth/forgot-password", post(forgot_password_handler))
        .route("/auth/reset-password", post(reset_password_handler))
        .layer(auth_rate_limit());

    // API autenticada — límites generales
    let api_routes = Router::new()
        .route("/api/v1/sedes", get(list_sedes).post(create_sede))
        .route("/api/v1/devices", get(list_devices).post(create_device))
        .route("/api/v1/devices/:id", get(get_device).put(update_device))
        .route("/api/v1/devices/:id/archive", put(archive_device))
        .route("/api/v1/metrics", get(get_metrics).post(ingest_metrics))
        .route("/api/v1/alerts", get(list_alerts))
        .route("/api/v1/alerts/:id/acknowledge", post(acknowledge_alert))
        .route("/api/v1/topology/:sede_id", get(get_topology))
        .route("/api/v1/intrusions", get(list_intrusions))
        .route("/api/v1/intrusions/:id/resolve", post(resolve_intrusion))
        .layer(auth_middleware)
        .layer(api_rate_limit());

    // Excluidos explícitamente — sin rate limiting
    let public_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/docs", get(scalar_ui_handler))
        .route("/openapi.json", get(openapi_json_handler));

    Router::new()
        .merge(auth_routes)
        .merge(api_routes)
        .merge(public_routes)
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>()
}
```

**Nota:** `into_make_service_with_connect_info::<SocketAddr>()` es obligatorio para que `SmartIpKeyExtractor` funcione correctamente.

---

## Respuesta HTTP al exceder el límite

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
Retry-After: 30
x-ratelimit-limit: 10
x-ratelimit-remaining: 0
x-ratelimit-after: 30

{
  "error": "too_many_requests",
  "message": "Rate limit exceeded. Try again later.",
  "retry_after": 30
}
```

---

## Tabla oficial de límites

| Endpoint | Límite | Burst | Objetivo |
| --- | --- | --- | --- |
| `/auth/register` | 5 / 15 min | 3 | Anti fuerza bruta |
| `/auth/login` | 10 / 5 min | 5 | Anti fuerza bruta |
| `/auth/forgot-password` | 3 / 60 min | 2 | Anti fuerza bruta |
| `/auth/reset-password` | 5 / 15 min | 3 | Anti fuerza bruta |
| `/auth/refresh` | 20 / 1 min | 10 | Uso legítimo alto |
| API autenticada (`/api/v1/*`) | 30 / 60 s | 20 | Uso normal dashboard |
| `/health` | Sin límite | — | Healthchecks |
| `/docs`, `/openapi.json` | Sin límite | — | Documentación |

---

## Integración con observabilidad

Cada evento de rate limit se registra automáticamente via `tracing` (feature `tracing` habilitado):

```rust
tracing::warn!(
    ip         = %client_ip,
    endpoint   = %req.uri(),
    limit      = %config.burst_size,
    "rate limit exceeded"
);
```

Además:

* `tower-governor` envía headers `x-ratelimit-*` en cada respuesta (no solo 429)
* Métricas internas pueden contar respuestas 429
* Grafana/Healthchecks pueden alertar picos anormales

---

## Extracción correcta de IP

Detrás de proxies reversos como Caddy:

```rust
use axum_client_ip::ClientIpSource;

// En Caddyfile:
// header_up X-Real-IP {remote_host}
// header_up X-Forwarded-For {remote_host}

let ip_source = ClientIpSource::XRealIp;
```

Caddy debe reenviar:

```caddyfile
reverse_proxy localhost:8080 {
    header_up X-Real-IP {remote_host}
    header_up X-Forwarded-For {remote_host}
}
```

---

## Cleanup de storage in-memory

`tower-governor` mantiene un HashMap en memoria que crece con cada IP nueva. Requiere limpieza periódica:

```rust
//! Ubicación: `apps/api/src/main.rs` (setup)
//!
//! Descripción: Background task para limpiar entries antiguos del rate limiter.

use std::time::Duration;

pub fn spawn_rate_limit_cleanup(config: &GovernorConfig) {
    let limiter = config.limiter().clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 min

        loop {
            interval.tick().await;
            limiter.retain_recent();
            tracing::info!("rate limiter storage cleaned: {} active entries", limiter.len());
        }
    });
}
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
| --- | --- |
| Rate limiting en Caddy | Sin contexto de endpoints ni lógica de aplicación |
| Redis + sliding window | Complejidad innecesaria para etapa inicial |
| Fail2ban | Solo protege por IP, no por endpoint |
| Cloudflare Rate Limit | Dependencia externa y costo adicional |
| Implementación manual | Más compleja y propensa a errores |

---

## Herramientas y Librerías

| Herramienta | Propósito | Versión |
| --- | --- | --- |
| `tower-governor` | Middleware GCRA para Tower/Axum | **0.8.0** |
| `axum-client-ip` | Extracción segura de IP real | **1.3.1** |
| `governor` | Algoritmo GCRA subyacente | **0.10.4** (via tower-governor) |
| `tracing` | Logging de eventos 429 | workspace |

---

## Consecuencias

### ✅ Positivas

* Protección inmediata contra fuerza bruta
* Bajo overhead — todo ocurre in-process
* Sin dependencias externas adicionales
* Integración natural con Axum y Tower
* Compatible con observabilidad moderna
* Fácil evolución futura a Redis distribuido

### ⚠️ Negativas / Trade-offs

**Estado in-memory**
El rate limit se reinicia cuando reinicia el proceso.
→ Aceptable en arquitectura monolítica inicial
→ Futuramente Redis puede centralizar el estado

**Usuarios detrás de NAT**
Múltiples usuarios pueden compartir IP pública.
→ Ajustar `burst_size` según tráfico real
→ Confiar en `X-Real-IP` desde proxy seguro

**Límites demasiado estrictos**
Puede bloquear clientes legítimos.
→ Monitorear métricas `429`
→ Ajustar límites basados en comportamiento real

---

## Decisiones derivadas

* `/health`, `/docs`, `/openapi.json` quedan excluidos explícitamente
* Caddy reenvía `X-Real-IP` y `X-Forwarded-For`
* `argon2id` y rate limiting funcionan como defensa combinada
* Los eventos `429` se registran en tracing
* Fase futura multi-instancia: evaluar Redis distribuido
* `tower-governor` es el estándar oficial del proyecto para control de tráfico
* Headers `x-ratelimit-*` obligatorios en todas las respuestas (feature `use_headers`)
* Cleanup de storage cada 5 minutos via background task

---

## Notas de actualización de versiones (2026-05-16)

| Componente | Versión/Config | Notas |
|------------|----------------|-------|
| **tower-governor** | **0.8.0** | Última estable (ago 2025). Requiere Tower 0.5+. Compatible con Axum 0.8. Features: `tracing`, `key-extractor`. |
| **axum-client-ip** | **1.3.1** | Última estable (ene 2026). Requiere Axum 0.7+. Compatible con Axum 0.8. Extracción de IP real detrás de proxies. |
| **governor** | **0.10.4** | Última estable (dic 2025). GCRA algorithm. Estado en `AtomicU64`, thread-safe vía CAS. 3.7M+ descargas/mes. |
| **Tower** | **0.5.x** | Middleware framework. `tower-governor 0.8.0` requiere Tower 0.5+. |
| **GCRA** | Generic Cell Rate Algorithm | Equivalente funcional a leaky bucket. Sin background drip process. Actualización continua en nanosegundos. |
