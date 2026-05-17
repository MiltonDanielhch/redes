# ADR 0007 — Manejo de Errores: DomainError + AppError + HTTP

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0003 (Stack Backend), ADR 0006 (RBAC), ADR 0008 (Seguridad), ADR 0009 (Rate Limiting), ADR 0016 (OpenAPI) |

---

## Contexto

Los errores son inevitables.

Cómo los manejamos define la calidad del sistema.

El sistema de monitoreo de infraestructura de red de la Gobernación del Beni
necesita:

* errores legibles para usuarios,
* trazabilidad completa para desarrolladores,
* información mínima al cliente (sin exponer detalles internos),
* y categorización clara para diagnóstico rápido.

Un error mal manejado puede exponer:

* estructura de base de datos,
* rutas internas,
* secretos,
* o información sensible de usuarios.

---

## Decisión

Usar una jerarquía de tres niveles:

```txt
DomainError     → Errores del negocio (domain/)
AppError        → Errores de la aplicación (api/)
HTTP Response   → Errores para el cliente (JSON)
```

Cada nivel conoce su responsabilidad.

Ningún nivel expone información de niveles inferiores innecesariamente.

---

## Nivel 1 — DomainError

Errores del dominio. Lenguaje del negocio.

```rust
// crates/domain/src/errors.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    // ── Validación ─────────────────────────────────────────
    #[error("Validación fallida: {0}")]
    Validation(String),

    // ── No encontrado ──────────────────────────────────────
    #[error("Recurso no encontrado: {0}")]
    NotFound(String),

    // ── Ya existe ──────────────────────────────────────────
    #[error("Recurso ya existe: {0}")]
    AlreadyExists(String),

    // ── Autenticación ──────────────────────────────────────
    #[error("Credenciales inválidas")]
    InvalidCredentials,

    #[error("Token inválido o expirado")]
    InvalidToken,

    #[error("Sesión expirada")]
    SessionExpired,

    // ── Autorización ───────────────────────────────────────
    #[error("Acceso no autorizado")]
    Unauthorized,

    #[error("Permiso requerido: {0}")]
    MissingPermission(String),

    // ── Conflicto ──────────────────────────────────────────
    #[error("Conflicto de estado: {0}")]
    Conflict(String),

    // ── Base de datos ──────────────────────────────────────
    #[error("Error de base de datos: {0}")]
    Database(String),

    // ── Interno ────────────────────────────────────────────
    #[error("Error interno: {0}")]
    Internal(String),
}
```

**Regla:** `DomainError` nunca conoce HTTP, Axum, ni serialización. Es puro negocio.

---

## Nivel 2 — AppError

Errores de la aplicación. Wrapper de HTTP.

