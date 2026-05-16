# ADR 0009 — Rate Limiting con tower-governor

| Campo               | Valor                                                                                                           |
| ------------------- | --------------------------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                                                      |
| **Fecha**           | 2026                                                                                                            |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                |
| **Relacionado con** | ADR 0003 (Axum + Tower middleware), ADR 0008 (Auth — límites por endpoint), ADR 0015 (Monitoreo) |

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

# Decisión

Usar **`tower-governor`** como middleware de rate limiting integrado con Axum y Tower.

La estrategia se divide en:

* límites estrictos para autenticación
* límites anti-spam para endpoints públicos
* límites generales para API autenticada
* exclusión explícita de endpoints internos (`/health`, `/metrics`)

---

# Dependencias

```toml
# apps/api/Cargo.toml

tower-governor = { version = "0.4", features = ["axum"] }
axum-client-ip = "0.6"
```

---

# Arquitectura del Rate Limiting

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
   ├── Auth Limits
   ├── Leads Limits
   └── API Limits
   │
   ▼
Handlers Axum
```

---

# Configuración de límites

## 1 — Auth: protección contra fuerza bruta

Endpoints:

* `/auth/login`
* `/auth/register`
* `/auth/refresh`
* `/auth/forgot-password`

Configuración:

```rust
// apps/api/src/http/middleware/rate_limit.rs

use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};

/// Auth — protección anti fuerza bruta
///
/// 1 request/segundo
/// burst de 5
///
/// Combinado con argon2id (~200ms)
/// hace extremadamente costoso el ataque automatizado.
pub fn auth_rate_limit() -> GovernorLayer<SmartIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(5)
        .finish()
        .expect("invalid auth rate limit config");

    GovernorLayer::new(config)
}
```

---

## 2 — Leads: anti-spam de formularios

```rust
/// Leads — anti spam
///
/// Máximo 3 requests por minuto.
pub fn leads_rate_limit() -> GovernorLayer<SmartIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_minute(3)
        .burst_size(3)
        .finish()
        .expect("invalid leads rate limit config");

    GovernorLayer::new(config)
}
```

---

## 3 — API autenticada general

```rust
/// API general autenticada
///
/// Uso normal de dashboard, tablas y navegación.
pub fn api_rate_limit() -> GovernorLayer<SmartIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(30)
        .finish()
        .expect("invalid api rate limit config");

    GovernorLayer::new(config)
}
```

---

# Integración en Axum

```rust
// apps/api/src/router.rs

pub fn build_router(state: AppState) -> Router {
    // Auth
    let auth_routes = Router::new()
        .route("/auth/login", post(login_handler))
        .route("/auth/register", post(register_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/auth/forgot-password", post(forgot_password_handler))
        .layer(auth_rate_limit());

    // Leads
    let leads_routes = Router::new()
        .route("/api/v1/leads", post(capture_lead_handler))
        .layer(leads_rate_limit());

    // API autenticada
    let api_routes = Router::new()
        .route("/api/v1/users", get(list_users).post(create_user))
        .route("/api/v1/users/:id", get(get_user).put(update_user))
        .route("/api/v1/roles", get(list_roles))
        .layer(auth_middleware)
        .layer(api_rate_limit());

    Router::new()
        .merge(auth_routes)
        .merge(leads_routes)
        .merge(api_routes)

        // Excluidos explícitamente
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))

        .with_state(state)
}
```

---

# Respuesta HTTP al exceder el límite

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
Retry-After: 30

{
  "error": "too_many_requests",
  "message": "Rate limit exceeded. Try again later.",
  "retry_after": 30
}
```

---

# Tabla oficial de límites

| Endpoint        | Límite     | Burst | Objetivo          |
| --------------- | ---------- | ----- | ----------------- |
| `/auth/*`       | 1 req/s    | 5     | Anti fuerza bruta |
| `/api/v1/leads` | 3 req/min  | 3     | Anti spam         |
| API autenticada | 10 req/s   | 30    | Uso normal        |
| `/health`       | Sin límite | —     | Healthchecks      |
| `/metrics`      | Interno    | —     | Observabilidad    |

---

# Integración con observabilidad

Cada evento de rate limit se registra en tracing:

```rust
tracing::warn!(
    ip         = %client_ip,
    endpoint   = %req.uri(),
    "rate limit exceeded"
);
```

Además:

* `metrics` incrementa contador de respuestas `429`
* Prometheus puede alertar picos anormales
* Grafana puede visualizar ataques o spam en tiempo real

---

# Extracción correcta de IP

Detrás de proxies reversos como Caddy o Nginx:

```rust
use axum_client_ip::SecureClientIpSource;

let ip_source = SecureClientIpSource::XRealIp;
```

Caddy debe reenviar:

```caddyfile
header_up X-Real-IP {remote_host}
header_up X-Forwarded-For {remote_host}
```

---

# Alternativas consideradas

| Opción                 | Motivo de descarte                                |
| ---------------------- | ------------------------------------------------- |
| Rate limiting en Caddy | Sin contexto de endpoints ni lógica de aplicación |
| Redis + sliding window | Complejidad innecesaria para etapa inicial        |
| Fail2ban               | Solo protege por IP, no por endpoint              |
| Cloudflare Rate Limit  | Dependencia externa y costo adicional             |
| Implementación manual  | Más compleja y propensa a errores                 |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta                   | Propósito                              |
| ----------------------------- | -------------------------------------- |
| `axum-client-ip`              | Extrae IP real detrás de proxies       |
| `governor`                    | Implementación GCRA eficiente          |
| `metrics`                     | Monitoreo de eventos 429               |
| `tower-http::CatchPanicLayer` | Evita caída del proceso ante panic     |
| `tracing`                     | Auditoría y debugging                  |
| `moka`                        | Cache futura para límites distribuidos |

---

# Consecuencias

## ✅ Positivas

* Protección inmediata contra fuerza bruta
* Bajo overhead — todo ocurre in-process
* Sin dependencias externas adicionales
* Integración natural con Axum y Tower
* Compatible con observabilidad moderna
* Fácil evolución futura a Redis distribuido

---

## ⚠️ Negativas / Trade-offs

### Estado in-memory

El rate limit se reinicia cuando reinicia el proceso.

→ Aceptable en arquitectura monolítica inicial
→ Futuramente Redis puede centralizar el estado

---

### Usuarios detrás de NAT

Múltiples usuarios pueden compartir IP pública.

→ Ajustar `burst_size` según tráfico real
→ Confiar en `X-Real-IP` desde proxy seguro

---

### Límites demasiado estrictos

Puede bloquear clientes legítimos.

→ Monitorear métricas `429`
→ Ajustar límites basados en comportamiento real

---

# Decisiones derivadas

* `/health` y `/metrics` quedan excluidos explícitamente
* Caddy reenvía `X-Real-IP`
* `argon2id` y rate limiting funcionan como defensa combinada
* Los eventos `429` se registran en tracing y métricas
* Fase futura multi-instancia: evaluar Redis distribuido
* `tower-governor` es el estándar oficial del proyecto para control de tráfico
