# ADR 0010 — Testing: 4 Capas + cargo-nextest

| Campo               | Valor                                                                                                                          |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                                                     |
| **Fecha**           | 2026-05-16                                                                                                                     |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                               |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0004 (PostgreSQL + SQLx), ADR 0008 (PASETO + Auth), ADR 0015 (Monitoreo) |
| **Última revisión** | 2026-05-16 — Actualización de versiones + configuración nextest |

---

## Contexto

Sin una estrategia clara de testing, los proyectos terminan dependiendo únicamente de tests E2E lentos y frágiles.

Esto provoca:

* feedback lento para el developer
* pipelines de CI pesados
* baja cobertura de edge cases
* dificultad para aislar bugs
* regresiones difíciles de detectar

Necesitamos una estrategia que:

* mantenga feedback rápido durante el desarrollo
* permita aislar cada capa correctamente
* no dependa de Docker para correr localmente
* sea escalable a medida que crecen los módulos
* funcione bien con arquitectura hexagonal

---

## Decisión

Adoptar una estrategia de **4 capas de testing**, donde cada capa valida una responsabilidad específica del sistema.

La arquitectura de testing queda dividida en:

| Capa            | Objetivo                     |
| --------------- | ---------------------------- |
| 1 — Dominio     | Reglas puras de negocio      |
| 2 — Aplicación  | Casos de uso y orquestación  |
| 3 — Integración | SQL real y repositorios      |
| 4 — E2E         | Flujos completos del sistema |

El runner oficial del proyecto será:

```bash
cargo install cargo-nextest --locked
```

**Versión mínima:** 0.9.135 (mayo 2026)

---

## Dependencias base

```toml
# Cargo.toml (workspace)

[workspace.dev-dependencies]

tokio      = { version = "1.45", features = ["rt-multi-thread", "macros"] }
mockall    = "0.14.0"
reqwest    = { version = "0.13.3", features = ["json", "cookies"] }
httpmock   = "0.8"
insta      = "1.46"
proptest   = "1.9"
fake       = { version = "4.0", features = ["derive", "chrono", "uuid"] }
sqlx       = { version = "0.8.6", features = ["runtime-tokio", "postgres", "chrono", "uuid", "migrate"] }
```

**Notas de versión:**
- `tokio`: feature `"test"` deprecado, usar `"rt-multi-thread"` + `"macros"`
- `cargo-nextest`: instalar siempre con `--locked` para evitar incompatibilidades
- `fake`: generación de datos de prueba (emails, nombres, UUIDs)
- `mockall`: versión 0.14.0 (feb 2026). Soporte mejorado para traits async con `#[async_trait]` y async nativo de Rust 2024.citeweb_search:22#0
- `reqwest`: versión 0.13.3 (abr 2026). `rustls` como TLS backend por defecto.citeweb_search:22#1
- `proptest`: versión 1.9 (feb 2026). Property testing.citeweb_search:22#6
- `cargo-mutants`: versión 27.0.0 (mar 2026). Mutation testing. MSRV 1.88.citeweb_search:22#2web_search:22#3
- `cargo-llvm-cov`: versión 0.8.7 (may 2026). Cobertura de código con LLVM.citeweb_search:22#5web_search:22#7

---

## Runner oficial

```bash
cargo install cargo-nextest --locked
# o en CI:
# uses: taiki-e/install-action@nextest
```

Ventajas:

* paralelismo real
* mejor output
* retries automáticos (configurable)
* aislamiento entre tests
* 3–5x más rápido que `cargo test`
* soporte para benchmarks (experimental)
* grabación y replay de test runs

---

## Configuración de perfiles (`.config/nextest.toml`)

```toml
[profile.default]
retries = 0
slow-timeout = { period = "60s", terminate-after = 2 }

[profile.ci]
retries = 2
slow-timeout = { period = "120s", terminate-after = 2 }
test-threads = "num-cpus"

[profile.e2e]
test-threads = 2          # E2E no paraleliza tanto (comparten DB)
slow-timeout = { period = "300s", terminate-after = 1 }
```

---

