# ADR 0010 — Testing: 4 Capas + cargo-nextest

| Campo               | Valor                                                                                                                          |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                                                     |
| **Fecha**           | 2026                                                                                                                           |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                               |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0004 (PostgreSQL + SQLx), ADR 0008 (PASETO + Auth), ADR 0015 (Monitoreo) |

---

# Contexto

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

# Decisión

Adoptar una estrategia de **4 capas de testing**, donde cada capa valida una responsabilidad específica del sistema.

La arquitectura de testing queda dividida en:

| Capa            | Objetivo                     |
| --------------- | ---------------------------- |
| 1 — Dominio     | Reglas puras de negocio      |
| 2 — Aplicación  | Casos de uso y orquestación  |
| 3 — Integración | SQL real y repositorios      |
| 4 — E2E         | Flujos completos del sistema |

El runner oficial del proyecto será:

```bash id="h6m4yi"
cargo-nextest
```

---

# Dependencias base

```toml id="upg9mg"
# Cargo.toml (workspace)

[workspace.dev-dependencies]

tokio      = { version = "1", features = ["test", "macros"] }
mockall    = "0.13"
reqwest    = { version = "0.12", features = ["json"] }
httpmock   = "0.7"
insta      = "1"
proptest   = "1"
```

---

# Runner oficial

```bash id="0v0bf0"
cargo install cargo-nextest
```

Ventajas:

* paralelismo real
* mejor output
* retries automáticos
* aislamiento entre tests
* 3–5x más rápido que `cargo test`

---

# Capa 1 — Tests Unitarios de Dominio

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

```rust id="i2f8xe"
// crates/domain/src/value_objects/email.rs

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

# Capa 2 — Tests de Aplicación con Mocks

## Objetivo

Validar casos de uso y flujo de aplicación.

## Características

* async
* mocks con `mockall`
* sin DB real
* verifica orquestación

---

## Ejemplo

```rust id="b3xjlwm"
// crates/application/src/use_cases/auth/register.rs

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

# Capa 3 — Tests de Integración

## Objetivo

Validar:

* queries SQL
* migraciones
* índices
* repositorios
* contratos reales con PostgreSQL

---

# Base de datos de testing

Para integración se usa PostgreSQL aislado para reproducir el entorno real.

Opciones válidas:

* PostgreSQL local
* Docker temporal
* `testcontainers`
* CI service container

---

## Setup recomendado

```rust id="l3uv2f"
async fn setup_test_db() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("TEST_DATABASE_URL").unwrap())
        .await
        .expect("failed to connect test database");

    sqlx::migrate!("../../data/migrations")
        .run(&pool)
        .await
        .expect("failed migrations");

    pool
}
```

---

## Ejemplo

```rust id="1gvjng"
// crates/database/tests/user_repository_test.rs

#[tokio::test]
async fn guardar_y_recuperar_usuario() {
    let pool = setup_test_db().await;

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

---

# Capa 4 — Tests End-to-End (E2E)

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

```rust id="r8h4zv"
// apps/api/tests/auth_flow_test.rs

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

    // Request autenticado
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

# Estructura oficial

```text id="h97yte"
crates/
├── domain/
│   └── src/**              # Capa 1
│
├── application/
│   └── src/**              # Capa 2
│
├── database/
│   └── tests/**            # Capa 3
│
apps/
└── api/
    └── tests/**            # Capa 4
```

---

# Estrategia de ejecución

| Comando         | Capas     |
| --------------- | --------- |
| `just test`     | 1 + 2 + 3 |
| `just test-e2e` | 4         |
| `just test-all` | Todas     |
| CI              | Todas     |

---

# Configuración CI

```yaml id="53vfrp"
# .github/workflows/ci.yml

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/

          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install nextest
        run: cargo install cargo-nextest

      - name: Run tests
        run: cargo nextest run

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: SQLX Offline
        run: just prepare && git diff --exit-code .sqlx/

      - name: Security audit
        run: |
          cargo install cargo-deny cargo-audit
          cargo deny check
          cargo audit
```

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta      | Propósito           |
| ---------------- | ------------------- |
| `cargo-nextest`  | Runner ultra rápido |
| `cargo-mutants`  | Mutation testing    |
| `proptest`       | Property testing    |
| `insta`          | Snapshot testing    |
| `cargo-llvm-cov` | Cobertura visual    |
| `testcontainers` | PostgreSQL efímero  |
| `httpmock`       | Simulación HTTP     |
| `mockall`        | Mocks type-safe     |

---

# Consecuencias

## ✅ Positivas

* Feedback ultra rápido en desarrollo
* Cada capa se prueba en el nivel correcto
* Tests aislados y mantenibles
* CI reproducible
* Alta cobertura de edge cases
* Arquitectura hexagonal naturalmente testeable

---

## ⚠️ Negativas / Trade-offs

### Mayor cantidad de archivos

La separación por capas aumenta la estructura del proyecto.

→ Mitigado con generadores CLI (`sintonia g module`)
→ Beneficio enorme en mantenibilidad

---

### Tests E2E más lentos

Los tests completos tardan más.

→ Solo corren en CI y antes de releases
→ El loop local sigue siendo rápido

---

### Mocks verbosos

`mockall` puede generar mucho boilerplate.

→ Centralizar helpers compartidos
→ Si el mock es enorme, el trait probablemente está mal diseñado

---

# Decisiones derivadas

* `cargo-nextest` es el runner oficial
* `just test` no ejecuta E2E
* Los tests E2E verifican explícitamente PASETO (`v4.local.`)
* El CI bloquea vulnerabilidades con `cargo audit`
* El dominio debe poder testearse sin infraestructura externa
* PostgreSQL de testing usa migraciones reales del proyecto