```rust
// apps/api/src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::errors::DomainError;
use serde::Serialize;
use tracing::{error, warn};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug)]
pub enum AppError {
    // ── 400 Bad Request ────────────────────────────────────
    BadRequest(String),                    // Parsing JSON, query params inválidos

    // ── 401 Unauthorized ───────────────────────────────────
    InvalidCredentials,                    // Login fallido
    InvalidToken,                          // Token malformado o expirado
    SessionExpired,                        // Refresh token expirado

    // ── 403 Forbidden ──────────────────────────────────────
    Forbidden,                             // Sin permisos
    MissingPermission(String),             // Permiso específico requerido

    // ── 404 Not Found ──────────────────────────────────────
    NotFound(String),                      // Recurso no existe

    // ── 409 Conflict ───────────────────────────────────────
    AlreadyExists(String),                 // Email duplicado, MAC duplicada

    // ── 422 Unprocessable Entity ───────────────────────────
    Validation(Vec<FieldError>),           // Errores de validación de dominio

    // ── 429 Too Many Requests ──────────────────────────────
    RateLimited { retry_after: u64 },      // Rate limiting

    // ── 500 Internal Server Error ──────────────────────────
    Internal(String),                    // Error interno (no expuesto al cliente)

    // ── 503 Service Unavailable ────────────────────────────
    ServiceUnavailable(String),            // PostgreSQL no responde, pool saturado

    // ── 504 Gateway Timeout ────────────────────────────────
    GatewayTimeout(String),                // Timeout de servicio externo (Resend, S3)
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Validation(msg) => {
                AppError::Validation(vec![FieldError {
                    field: "general".to_string(),
                    message: msg,
                }])
            }
            DomainError::NotFound(msg) => AppError::NotFound(msg),
            DomainError::AlreadyExists(msg) => AppError::AlreadyExists(msg),
            DomainError::InvalidCredentials => AppError::InvalidCredentials,
            DomainError::InvalidToken => AppError::InvalidToken,
            DomainError::SessionExpired => AppError::SessionExpired,
            DomainError::Unauthorized => AppError::Forbidden,
            DomainError::MissingPermission(perm) => AppError::MissingPermission(perm),
            DomainError::Conflict(msg) => AppError::AlreadyExists(msg),
            DomainError::Database(msg) => AppError::Internal(msg),
            DomainError::Internal(msg) => AppError::Internal(msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message, details, retry_after) = match &self {
            // 400 Bad Request
            AppError::BadRequest(msg) => {
                warn!(error = %msg, "Bad request");
                (StatusCode::BAD_REQUEST, "bad_request", msg.clone(), None, None)
            }

            // 401 Unauthorized
            AppError::InvalidCredentials => {
                warn!("Invalid credentials attempt");
                (StatusCode::UNAUTHORIZED, "invalid_credentials", "Credenciales incorrectas".to_string(), None, None)
            }
            AppError::InvalidToken => {
                warn!("Invalid token");
                (StatusCode::UNAUTHORIZED, "invalid_token", "Token inválido o expirado".to_string(), None, None)
            }
            AppError::SessionExpired => {
                warn!("Session expired");
                (StatusCode::UNAUTHORIZED, "session_expired", "Sesión expirada. Inicia sesión nuevamente.".to_string(), None, None)
            }

            // 403 Forbidden
            AppError::Forbidden => {
                warn!("Forbidden access");
                (StatusCode::FORBIDDEN, "forbidden", "Acceso no autorizado".to_string(), None, None)
            }
            AppError::MissingPermission(perm) => {
                warn!(permission = %perm, "Missing permission");
                (StatusCode::FORBIDDEN, "missing_permission", format!("Permiso requerido: {}", perm), None, None)
            }

            // 404 Not Found
            AppError::NotFound(msg) => {
                warn!(resource = %msg, "Not found");
                (StatusCode::NOT_FOUND, "not_found", msg.clone(), None, None)
            }

            // 409 Conflict
            AppError::AlreadyExists(msg) => {
                warn!(resource = %msg, "Already exists");
                (StatusCode::CONFLICT, "already_exists", msg.clone(), None, None)
            }

            // 422 Unprocessable Entity
            AppError::Validation(fields) => {
                warn!(fields = ?fields, "Validation failed");
                let details = serde_json::to_value(fields).ok();
                (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", "Validación fallida".to_string(), details, None)
            }

            // 429 Too Many Requests
            AppError::RateLimited { retry_after } => {
                warn!(retry_after = *retry_after, "Rate limited");
                (StatusCode::TOO_MANY_REQUESTS, "rate_limited", "Demasiadas peticiones".to_string(), None, Some(*retry_after))
            }

            // 500 Internal Server Error
            AppError::Internal(msg) => {
                error!(error = %msg, "Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Error interno del servidor".to_string(), None, None)
            }

            // 503 Service Unavailable
            AppError::ServiceUnavailable(msg) => {
                error!(error = %msg, "Service unavailable");
                (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable", "Servicio no disponible. Intenta más tarde.".to_string(), None, None)
            }

            // 504 Gateway Timeout
            AppError::GatewayTimeout(msg) => {
                error!(error = %msg, "Gateway timeout");
                (StatusCode::GATEWAY_TIMEOUT, "gateway_timeout", "Tiempo de espera agotado".to_string(), None, None)
            }
        };

        let body = Json(ErrorResponse {
            error: error_code.to_string(),
            message,
            details,
            request_id: None, // Se inyecta en middleware de request_id
        });

        let mut response = (status, body).into_response();

        // Headers adicionales
        if let Some(retry) = retry_after {
            response.headers_mut().insert(
                "Retry-After",
                retry.to_string().parse().unwrap(),
            );
        }

        response
    }
}
```

