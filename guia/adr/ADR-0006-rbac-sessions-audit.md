# ADR 0006 — RBAC, Sesiones y Auditoría

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0002 (Configuración), ADR 0003 (Stack Backend), ADR 0004 (PostgreSQL), ADR 0005 (Migraciones), ADR 0008 (Seguridad PASETO), ADR 0016 (OpenAPI) |

---

## Contexto

El sistema de monitoreo de infraestructura de red de la Gobernación del Beni
maneja datos sensibles:

* métricas de red institucional,
* inventario de dispositivos,
* alertas de seguridad,
* detección de intrusos,
* configuración de agentes.

Necesitamos un modelo de seguridad que permita:

* control de acceso granular,
* trazabilidad completa,
* cumplimiento institucional,
* y auditoría permanente.

---

## Decisión

Usar:

* **RBAC** (Role-Based Access Control) con permisos granulares
* **sesiones con PASETO v4** (no JWT, no cookies sin firma)
* **Soft Delete** en todas las entidades
* **auditoría obligatoria** en toda mutación

---

## Modelo de datos

### Usuarios

```sql
-- Migración: create_users_table

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,                    -- Nombre completo (no username)
    is_active BOOLEAN NOT NULL DEFAULT FALSE, -- FALSE hasta verificar email
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ               -- Soft Delete
);

-- Índices
CREATE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_active ON users(is_active) WHERE deleted_at IS NULL;

-- Trigger updated_at
CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION trg_updated_at();
```

**Nota:** No hay `username` ni `avatar_url`. El dominio usa `email` como identificador único y `name` para display.

---

### Roles

```sql
-- Migración: create_roles_table

CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ               -- Soft Delete
);

CREATE INDEX idx_roles_name ON roles(name) WHERE deleted_at IS NULL;

CREATE TRIGGER trg_roles_updated_at
    BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION trg_updated_at();
```

---

### Permisos

```sql
-- Migración: create_permissions_table

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    name TEXT NOT NULL UNIQUE,          -- Formato: "recurso:acción"
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ               -- Soft Delete
);

CREATE INDEX idx_permissions_name ON permissions(name) WHERE deleted_at IS NULL;

CREATE TRIGGER trg_permissions_updated_at
    BEFORE UPDATE ON permissions
    FOR EACH ROW EXECUTE FUNCTION trg_updated_at();
```

---

### Asignación de roles a usuarios

```sql
-- Migración: create_user_roles_table

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);
```

---

### Asignación de permisos a roles

```sql
-- Migración: create_role_permissions_table

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, permission_id)
);
```

---

### Tokens (verificación email, password reset)

```sql
-- Migración: create_tokens_table

CREATE TYPE token_purpose AS ENUM (
    'email_verification',
    'password_reset'
);

CREATE TABLE IF NOT EXISTS tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,             -- SHA-256 del token raw (nunca almacenar raw)
    purpose token_purpose NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tokens_user_purpose ON tokens(user_id, purpose);
CREATE INDEX idx_tokens_expires ON tokens(expires_at);
```

**Nota:** `token_hash` almacena SHA-256 del token raw. El token raw solo existe en memoria/email. Nunca se almacena en texto plano.

---

### Sesiones

```sql
-- Migración: create_sessions_table

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,      -- SHA-256 del refresh token (nunca raw)
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
```

**Nota:** `token_hash` almacena SHA-256 del refresh token. El token raw (PASETO v4) se envía al cliente. Si la DB se compromete, los hashes no permiten suplantación (requieren preimagen).

---

### Logs de auditoría

```sql
-- Migración: create_audit_logs_table

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action TEXT NOT NULL,                 -- ej: "device_created", "alert_acknowledged"
    resource TEXT NOT NULL,               -- ej: "devices", "alerts"
    resource_id UUID,                     -- ID del recurso afectado
    details JSONB,                       -- Payload adicional (flexible)
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índices críticos para queries frecuentes
CREATE INDEX idx_audit_user ON audit_logs(user_id, created_at DESC);
CREATE INDEX idx_audit_resource ON audit_logs(resource, resource_id, created_at DESC);
CREATE INDEX idx_audit_action ON audit_logs(action, created_at DESC);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);

-- Índice GIN para búsquedas en JSONB
CREATE INDEX idx_audit_details ON audit_logs USING GIN (details);
```

---

## Seed de sistema (roles y permisos)