## Capa 1 — Tests Unitarios de Dominio

## Objetivo

Validar reglas de negocio puras.

## Características

* sin async
* sin DB
* sin HTTP
* sin mocks externos
* ejecución en milisegundos

---

## Ejemplo

```rust
//! Ubicación: `crates/domain/src/value_objects/email.rs`
//!
//! Descripción: Tests unitarios de dominio — reglas puras de negocio.
//!
//! ADRs: 0010, 0001

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_valido_se_crea() {
        assert!(Email::new("user@example.com").is_ok());
    }

    #[test]
    fn email_se_normaliza() {
        let email = Email::new("User@EXAMPLE.COM").unwrap();

        assert_eq!(
            email.as_str(),
            "user@example.com"
        );
    }

    #[test]
    fn email_sin_arroba_falla() {
        assert!(matches!(
            Email::new("userexample.com").unwrap_err(),
            DomainError::InvalidEmail(_)
        ));
    }

    #[test]
    fn usuario_soft_delete_funciona() {
        let mut user = User::new(
            Email::new("u@example.com").unwrap(),
            PasswordHash::from_hash("hash".into()),
        );

        assert!(user.is_active());

        user.soft_delete();

        assert!(!user.is_active());
        assert!(user.deleted_at.is_some());
    }
}
```

---

## Capa 2 — Tests de Aplicación con Mocks

## Objetivo

Validar casos de uso y flujo de aplicación.

## Características

* async
* mocks con `mockall`
* sin DB real
* verifica orquestación

---

## Ejemplo

```rust
//! Ubicación: `crates/application/src/use_cases/auth/register.rs`
//!
//! Descripción: Tests de aplicación con mocks — valida orquestación de casos de uso.
//!
//! ADRs: 0010, 0001

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};

    mock! {
        UserRepo {}

        impl UserRepository for UserRepo {
            async fn find_active_by_email(
                &self,
                email: &Email
            ) -> Result<Option<User>, DomainError>;

            async fn save(
                &self,
                user: &User
            ) -> Result<(), DomainError>;
        }
    }

    #[tokio::test]
    async fn registro_correcto() {
        let mut mock = MockUserRepo::new();

        mock.expect_find_active_by_email()
            .once()
            .returning(|_| Ok(None));

        mock.expect_save()
            .once()
            .returning(|_| Ok(()));

        let result = RegisterUseCase::new(
            Arc::new(mock),
            fake_hash_fn(),
        )
        .execute(RegisterInput {
            email:    "nuevo@example.com".into(),
            password: "Password123!".into(),
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn email_duplicado_falla() {
        let mut mock = MockUserRepo::new();

        mock.expect_find_active_by_email()
            .once()
            .returning(|_| Ok(Some(fake_user())));

        mock.expect_save().never();

        let result = RegisterUseCase::new(
            Arc::new(mock),
            fake_hash_fn(),
        )
        .execute(RegisterInput {
            email:    "existente@example.com".into(),
            password: "Password123!".into(),
        })
        .await;

        assert!(matches!(
            result.unwrap_err(),
            DomainError::EmailAlreadyExists
        ));
    }
}
```

---

## Capa 3 — Tests de Integración

## Objetivo

Validar:

* queries SQL
* migraciones
* índices
* repositorios
* contratos reales con PostgreSQL

---

## Base de datos de testing

Para integración se usa PostgreSQL aislado para reproducir el entorno real.

Opciones válidas (en orden de preferencia):

1. **`sqlx test`** — macro `#[sqlx::test]` que crea DB temporal por test (recomendado)
2. **PostgreSQL local** — `TEST_DATABASE_URL` apuntando a DB dedicada
3. **Docker temporal** — script que levanta PostgreSQL en puerto efímero
4. **`testcontainers`** — para CI cuando Docker está disponible

---

## Setup recomendado (sqlx test)

