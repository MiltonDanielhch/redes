# ADR 0008 — Seguridad: Argon2id + PASETO v4 Local + Refresh Tokens

| Campo               | Valor                                                                                       |
| ------------------- | ------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                                  |
| **Fecha**           | 2026-05-16                                                                                  |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                            |
| **Relacionado con** | ADR 0003 (Axum middleware), ADR 0006 (RBAC/Sessions/Audit), ADR 0004 (PostgreSQL) |
| **Última revisión** | 2026-05-16 — Actualización API pasetors 0.7.8 + alineación con ROADMAP-AUTH |

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

argon2   = "0.5.3"
pasetors = { version = "0.7.8", features = ["v4", "std"] }
secrecy  = "0.10.3"
rand_core = { version = "0.6", features = ["getrandom"] }

# JWT prohibido — cargo-deny lo bloquea
# jsonwebtoken = ❌
```

**Nota:** `pasetors 0.7.8` requiere Rust ≥ 1.88.0. El proyecto usa 1.86.0 (Edition 2024), 
por lo cual se actualizará el toolchain a 1.88.0+ antes de implementar este ADR.

---

## 1 — Hash de Contraseñas: Argon2id (OWASP 2024)

Parámetros fijos para el proyecto:
- **Variante:** Argon2id
- **Memory:** 19,456 KiB (19 MiB)
- **Iterations:** 2
- **Parallelism:** 1
- **Salt length:** 16 bytes (recomendado por password-hash)
- **Output length:** 32 bytes

```rust
//! Ubicación: `crates/auth/src/password.rs`
//!
//! Descripción: Hashing y verificación de contraseñas con Argon2id siguiendo
//!              parámetros OWASP 2024. Usa timing-safe comparison.
//!
//! ADRs: 0008

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{rand_core::OsRng, SaltString},
};

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Argon2id por defecto

    Ok(
        argon2
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

## 2 — Access Tokens con PASETO v4 Local (API 0.7.8)

```rust
//! Ubicación: `crates/auth/src/paseto.rs`
//!
//! Descripción: Servicio PASETO v4 Local para access tokens. Usa XChaCha20-Poly1305.
//!              API actualizada a pasetors 0.7.8 (SymmetricKey + local::encrypt/decrypt).
//!
//! ADRs: 0008

use pasetors::{
    claims::{Claims, ClaimsValidationRules},
    keys::{Generate, SymmetricKey},
    local, Local, version4::V4,
    token::UntrustedToken,
};
use secrecy::{ExposeSecret, SecretBox, SecretString};
use time::{Duration, OffsetDateTime};

pub struct PasetoService {
    key: SecretBox<SymmetricKey<V4>>,
}

impl PasetoService {
    pub fn new(secret: &SecretString) -> Result<Self, AuthError> {
        let secret_bytes = secret.expose_secret().as_bytes();

        if secret_bytes.len() != 32 {
            return Err(AuthError::InvalidSecretLength);
        }

        let key = SymmetricKey::<V4>::try_from(secret_bytes)
            .map_err(|_| AuthError::InvalidSecret)?;

        Ok(Self {
            key: SecretBox::new(Box::new(key)),
        })
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

        local::encrypt(
            self.key.expose_secret(),
            &claims,
            None,
            Some(b"redes-monitor-v1"),
        )
        .map_err(|_| AuthError::TokenCreation)
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let validation_rules = ClaimsValidationRules::new();
        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|_| AuthError::Unauthorized)?;

        let trusted_token = local::decrypt(
            self.key.expose_secret(),
            &untrusted_token,
            &validation_rules,
            None,
            Some(b"redes-monitor-v1"),
        )
        .map_err(|_| AuthError::Unauthorized)?;

        trusted_token
            .payload_claims()
            .cloned()
            .ok_or(AuthError::Unauthorized)
    }
}
```

**Cambios clave vs versión anterior:**
- `LocalKey` → `SymmetricKey<V4>`
- `encode/decode` → `local::encrypt/local::decrypt`
- `ClaimsValidationRules` requerido para `decrypt`
- `UntrustedToken` intermediario para parsing seguro
- `SecretBox` de `secrecy 0.10` para proteger la key en memoria

---

## 3 — Refresh Tokens Persistidos en PostgreSQL

Alineado con ROADMAP-AUTH-FULLSTACK.md (A.5):

```sql
CREATE TABLE IF NOT EXISTS sessions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash    TEXT NOT NULL,
    ip_address    INET,
    user_agent    TEXT,
    expires_at    TIMESTAMPTZ NOT NULL,
    created_at    TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(user_id, token_hash)
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expiry ON sessions(expires_at);
CREATE INDEX idx_sessions_hash ON sessions(token_hash);
```

**Nota:** El hash del refresh token usa SHA-256 (32 bytes hex). Nunca se almacena el token plano.
**Rotación:** Un refresh token se usa UNA VEZ. Al usarlo, se crea uno nuevo y se marca el anterior como revocado (eliminado).

---

## 4 — Middleware de Autenticación en Axum

```rust
//! Ubicación: `crates/infrastructure/src/http/middleware/auth.rs`
//!
//! Descripción: Middleware Axum que extrae Bearer token, verifica PASETO v4,
//!              rechaza JWT explícitamente, inyecta UserId en extensions.
//!
//! ADRs: 0003, 0008

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use secrecy::ExposeSecret;

pub async fn auth_middleware(
    State(paseto): State<std::sync::Arc<PasetoService>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    // Rechazo explícito de JWT — ADR 0008
    if token.starts_with("eyJ") {
        tracing::warn!(token_prefix = %&token[..10], "JWT detectado y rechazado");
        return Err(AppError::Unauthorized);
    }

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

| Herramienta        | Propósito                                | Versión proyecto |
| ------------------ | ---------------------------------------- | ---------------- |
| `secrecy`          | Protección de secretos en memoria        | 0.10.3           |
| `constant_time_eq` | Comparación segura contra timing attacks | (via argon2)     |
| `rand_core`        | RNG seguro para salts y tokens opacos    | 0.6              |
| `tracing`          | Auditoría y correlación de requests      | workspace        |
| `uuid` v7          | IDs ordenables para sesiones y tokens    | workspace        |

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

* **MSRV elevado por pasetors:** Requiere Rust 1.88.0+ (vs 1.86 del proyecto)
  → Mitigación: Actualizar rust-toolchain.toml antes de implementar este ADR

* **Ecosistema PASETO menor que JWT**
  → Mitigación: encapsular toda la lógica dentro de `crates/auth`

* **Mayor complejidad inicial**
  → Mitigación: centralizar middleware y generación de tokens

* **Requiere gestión segura de secretos**
  → Mitigación: `secrecy` + variables de entorno + Docker Secrets

---

## Decisiones derivadas

1. `jsonwebtoken` queda prohibido en el workspace (cargo-deny lo bloquea)
2. `PASETO_SECRET` se valida al arrancar la aplicación: exactamente 32 bytes, fail-fast si falta
3. Todos los endpoints protegidos usan middleware Axum con reject JWT
4. Los refresh tokens se almacenan únicamente hasheados (SHA-256)
5. El cleanup de tokens expirados corre mediante job programado (Apalis)
6. El `user_id` queda registrado en tracing spans para auditoría y observabilidad
7. **NUEVO:** Rust toolchain se actualiza a 1.88.0+ antes de implementar auth
8. **NUEVO:** `pasetors` usa features `["v4", "std"]` (no solo `"v4"`)
