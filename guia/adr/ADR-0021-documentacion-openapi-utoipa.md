# ADR 0021 — OpenAPI: Utoipa + Scalar + IA-Ready

| Campo               | Valor                                                                                                                    |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                                               |
| **Fecha**           | 2026                                                                                                                     |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                         |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0003 (Axum), ADR 0008 (PASETO Auth), ADR 0010 (Testing), ADR 0027 (Contratos API), ADR 0035 (Monitoreo Regional) |

---

# Contexto

La documentación de API escrita manualmente inevitablemente se desincroniza del código real.

En arquitecturas modernas donde:

* múltiples clientes consumen la API
* existen agentes IA que deben entender contratos automáticamente
* el frontend genera tipos desde backend
* los endpoints cambian constantemente

... mantener archivos YAML manuales se convierte en deuda técnica.

Necesitamos una solución que:

* Genere OpenAPI directamente desde Rust
* Mantenga el spec sincronizado automáticamente
* Permita testing interactivo desde navegador
* Sea ligera para un VPS de bajos recursos
* Sea consumible por herramientas IA y codegen
* No contamine el dominio con macros de infraestructura

---

# Decisión

Usar:

* **Utoipa** para generación automática de OpenAPI
* **Scalar** como interfaz visual moderna
* `/openapi.json` como contrato oficial IA-ready
* Macros exclusivamente en infraestructura

La arquitectura sigue:

```text id="o2mqm5"
HTTP Handlers
      ↓
DTOs + utoipa macros
      ↓
OpenAPI Generator
      ↓
/openapi.json
      ↓
Scalar UI / IA / Frontend codegen
```

---

# Objetivos Técnicos

| Objetivo               | Resultado                               |
| ---------------------- | --------------------------------------- |
| Single source of truth | El código Rust genera el spec           |
| IA-ready               | Agentes pueden consumir `/openapi.json` |
| Frontend type-safe     | Generación automática de tipos          |
| Bajo consumo           | Sin Swagger pesado                      |
| Arquitectura limpia    | Sin macros en dominio                   |
| DX moderna             | Scalar UI                               |

---

# Dependencias

```toml id="ovjv8e"
# apps/api/Cargo.toml

[dependencies]
utoipa = {
    version = "4",
    features = [
        "axum_extras",
        "uuid",
        "chrono",
    ]
}

utoipa-scalar = {
    version = "0.1",
    features = ["axum"]
}
```

---

# Reglas Arquitectónicas

## Permitido

✅ DTOs HTTP

✅ Handlers Axum

✅ Responses HTTP

✅ Infraestructura

---

## Prohibido

❌ Entidades de dominio

❌ Casos de uso

❌ Value Objects

❌ Traits de dominio

---

# DTOs Documentados

```rust id="r4mkos"
// crates/infrastructure/src/http/dto/user_dto.rs

use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(
    Serialize,
    Deserialize,
    ToSchema,
)]
pub struct UserDto {

    #[schema(example = "usr_123")]
    pub id: String,

    #[schema(example = "user@example.com")]
    pub email: String,

    #[schema(example = "2026-01-15T10:00:00Z")]
    pub created_at: String,
}

#[derive(
    Serialize,
    Deserialize,
    ToSchema,
)]
pub struct CreateUserRequest {

    #[schema(example = "user@example.com")]
    pub email: String,

    #[schema(
        example = "Password123!",
        min_length = 8
    )]
    pub password: String,
}
```

---

# Documentación de Endpoints

```rust id="cq7te9"
// crates/infrastructure/src/http/handlers/user_handler.rs

#[utoipa::path(
    post,
    path = "/api/v1/users",

    request_body = CreateUserRequest,

    responses(
        (
            status = 201,
            description = "Usuario creado",
            body = UserDto
        ),

        (
            status = 409,
            description = "Email duplicado",
            body = ErrorResponse
        ),

        (
            status = 422,
            description = "Validation error",
            body = ValidationError
        )
    ),

    security(
        ("bearer_auth" = [])
    ),

    tag = "users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> impl IntoResponse {

    // ...
}
```

---

# Registro Central del OpenAPI

```rust id="lqfrqs"
// apps/api/src/docs.rs

use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
#[openapi(

    paths(
        user_handler::create_user,
        user_handler::get_user,
        user_handler::list_users,

        auth_handler::login,
        auth_handler::logout,
        auth_handler::refresh,

        network_handler::list_devices,
        network_handler::device_metrics,
    ),

    components(
        schemas(
            UserDto,
            CreateUserRequest,

            LoginRequest,
            AuthResponse,

            ErrorResponse,
            ValidationError,
        )
    ),

    modifiers(&SecurityAddon),

    tags(
        (
            name = "auth",
            description = "Autenticación"
        ),

        (
            name = "users",
            description = "Usuarios"
        ),

        (
            name = "network",
            description = "Monitoreo de red"
        )
    ),

    info(
        title = "Gobernación Beni — Network Monitor API",

        version = env!("CARGO_PKG_VERSION"),

        description = "API de monitoreo de red del sistema de la Gobernación del Beni"
    )
)]
pub struct ApiDoc;
```

---

# Seguridad — PASETO

```rust id="xjc3ml"
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {

    fn modify(
        &self,
        openapi: &mut utoipa::openapi::OpenApi,
    ) {

        if let Some(components) =
            openapi.components.as_mut()
        {

            components.add_security_scheme(
                "bearer_auth",

                utoipa::openapi::security::SecurityScheme::Http(

                    utoipa::openapi::security::HttpBuilder::new()

                        .scheme(
                            utoipa::openapi::security::HttpAuthScheme::Bearer
                        )

                        // IMPORTANTE:
                        // No JWT — usamos PASETO v4.local
                        .bearer_format("PASETO")

                        .build(),
                ),
            );
        }
    }
}
```

