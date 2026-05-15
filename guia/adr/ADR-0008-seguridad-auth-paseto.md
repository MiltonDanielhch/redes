# ADR 0008 — Seguridad: Argon2id + PASETO v4 Local + Refresh Tokens

| Campo               | Valor                                                                                       |
| ------------------- | ------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                                  |
| **Fecha**           | 2026                                                                                        |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                            |
| **Relacionado con** | ADR 0003 (Axum middleware), ADR 0016 (Tracing / request_id), ADR 0004 (PostgreSQL + Docker) |

---

## Contexto

El sistema de monitoreo de infraestructura de red de la Gobernación del Beni requiere un esquema de autenticación moderno, seguro y completamente soberano.

JWT presenta riesgos conocidos:

* El header `alg` puede ser manipulado si la implementación es incorrecta
* El payload es visible en Base64 aunque esté firmado
* Muchas implementaciones permiten configuraciones inseguras

Necesitamos una solución que cumpla con:

* **Seguridad por defecto**
* **Payload cifrado**
* **Control total del sistema**
* **Compatibilidad con arquitectura hexagonal**
* **Sesiones auditables**
* **Escalabilidad para múltiples oficinas regionales**

---

## Decisión

Se utilizará:

* **argon2id** para hashing de contraseñas
* **PASETO v4 Local** para access tokens
* **Refresh Tokens opacos** persistidos en PostgreSQL
* **Rotación obligatoria de refresh tokens**
* **Middleware de autenticación en Axum**

JWT queda prohibido dentro del workspace.

---

## Stack Criptográfico

```toml
# crates/auth/Cargo.toml

argon2   = "0.5"
pasetors = { version = "0.7", features = ["v4"] }
secrecy  = "0.8"

# JWT prohibido
# jsonwebtoken = ❌
```

---

## 1 — Hash de Contraseñas: Argon2id

Se adopta Argon2id siguiendo recomendaciones OWASP 2024.

```rust
// crates/auth/src/password.rs

use argon2::{
    Argon2,
    PasswordHash,
    PasswordHasher,
    PasswordVerifier,
};

use argon2::password_hash::{
    rand_core::OsRng,
    SaltString,
};

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);

    Ok(
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::HashingFailed)?
            .to_string()
    )
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|_| AuthError::InvalidHash)?;

    Ok(
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    )
}
```

---

## 2 — Access Tokens con PASETO v4 Local

Los access tokens utilizan cifrado simétrico moderno:

* XChaCha20-Poly1305
* Payload completamente cifrado
* Sin cabecera `alg`
* Sin posibilidad de degradación criptográfica

```rust
// crates/auth/src/paseto.rs

use pasetors::claims::Claims;
use pasetors::v4::local::{
    decode,
    encode,
    LocalKey,
};

pub struct PasetoService {
    key: LocalKey,
}

impl PasetoService {
    pub fn new(secret: &str) -> Self {
        assert!(
            secret.len() == 32,
            "PASETO_SECRET debe tener exactamente 32 bytes"
        );

        let key = LocalKey::from(
            secret.as_bytes().try_into().unwrap()
        );

        Self { key }
    }

    pub fn generate_access_token(
        &self,
        user_id: &str,
    ) -> Result<String, AuthError> {

        let mut claims = Claims::new()
            .map_err(|_| AuthError::Internal)?;

        claims
            .add_additional("sub", user_id)
            .map_err(|_| AuthError::Internal)?;

        claims
            .expiration(
                &(OffsetDateTime::now_utc() + Duration::minutes(15))
            )
            .map_err(|_| AuthError::Internal)?;

        encode(
            &self.key,
            &claims,
            None,
            Some(b"redes-monitor-v1"),
        )
        .map_err(|_| AuthError::TokenCreation)
    }

    pub fn verify(
        &self,
        token: &str,
    ) -> Result<Claims, AuthError> {

        decode(
            &self.key,
            token,
            None,
            Some(b"redes-monitor-v1"),
        )
        .map_err(|_| AuthError::Unauthorized)
    }
}
```

---

## 3 — Refresh Tokens Persistidos en PostgreSQL

Los refresh tokens son opacos y persistidos en PostgreSQL.