```sql
-- data/seeds/system/001_roles.sql

INSERT INTO roles (id, name, description, created_at)
VALUES
    (uuid_generate_v7(), 'admin', 'Acceso total al sistema', NOW()),
    (uuid_generate_v7(), 'operator', 'Operador de red', NOW()),
    (uuid_generate_v7(), 'viewer', 'Solo lectura', NOW()),
    (uuid_generate_v7(), 'agent', 'Agente de monitoreo', NOW())
ON CONFLICT (name) DO NOTHING;
```

```sql
-- data/seeds/system/002_permissions.sql

INSERT INTO permissions (id, name, description, created_at)
VALUES
    (uuid_generate_v7(), 'users:read', 'Ver usuarios', NOW()),
    (uuid_generate_v7(), 'users:write', 'Crear/editar usuarios', NOW()),
    (uuid_generate_v7(), 'users:delete', 'Archivar usuarios', NOW()),
    (uuid_generate_v7(), 'devices:read', 'Ver dispositivos', NOW()),
    (uuid_generate_v7(), 'devices:write', 'Crear/editar dispositivos', NOW()),
    (uuid_generate_v7(), 'devices:delete', 'Archivar dispositivos', NOW()),
    (uuid_generate_v7(), 'sedes:read', 'Ver sedes', NOW()),
    (uuid_generate_v7(), 'sedes:write', 'Crear/editar sedes', NOW()),
    (uuid_generate_v7(), 'alerts:read', 'Ver alertas', NOW()),
    (uuid_generate_v7(), 'alerts:write', 'Acknowledge/resolve alertas', NOW()),
    (uuid_generate_v7(), 'intrusions:read', 'Ver intrusiones', NOW()),
    (uuid_generate_v7(), 'intrusions:write', 'Resolver intrusiones', NOW()),
    (uuid_generate_v7(), 'audit:read', 'Ver audit logs', NOW()),
    (uuid_generate_v7(), 'audit:export', 'Exportar audit logs', NOW()),
    (uuid_generate_v7(), 'topology:read', 'Ver topología', NOW()),
    (uuid_generate_v7(), 'topology:write', 'Refrescar topología', NOW()),
    (uuid_generate_v7(), 'agents:read', 'Ver agentes', NOW()),
    (uuid_generate_v7(), 'agents:write', 'Configurar agentes', NOW()),
    (uuid_generate_v7(), 'metrics:read', 'Ver métricas', NOW())
ON CONFLICT (name) DO NOTHING;
```

```sql
-- data/seeds/system/003_role_permissions.sql

-- Admin: todos los permisos
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin' AND r.deleted_at IS NULL AND p.deleted_at IS NULL
ON CONFLICT DO NOTHING;

-- Operator: lectura + alertas + intrusiones + agentes + topología
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'operator'
  AND r.deleted_at IS NULL AND p.deleted_at IS NULL
  AND p.name IN (
    'devices:read', 'sedes:read', 'alerts:read', 'alerts:write',
    'intrusions:read', 'intrusions:write',
    'topology:read', 'topology:write', 'agents:read', 'agents:write',
    'metrics:read'
  )
ON CONFLICT DO NOTHING;

-- Viewer: solo lectura
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'viewer'
  AND r.deleted_at IS NULL AND p.deleted_at IS NULL
  AND p.name LIKE '%:read'
ON CONFLICT DO NOTHING;

-- Agent: solo lectura de dispositivos (para reportar métricas)
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'agent'
  AND r.deleted_at IS NULL AND p.deleted_at IS NULL
  AND p.name IN ('devices:read', 'metrics:read')
ON CONFLICT DO NOTHING;
```

---

## Principios adoptados

| Principio | Descripción |
|-----------|-------------|
| RBAC explícito | Roles con permisos granulares (no booleanos simples) |
| Soft Delete | `deleted_at` en todas las entidades (nunca DELETE físico) |
| Auditoría obligatoria | Toda mutación genera `audit_log` |
| Tokens hasheados | SHA-256 en DB, raw solo en memoria/email |
| PASETO v4 | Tokens de sesión (no JWT) |
| Email como identidad | `email` único, case-insensitive |
| Verificación obligatoria | `is_active = FALSE` hasta verificar email |
| Expiración de sesiones | Refresh tokens con TTL (7 días default) |
| Cache de permisos | Moka TTL 5min para evitar consultas repetidas |

---

## Flujo de autenticación

```txt
Registro
   ↓
Email + Password → Crear user (is_active = FALSE)
   ↓
Generar token de verificación (SHA-256 hash en DB)
   ↓
Enviar email con link de verificación
   ↓
Usuario clickea link → Verificar email → is_active = TRUE
   ↓
Login → Generar PASETO v4 (15min) + Refresh token (7 días)
   ↓
Sesión activa → RBAC verifica permisos en cada request
   ↓
Logout → Invalidar refresh token en DB
```