---

# Router de Documentación

```rust id="jv8mtt"
// apps/api/src/docs/router.rs

pub fn docs_router() -> Router {

    Router::new()

        .route(
            "/openapi.json",

            get(|| async {
                Json(ApiDoc::openapi())
            })
        )

        .merge(
            Scalar::with_url(
                "/docs",
                ApiDoc::openapi(),
            )
        )
}
```

---

# Integración Principal

```rust id="qj2o2n"
// apps/api/src/router.rs

pub fn build_router(
    state: AppState,
) -> Router {

    Router::new()

        .nest("/api/v1", api_router(state))

        .merge(docs_router())

        .route(
            "/health",
            get(health_handler),
        )
}
```

---

# Endpoints Generados

| Endpoint        | Propósito                |
| --------------- | ------------------------ |
| `/docs`         | Interfaz visual Scalar   |
| `/openapi.json` | Contrato OpenAPI oficial |

---

# IA-Ready

El archivo `/openapi.json` permite:

* generación automática de SDKs
* integración con agentes IA
* inspección automática de endpoints
* generación de tests
* generación de clientes TypeScript

---

# Frontend Type-Safe

## Generación automática de cliente TS

```bash id="m4f1dz"
pnpm add openapi-typescript-codegen
```

```bash id="0n1d7z"
openapi \
  --input http://localhost:8080/openapi.json \
  --output apps/web/src/lib/api
```

Resultado:

```text id="v2r8xk"
apps/web/src/lib/api/
 ├─ core/
 ├─ models/
 ├─ services/
 └─ index.ts
```

---

# Integración con CI

```yaml id="f4v33s"
# .github/workflows/ci.yml

- name: Validar OpenAPI
  run: |
    cargo run --bin api &
    sleep 5

    curl -f http://localhost:8080/openapi.json

- name: Lint OpenAPI
  run: spectral lint openapi.json
```

---

# Spectral — Validación OpenAPI

```bash id="8p1pcu"
pnpm add -D @stoplight/spectral-cli
```

```bash id="w7p7ry"
spectral lint openapi.json
```

Permite validar:

* naming conventions
* responses faltantes
* schemas inválidos
* seguridad inconsistente
* best practices

---

# Comparativa: Scalar vs Swagger UI

| Característica    | Swagger UI | Scalar      |
| ----------------- | ---------- | ----------- |
| Diseño            | Antiguo    | Moderno     |
| Rendimiento       | Pesado     | Muy ligero  |
| Bundle            | ~1MB       | ~100KB      |
| Búsqueda          | Básica     | Instantánea |
| UX                | Regular    | Excelente   |
| Persistencia auth | Limitada   | Sí          |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta                      | Propósito                          |
| -------------------------------- | ---------------------------------- |
| **`utoipa-redoc`**               | Alternativa UI moderna             |
| **`openapi-typescript-codegen`** | SDK TypeScript automático          |
| **`spectral`**                   | Linter OpenAPI                     |
| **`cargo-udeps`**                | Detectar dependencias innecesarias |
| **`scalar`**                     | Exploración moderna                |
| **`utoipa`**                     | Generación automática del spec     |

---

# Testing

## Validar generación del spec

```rust id="td6q2r"
#[test]
fn openapi_se_genera() {

    let spec = ApiDoc::openapi();

    assert!(
        spec.paths.paths.contains_key(
            "/api/v1/users"
        )
    );
}
```

---

# Seguridad

## Reglas

* `/docs` solo en development/staging
* `/openapi.json` sí disponible en producción
* Nunca exponer endpoints internos
* Nunca documentar secrets
* Nunca usar macros OpenAPI en dominio

---

# Performance

## Impacto real

| Componente   | RAM aprox         |
| ------------ | ----------------- |
| Utoipa       | ~0 runtime        |
| Scalar UI    | ~100KB            |
| openapi.json | Generación mínima |

Apto para VPS de 1GB RAM.

---

# Alternativas consideradas

| Opción              | Motivo de descarte |
| ------------------- | ------------------ |
| YAML manual         | Se desincroniza    |
| Swagger UI          | Pesado y anticuado |
| Postman Collections | No IA-ready        |
| Redoc standalone    | Menos interactivo  |
| Stoplight Studio    | Overkill para MVP  |

---

# Consecuencias

## ✅ Positivas

* El código Rust es la única fuente de verdad
* Los agentes IA entienden la API automáticamente
* Frontend genera tipos automáticamente
* Scalar mejora DX enormemente
* OpenAPI siempre sincronizado
* Validación en compilación

---

## ⚠️ Negativas / Trade-offs

### Verbosidad de macros

Las macros `#[utoipa::path]` aumentan tamaño de handlers.

→ Compensado con documentación sincronizada automáticamente.

→ `sintonia g module` puede generar boilerplate automáticamente.

---

### Registro manual

Cada endpoint debe registrarse en `ApiDoc`.

→ Si se olvida: el endpoint no aparece.

→ No rompe producción.

---

### Riesgo de documentar demasiado

Exponer endpoints internos accidentalmente.

→ Mantener DTOs separados.

→ Nunca derivar `ToSchema` en entidades internas.

---

# Decisiones derivadas

* `/docs` solo en development/staging
* `/openapi.json` disponible siempre
* PASETO se documenta explícitamente
* El frontend genera tipos desde OpenAPI
* Las macros viven solo en infraestructura
* `spectral lint` corre en CI
* `openapi.json` es parte del contrato oficial del sistema