---

## Mapeo HTTP completo

| Status | AppError | Uso |
|--------|----------|-----|
| 400 | `BadRequest` | JSON malformado, query params inválidos |
| 401 | `InvalidCredentials` | Login fallido |
| 401 | `InvalidToken` | Token PASETO inválido |
| 401 | `SessionExpired` | Refresh token expirado |
| 403 | `Forbidden` | Sin permisos (RBAC) |
| 403 | `MissingPermission` | Permiso específico faltante |
| 404 | `NotFound` | Recurso no existe |
| 409 | `AlreadyExists` | Email duplicado, MAC duplicada |
| 422 | `Validation` | Errores de validación de dominio |
| 429 | `RateLimited` | Demasiadas peticiones |
| 500 | `Internal` | Error interno (no expuesto al cliente) |
| 503 | `ServiceUnavailable` | PostgreSQL no responde |
| 504 | `GatewayTimeout` | Timeout de servicio externo |

---

## Middleware de error con contexto

```rust
// apps/api/src/middleware/error_context.rs

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::Span;
use uuid::Uuid;

pub async fn error_context_middleware(
    request: Request,
    next: Next,
) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_else(|| {
            let id = Uuid::new_v4().to_string();
            // Inyectar request_id en extensions para uso posterior
            id.as_str()
        });

    let user_id = request
        .extensions()
        .get::<AuthClaims>()
        .map(|c| c.user_id.to_string());

    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        user_id = ?user_id,
        method = %request.method(),
        path = %request.uri().path(),
    );

    let response = next.run(request).instrument(span).await;

    // Inyectar request_id en response headers
    let mut response = response;
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    response
}
```

---

## Ejemplos de errores en endpoints

```rust
// POST /auth/register
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Validación de email
    if !is_valid_email(&payload.email) {
        return Err(AppError::Validation(vec![
            FieldError {
                field: "email".to_string(),
                message: "Formato de email inválido".to_string(),
            }
        ]));
    }

    // Validación de password
    if payload.password.len() < 12 {
        return Err(AppError::Validation(vec![
            FieldError {
                field: "password".to_string(),
                message: "La contraseña debe tener al menos 12 caracteres".to_string(),
            }
        ]));
    }

    // Caso de uso
    let user = state
        .register_use_case
        .execute(payload.into())
        .await
        .map_err(|e| match e {
            DomainError::AlreadyExists(msg) => AppError::AlreadyExists(msg),
            DomainError::Validation(msg) => AppError::Validation(vec![
                FieldError { field: "general".to_string(), message: msg }
            ]),
            other => AppError::from(other),
        })?;

    Ok(Json(UserResponse::from(user)))
}
```

---

## Formato de respuesta HTTP

### Error simple