```rust
//! Ubicación: `crates/database/tests/user_repository_test.rs`
//!
//! Descripción: Tests de integración con PostgreSQL real via sqlx::test.
//!
//! ADRs: 0010, 0004

use sqlx::PgPool;

#[sqlx::test]
async fn guardar_y_recuperar_usuario(pool: PgPool) {
    let repo = PostgresUserRepository::new(
        Arc::new(pool)
    );

    let user = fake_user("test@example.com");

    repo.save(&user).await.unwrap();

    let found = repo
        .find_active_by_email(
            &Email::new("test@example.com").unwrap()
        )
        .await
        .unwrap();

    assert!(found.is_some());
}
```

**Nota:** `#[sqlx::test]` requiere feature `"migrate"` en `sqlx`. Crea una DB temporal por test, aplica migraciones automáticamente, y la limpia al final.

---

## Capa 4 — Tests End-to-End (E2E)

## Objetivo

Validar flujos completos reales.

Incluye:

* servidor HTTP real
* middleware
* auth
* DB
* serialización JSON
* rate limiting
* permisos

---

## Ejemplo

```rust
//! Ubicación: `apps/api/tests/auth_flow_test.rs`
//!
//! Descripción: Tests E2E — flujo completo de autenticación.
//!              Verifica PASETO, no JWT.
//!
//! ADRs: 0010, 0003, 0008

#[tokio::test]
async fn flujo_completo_auth() {
    let base_url = spawn_test_server().await;

    let client = reqwest::Client::new();

    // Registro
    let res = client
        .post(format!("{}/auth/register", base_url))
        .json(&json!({
            "email": "e2e@example.com",
            "password": "Password123!"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 201);

    // Login
    let body = res.json::<serde_json::Value>().await.unwrap();

    let access_token = body["access_token"]
        .as_str()
        .unwrap();

    // Debe ser PASETO — nunca JWT
    assert!(
        access_token.starts_with("v4.local.")
    );

    // Request autenticada
    let res = client
        .get(format!("{}/api/v1/users/me", base_url))
        .bearer_auth(access_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}
```

---

## Helpers compartidos para tests

```rust
//! Ubicación: `crates/test-helpers/src/lib.rs`
//!
//! Descripción: Utilidades compartidas para todos los tests del workspace.
//!
//! ADRs: 0010

use fake::{faker::internet::en::SafeEmail, Fake};
use uuid::Uuid;

/// Genera un email de prueba único
pub fn fake_email() -> String {
    format!("test-{}@example.com", Uuid::new_v4())
}

/// Genera un usuario de prueba con datos realistas
pub fn fake_user(email: &str) -> User {
    User::new(
        Email::new(email).unwrap(),
        PasswordHash::from_hash("$argon2id$v=19$m=19456,t=2,p=1$...".into()),
    )
}

/// Genera un hash de password válido para tests (no verifica)
pub fn fake_hash_fn() -> impl Fn(&str) -> String {
    |_password: &str| -> String {
        "$argon2id$v=19$m=19456,t=2,p=1$fake$fake".into()
    }
}

/// Levanta servidor de test con DB temporal
pub async fn spawn_test_server() -> String {
    // ...
}
```

---

## Estructura oficial

```text
crates/
├── domain/
│   └── src/**              # Capa 1 — unit tests inline
│
├── application/
│   └── src/**              # Capa 2 — unit tests inline con mocks
│
├── database/
│   └── tests/**            # Capa 3 — integration tests (sqlx::test)
│
├── test-helpers/           # Helpers compartidos (fake data, spawn server)
│   └── src/lib.rs
│
apps/
└── api/
    └── tests/**            # Capa 4 — E2E tests
```

---

## Estrategia de ejecución

| Comando | Capas | Perfil nextest |
| --- | --- | --- |
| `just test` | 1 + 2 + 3 | `default` |
| `just test-e2e` | 4 | `e2e` |
| `just test-all` | Todas | `ci` |
| CI | Todas | `ci` |

**justfile:**
```just
test:
    cargo nextest run --profile default -E 'not test(e2e)'

test-e2e:
    cargo nextest run --profile e2e -E 'test(e2e)'

test-all:
    cargo nextest run --profile ci
```

---

## Configuración CI