---

## Middleware de RBAC

```rust
// crates/infrastructure/src/middleware/rbac.rs

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use domain::ports::UserRepository;
use moka::future::Cache;

pub async fn rbac_middleware(
    required_permission: &str,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extraer AuthClaims del request (inyectado por auth_middleware)
    let claims = request.extensions()
        .get::<AuthClaims>()
        .ok_or(AppError::Unauthorized)?;

    // Verificar cache de permisos (Moka, TTL 5min)
    let cache_key = format!("perms:{}", claims.user_id);
    let has_permission = PERMISSION_CACHE
        .try_get_with(cache_key, async {
            // Consultar DB si no en cache
            user_repo.has_permission(claims.user_id, required_permission).await
        })
        .await
        .map_err(|_| AppError::Internal)?;

    if !has_permission {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(request).await)
}
```

---

## Auditoría automática

```rust
// crates/infrastructure/src/middleware/audit.rs

pub async fn audit_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    // Extraer datos del request
    let user_id = request.extensions().get::<AuthClaims>().map(|c| c.user_id);
    let action = determine_action(&request);  // "device_created", "alert_acknowledged", etc.
    let resource = determine_resource(&request);  // "devices", "alerts", etc.

    // Fire-and-forget a audit log (no bloquear response)
    tokio::spawn(async move {
        audit_repo.log(AuditLogEntry {
            user_id,
            action,
            resource,
            resource_id: extract_resource_id(&request),
            details: json!({ "duration_ms": duration.as_millis() }),
            ip_address: extract_ip(&request),
            user_agent: extract_ua(&request),
        }).await;
    });

    response
}
```

---

## Herramientas aprobadas

| Herramienta | Propósito | Versión |
|-------------|-----------|---------|
| `pasetors` | PASETO v4 tokens | `0.7` |
| `argon2` | Password hashing (argon2id) | `0.5` |
| `secrecy` | Protección de secretos en memoria | `0.10` |
| `moka` | Cache de permisos RBAC | `0.12` |
| `sqlx` | Queries compile-time checked | `0.8.5` |
| `cargo-nextest` | Tests rápidos | `0.9` |

**Nota:** No se usa `tower-sessions` (sesiones manejadas manualmente con PASETO + PostgreSQL). No se usa `zxcvbn` (validación de password con reglas simples: 12 chars, mayúscula, minúscula, número, símbolo).

---

## Alternativas descartadas

| Opción | Motivo |
|--------|--------|
| JWT | Tokens sin estado, difícil revocación, vulnerabilidades conocidas |
| OAuth2 / SSO | Overkill para MVP institucional (evaluar en futuro) |
| LDAP / Active Directory | No hay infraestructura de directorio en la Gobernación |
| Cookies sin firma | Vulnerables a tampering |
| RBAC simplificado (solo roles) | Sin granularidad suficiente (necesitamos permisos por recurso) |
| ABAC (Attribute-Based) | Complejidad innecesaria para el dominio actual |

---

## Consecuencias

### ✅ Positivas

* Control de acceso granular (permisos por recurso y acción)
* Trazabilidad completa de toda mutación
* Cumplimiento institucional (auditoría permanente)
* Tokens revocables (refresh tokens en DB)
* Seguridad de memoria (secrecy para secretos)
* Cache de permisos para performance (Moka)
* Soft delete para recuperación de datos
* Verificación de email obligatoria (reduce spam/fake accounts)

---

### ⚠️ Trade-offs

* Mayor complejidad que auth simple (user/pass sin roles)
* Cache de permisos requiere invalidación manual al cambiar roles
* Auditoría genera carga de escritura en DB (fire-and-forget mitiga)
* Tokens PASETO requieren rotación (implementado en refresh)
* Verificación de email requiere servicio de email (Resend)

---

## Impacto regional

El sistema de RBAC permite:

* delegar operación a personal local (operadores de sede),
* restringir acceso a datos sensibles (solo admins ven audit logs),
* auditar quién hizo qué y cuándo (crítico para instituciones públicas),
* y cumplir con regulaciones de transparencia.

La auditoría obligatoria genera:

* trazabilidad de cambios en infraestructura,
* evidencia para investigaciones de seguridad,
* reportes de actividad administrativa,
* y cumplimiento de normativas institucionales.

---

## Resultado esperado

Un sistema de seguridad:

* granular (RBAC con permisos),
* trazable (auditoría completa),
* seguro (PASETO v4, argon2id, tokens hasheados),
* eficiente (cache de permisos),
* recuperable (soft delete),
* verificable (email verification),
* y preparado para cumplimiento institucional.