```json
{
  "error": "not_found",
  "message": "Dispositivo no encontrado: dev_xxx",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Error de validación (422)

```json
{
  "error": "validation_error",
  "message": "Validación fallida",
  "details": [
    { "field": "email", "message": "Formato inválido" },
    { "field": "password", "message": "Debe tener al menos 12 caracteres" },
    { "field": "mac_address", "message": "Formato IEEE 802 requerido: xx:xx:xx:xx:xx:xx" }
  ],
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Rate limiting (429)

```json
{
  "error": "rate_limited",
  "message": "Demasiadas peticiones",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Headers:**
```
Retry-After: 60
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1715863200
```

---

## Principios adoptados

| Principio | Descripción |
|-----------|-------------|
| Nunca exponer internos | El cliente no ve stack traces ni queries SQL |
| Errores tipados | Cada error tiene categoría y mensaje |
| Jerarquía clara | Domain → App → HTTP, sin mezcla |
| Logging completo | Todos los errores loggeados con contexto (request_id, user_id) |
| Headers informativos | Rate limiting incluye Retry-After |
| Request ID propagado | Cada error incluye request_id para trazabilidad |
| 422 para validación | Distinguir syntax errors (400) de business validation (422) |

---

## Herramientas aprobadas

| Herramienta | Propósito | Versión | Notas |
|-------------|-----------|---------|-------|
| `thiserror` | Errores tipados en dominio | `2.0.18` | Última estable (feb 2026). Soporta `#[no_std]` con `default-features = false` bajo rustc 1.81.0+. Usar en `crates/` |
| `anyhow` | Errores genéricos en bins | `1.0.98` | Última estable (abr 2026). **Solo** en `apps/` (api, agent, cli), nunca en `crates/` |
| `eyre` | Reportes de error personalizados | `0.6.12` | Fork de anyhow con handlers customizables. Alternativa si se necesita `color-eyre` |
| `serde_json` | Serialización de errores HTTP | `1.0.149` | Última estable (ene 2026). En `apps/api/src/error.rs` |
| `tracing` | Logging estructurado | `0.1` | Con contexto (request_id, user_id) |

---

## Alternativas descartadas

| Opción | Motivo |
|--------|--------|
| `anyhow` en librerías | Pierde tipado, dificulta matching de errores |
| `eyre` | Similar a anyhow, mismo problema |
| Errores como strings | Sin estructura, imposible categorizar |
| Exponer SQL al cliente | Riesgo de seguridad |
| Un solo tipo de error | Sin granularidad para HTTP |

---

## Consecuencias

### ✅ Positivas

* Errores categorizados y tipados
* Mensajes claros para usuarios
* Trazabilidad completa para desarrolladores (request_id)
* Seguridad (no se exponen detalles internos)
* Fácil testing (match por variantes)
* Compatibilidad con OpenAPI (códigos HTTP documentados)
* Rate limiting con headers informativos
* Diferenciación clara entre 400, 422, 409

---

### ⚠️ Trade-offs

* Más código inicial que `Result<T, String>`
* Requiere mantener mapeo DomainError → AppError
* Cada nuevo error requiere actualizar dos enums
* Contexto de logging requiere middleware adicional

---

## Impacto regional

Un manejo de errores robusto permite:

* diagnóstico remoto eficiente (request_id en cada error),
* soporte técnico rápido (categorización clara),
* menos frustración para usuarios (mensajes en español),
* y menor riesgo de exposición de información sensible.

Esto es crítico para:

* oficinas con conectividad limitada,
* personal no técnico,
* y sistemas que manejan datos institucionales.

---

## Resultado esperado

Un sistema donde:

* los errores son predecibles,
* los mensajes son claros,
* la trazabilidad es completa,
* la seguridad no se compromete,
* y el diagnóstico es eficiente.

---

## Notas de actualización de versiones (2026-05-16)

| Componente | Versión/Config | Notas |
|------------|----------------|-------|
| **thiserror** | **2.0.18** | Última estable (feb 2026). Soporta `#[no_std]` con `default-features = false` bajo rustc 1.81.0+. Usar en `crates/` (librerías). |
| **thiserror-impl** | **2.0.18** | Procedural macros para `thiserror`. Se instala automáticamente como dependencia. |
| **anyhow** | **1.0.98** | Última estable (abr 2026). Errores genéricos con contexto. **Solo en `apps/`**, nunca en `crates/`. |
| **eyre** | **0.6.12** | Fork de anyhow con handlers customizables. Usar con `color-eyre` para pretty-print. |
| **serde_json** | **1.0.149** | Última estable (ene 2026). Serialización JSON. Serie 1.x estable. |
| **tracing** | **0.1.x** | Serie 0.1.x estable. Logging estructurado con spans. |