```yaml
# .github/workflows/ci.yml

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - uses: taiki-e/install-action@nextest

      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit + integration tests
        run: cargo nextest run --profile ci -E 'not test(e2e)'

      - name: Run E2E tests
        run: cargo nextest run --profile e2e -E 'test(e2e)'

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: SQLX Offline
        run: just prepare && git diff --exit-code .sqlx/

      - name: Security audit
        run: |
          cargo install cargo-deny cargo-audit --locked
          cargo deny check
          cargo audit
```

---

## Herramientas y Librerías

| Herramienta | Propósito | Versión |
| --- | --- | --- |
| `cargo-nextest` | Runner ultra rápido | 0.9.135 |
| `cargo-mutants` | Mutation testing | 27.0.0 |
| `proptest` | Property testing | 1.9 |
| `insta` | Snapshot testing | 1.46 |
| `cargo-llvm-cov` | Cobertura visual | 0.8.7 |
| `sqlx::test` | PostgreSQL efímero por test | 0.8.6 |
| `httpmock` | Simulación HTTP | 0.8 |
| `mockall` | Mocks type-safe | 0.14.0 |
| `fake` | Generación de datos de prueba | 4.0 |

---

## Consecuencias

### ✅ Positivas

* Feedback ultra rápido en desarrollo
* Cada capa se prueba en el nivel correcto
* Tests aislados y mantenibles
* CI reproducible
* Alta cobertura de edge cases
* Arquitectura hexagonal naturalmente testeable

### ⚠️ Negativas / Trade-offs

**Mayor cantidad de archivos**
La separación por capas aumenta la estructura del proyecto.
→ Mitigado con generadores CLI (`sintonia g module`)
→ Beneficio enorme en mantenibilidad

**Tests E2E más lentos**
Los tests completos tardan más.
→ Solo corren en CI y antes de releases
→ El loop local sigue siendo rápido

**Mocks verbosos**
`mockall` puede generar mucho boilerplate.
→ Centralizar helpers compartidos
→ Si el mock es enorme, el trait probablemente está mal diseñado

---

## Decisiones derivadas

* `cargo-nextest` es el runner oficial (versión 0.9.135+)
* `just test` no ejecuta E2E (usa `--profile default`)
* Los tests E2E verifican explícitamente PASETO (`v4.local.`)
* El CI bloquea vulnerabilidades con `cargo audit`
* El dominio debe poder testearse sin infraestructura externa
* PostgreSQL de testing usa `#[sqlx::test]` (DB temporal por test)
* `testcontainers` como fallback para CI sin PostgreSQL local
* `fake` crate para generación de datos de prueba consistentes

---

## Notas de actualización de versiones (2026-05-16)

| Componente | Versión/Config | Notas |
|------------|----------------|-------|
| **cargo-nextest** | **0.9.135** | Última estable (may 2026). MSRV 1.91. Features: grabación/replay de tests, benchmarks experimentales, Chrome trace export, `group()` filterset. |
| **cargo-mutants** | **27.0.0** | Última estable (mar 2026). MSRV 1.88. Mutation testing. `--Zmutate-file` para debug de mutaciones. |
| **mockall** | **0.14.0** | Última estable (feb 2026). Soporte mejorado para traits async con `#[async_trait]` y async nativo Rust 2024. |
| **reqwest** | **0.13.3** | Última estable (abr 2026). `rustls` como TLS backend por defecto. Features `json`, `cookies`. |
| **proptest** | **1.9** | Última estable (feb 2026). Property testing. `RngAlgorithm::Recorder` para capturar datos random. |
| **insta** | **1.46** | Última estable. Snapshot testing. |
| **cargo-llvm-cov** | **0.8.7** | Última estable (may 2026). Cobertura LLVM. Mejoras en compilación y soporte proc-macro. |
| **fake** | **4.0** | Última estable. Generación de datos de prueba. Features: `derive`, `chrono`, `uuid`. |
| **sqlx::test** | **0.8.6** | Macro `#[sqlx::test]`. Feature `migrate` requerida. |
| **tokio** | **1.45** | Serie 1.x estable. Features: `rt-multi-thread`, `macros`. |