Nunca se almacena el token plano.

```sql
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id            UUID PRIMARY KEY,
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash    TEXT NOT NULL UNIQUE,
    expires_at    TIMESTAMPTZ NOT NULL,
    created_at    TIMESTAMPTZ DEFAULT NOW(),
    revoked_at    TIMESTAMPTZ,
    replaced_by   UUID
);

CREATE INDEX idx_refresh_tokens_user
ON refresh_tokens(user_id);

CREATE INDEX idx_refresh_tokens_expiry
ON refresh_tokens(expires_at);
```

---

## 4 — Middleware de Autenticación en Axum

```rust
// crates/infrastructure/src/http/middleware/auth.rs

pub async fn auth_middleware(
    State(paseto): State<Arc<PasetoService>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {

    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = paseto
        .verify(token)
        .map_err(|_| AppError::Unauthorized)?;

    let user_id = claims
        .get_claim("sub")
        .and_then(|v| v.as_str())
        .ok_or(AppError::Unauthorized)?;

    tracing::Span::current()
        .record("user_id", user_id);

    req.extensions_mut()
        .insert(UserId(user_id.to_string()));

    Ok(next.run(req).await)
}
```

---

## Flujo de Autenticación

```text
Login
  ↓
Verificar password con Argon2id
  ↓
Generar Access Token PASETO (15 min)
  ↓
Generar Refresh Token opaco
  ↓
Guardar hash del refresh token en PostgreSQL
  ↓
Cliente consume API
  ↓
Access token expira
  ↓
Refresh token rota automáticamente
```

---

## Comparativa: JWT vs PASETO

| JWT                            | PASETO v4 Local          |
| ------------------------------ | ------------------------ |
| Header `alg` configurable      | Algoritmo fijo           |
| Payload visible                | Payload cifrado          |
| Riesgo de mala configuración   | Seguridad por defecto    |
| Amplia superficie de ataque    | Superficie mínima        |
| Requiere validaciones manuales | API criptográfica segura |

---

## Herramientas y Librerías Complementarias

| Herramienta        | Propósito                                |
| ------------------ | ---------------------------------------- |
| `secrecy`          | Evita filtrado accidental de secretos    |
| `constant_time_eq` | Comparación segura contra timing attacks |
| `tower-sessions`   | Manejo de sesiones HTTP                  |
| `tracing`          | Auditoría y correlación de requests      |
| `uuid` v7          | IDs ordenables para sesiones y tokens    |

---

## Consideraciones Regionales — Gobernación del Beni

Esta arquitectura permite:

* Operación completamente local sin depender de terceros
* Control total de credenciales institucionales
* Auditoría de accesos desde oficinas regionales
* Escalabilidad para múltiples sedes
* Integración futura con LDAP/Active Directory sin modificar el dominio

---

## Alternativas consideradas

| Opción                  | Motivo de descarte                                         |
| ----------------------- | ---------------------------------------------------------- |
| JWT                     | Payload visible y mayor superficie de ataque               |
| Cookies de sesión puras | Requieren infraestructura adicional para escalar           |
| Auth0 / Clerk           | Dependencia externa incompatible con soberanía tecnológica |

---

## Consecuencias

### ✅ Positivas

* Criptografía moderna y segura
* Payload cifrado
* Tokens imposibles de degradar criptográficamente
* Rotación automática de refresh tokens
* Arquitectura compatible con auditoría institucional
* Sin dependencia de servicios externos

### ⚠️ Negativas / Trade-offs

* Ecosistema PASETO menor que JWT
  → Mitigación: encapsular toda la lógica dentro de `crates/auth`

* Mayor complejidad inicial
  → Mitigación: centralizar middleware y generación de tokens

* Requiere gestión segura de secretos
  → Mitigación: variables de entorno + Docker Secrets

---

## Decisiones derivadas

* `jsonwebtoken` queda prohibido en el workspace
* `PASETO_SECRET` se valida al arrancar la aplicación
* Todos los endpoints protegidos usan middleware Axum
* Los refresh tokens se almacenan únicamente hasheados
* El cleanup de tokens expirados corre mediante job programado
* El `user_id` queda registrado en tracing para auditoría y observabilidad
