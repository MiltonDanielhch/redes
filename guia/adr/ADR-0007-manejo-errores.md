# ADR 0007 — Jerarquía de Errores: Domain → App → HTTP

| Campo               | Valor                                                                  |
| ------------------- | ---------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                             |
| **Fecha**           | 2026                                                                   |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                       |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0003 (Axum), ADR 0016 (Tracing) |

---

# Contexto

La arquitectura hexagonal establece
que el dominio no debe conocer HTTP,
JSON,
ni detalles de infraestructura.

Sin una estrategia unificada de errores:

* las respuestas HTTP se vuelven inconsistentes,
* los errores internos pueden filtrarse,
* el frontend debe manejar múltiples formatos,
* y el debugging operativo se vuelve caótico.

El sistema necesita:

* separación clara de responsabilidades,
* respuestas uniformes,
* logging centralizado,
* y seguridad operacional.

---

# Decisión

Adoptar una jerarquía de errores
dividida en tres niveles:

```txt id="mjlwmw"
DomainError
    ↓
AppError
    ↓
HTTP Response
```

Cada nivel tiene una responsabilidad específica.

---

# Nivel 1 — DomainError

Representa únicamente:

* reglas de negocio,
* validaciones,
* restricciones funcionales.

El dominio:

* no conoce HTTP,
* no conoce Axum,
* no devuelve StatusCode.

---

# Implementación DomainError

```rust id="wtztel"
#[derive(Debug, thiserror::Error)]
pub enum DomainError {

    #[error("email inválido")]
    InvalidEmail,

    #[error("credenciales inválidas")]
    InvalidCredentials,

    #[error("recurso no encontrado")]
    NotFound,

    #[error("operación no permitida")]
    Forbidden,

    #[error("token inválido")]
    InvalidToken,

    #[error("recurso duplicado")]
    Conflict,

    #[error("error interno")]
    Internal,
}
```

---

# Nivel 2 — AppError

Responsable de:

* integrar infraestructura,
* convertir errores externos,
* mapear errores a respuestas HTTP.

---

# Implementación AppError

```rust id="h0qkhy"
#[derive(Debug, thiserror::Error)]
pub enum AppError {

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unauthorized")]
    Unauthorized,

    #[error("internal")]
    Internal,
}
```

---

# Conversión HTTP automática

```rust id="r0z8fr"
impl IntoResponse for AppError {

    fn into_response(self) -> Response {

        let (status, error, message) = match self {

            AppError::Domain(DomainError::InvalidEmail) =>
                (
                    StatusCode::BAD_REQUEST,
                    "invalid_email",
                    "Email inválido"
                ),

            AppError::Domain(DomainError::InvalidCredentials) =>
                (
                    StatusCode::UNAUTHORIZED,
                    "invalid_credentials",
                    "Credenciales inválidas"
                ),

            AppError::Domain(DomainError::Forbidden) =>
                (
                    StatusCode::FORBIDDEN,
                    "forbidden",
                    "Acceso denegado"
                ),

            AppError::Domain(DomainError::NotFound) =>
                (
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "Recurso no encontrado"
                ),

            _ => {
                tracing::error!(error = ?self);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Error interno del servidor"
                )
            }
        };

        (
            status,
            Json(json!({
                "error": error,
                "message": message
            }))
        )
        .into_response()
    }
}
```

---

# Formato estándar de errores

```json id="bzpfhg"
{
  "error": "invalid_credentials",
  "message": "Credenciales inválidas"
}
```

---

# Validaciones HTTP

Las validaciones de entrada
usan respuestas consistentes:

```json id="csjlrk"
{
  "error": "validation_error",
  "fields": [
    {
      "field": "email",
      "message": "Formato inválido"
    }
  ]
}
```

---

# Principios adoptados

| Principio             | Resultado              |
| --------------------- | ---------------------- |
| Dominio puro          | Sin HTTP               |
| Respuestas uniformes  | Frontend simple        |
| Logging centralizado  | Mejor observabilidad   |
| Seguridad             | Sin filtración interna |
| Conversión automática | Menos boilerplate      |

---

# Simplificaciones aplicadas

Para mantener el estándar Código 3026
NO se incluyen:

* árboles complejos de excepciones,
* error frameworks enterprise,
* propagación dinámica avanzada,
* códigos internos excesivos,
* serialización personalizada compleja.

El sistema usa:

* enums simples,
* thiserror,
* IntoResponse,
* y tracing.

---

# Uso en handlers

```rust id="kpjlwm"
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<UserDto>, AppError> {

    let email = Email::new(&body.email)?;

    let user = state
        .user_service
        .create(email)
        .await?;

    Ok(Json(user.into()))
}
```

Los handlers:

* no construyen respuestas manuales,
* no repiten lógica HTTP,
* y delegan el manejo de errores.

---

# Herramientas aprobadas

| Herramienta  | Uso                    |
| ------------ | ---------------------- |
| `thiserror`  | Definición de errores  |
| `tracing`    | Logging                |
| `serde_json` | Respuestas JSON        |
| `axum`       | IntoResponse           |
| `anyhow`     | Arranque de aplicación |

---

# Alternativas descartadas

| Opción                  | Motivo                |
| ----------------------- | --------------------- |
| Panic-based flow        | Inseguro              |
| Strings como errores    | Difícil mantenimiento |
| Responses manuales      | Boilerplate excesivo  |
| HTTP dentro del dominio | Viola arquitectura    |

---

# Consecuencias

## ✅ Positivas

* Respuestas consistentes
* Frontend simplificado
* Mejor trazabilidad
* Seguridad operacional
* Menos duplicación
* Dominio desacoplado

---

## ⚠️ Trade-offs

* Requiere más enums
* El match HTTP crece con el sistema
* Mayor disciplina arquitectónica

---

# Impacto regional

La consistencia de errores:

* facilita soporte remoto,
* acelera debugging,
* mejora monitoreo,
* y reduce tiempo operativo
  en oficinas regionales.

Los logs centralizados
permiten diagnóstico más rápido
ante fallos de infraestructura.

---

# Resultado esperado

Un sistema de errores:

* consistente,
* seguro,
* mantenible,
* observable,
* y desacoplado de infraestructura.
