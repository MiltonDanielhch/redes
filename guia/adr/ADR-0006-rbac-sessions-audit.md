# ADR 0006 — Esquema Base: RBAC + Sessions + Auditoría

| Campo               | Valor                                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                       |
| **Fecha**           | 2026                                                                                             |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                 |
| **Relacionado con** | ADR 0004 (PostgreSQL + Docker), ADR 0005 (Migraciones SQLx), ADR 0007 (Errores), ADR 0008 (Auth) |

---

# Contexto

El sistema de monitoreo de infraestructura de red
requiere control de acceso,
auditoría,
y sesiones persistentes
desde el inicio del proyecto.

Agregar autenticación y trazabilidad
después de entrar en producción
genera deuda técnica difícil de corregir.

Se necesita desde el día uno:

* control granular de permisos,
* auditoría de acciones,
* trazabilidad de sesiones,
* y recuperación segura de acceso.

---

# Decisión

Implementar un esquema base compuesto por:

* usuarios,
* roles,
* permisos,
* sesiones,
* tokens de recuperación,
* y auditoría.

El sistema seguirá un modelo RBAC
(Role-Based Access Control).

---

# Objetivos del esquema

| Objetivo          | Resultado            |
| ----------------- | -------------------- |
| Control de acceso | Permisos granulares  |
| Seguridad         | Sesiones controladas |
| Auditoría         | Registro completo    |
| Escalabilidad     | Roles reutilizables  |
| Mantenimiento     | Separación clara     |

---

# Migración 1 — Usuarios

```sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,

    username TEXT,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,

    avatar_url TEXT,

    email_verified BOOLEAN NOT NULL DEFAULT FALSE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_active
ON users(email)
WHERE deleted_at IS NULL;
```

---

# Migración 2 — Roles y Permisos

```sql
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL,
    permission_id UUID NOT NULL,

    PRIMARY KEY(role_id, permission_id),

    FOREIGN KEY(role_id)
        REFERENCES roles(id)
        ON DELETE CASCADE,

    FOREIGN KEY(permission_id)
        REFERENCES permissions(id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL,
    role_id UUID NOT NULL,

    PRIMARY KEY(user_id, role_id),

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,

    FOREIGN KEY(role_id)
        REFERENCES roles(id)
        ON DELETE CASCADE
);
```

---

# Migración 3 — Tokens

```sql
CREATE TABLE IF NOT EXISTS tokens (
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL,

    token_hash TEXT NOT NULL UNIQUE,

    type TEXT NOT NULL,

    expires_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    used BOOLEAN DEFAULT FALSE,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);
```

---

# Migración 4 — Auditoría

```sql
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,

    user_id UUID,

    action TEXT NOT NULL,

    resource TEXT NOT NULL,

    resource_id TEXT,

    payload JSONB,

    ip_address TEXT,

    user_agent TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE SET NULL
);
```

---

# Migración 5 — Seed Inicial

```sql
INSERT INTO roles (id, name, description)
VALUES
(gen_random_uuid(), 'Admin', 'Acceso total'),
(gen_random_uuid(), 'Operator', 'Operador de monitoreo')
ON CONFLICT DO NOTHING;
```

El usuario administrador inicial
se crea únicamente
mediante variables seguras de entorno
o comando CLI.

No se incluyen passwords hardcodeadas
en migraciones.

---

# Migración 6 — Sesiones

```sql
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL,

    session_token TEXT NOT NULL UNIQUE,

    ip_address TEXT,

    user_agent TEXT,

    expires_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    last_activity_at TIMESTAMPTZ DEFAULT NOW(),

    is_revoked BOOLEAN DEFAULT FALSE,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);
```

---

# Relaciones del sistema

```txt
users
 ├── user_roles ─── roles ─── role_permissions ─── permissions
 ├── sessions
 ├── tokens
 └── audit_logs
```

---

# Verificación de permisos en Rust

```rust
async fn has_permission(
    &self,
    user_id: Uuid,
    permission: &str,
) -> Result<bool, DomainError> {

    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM users u
        JOIN user_roles ur
            ON ur.user_id = u.id
        JOIN role_permissions rp
            ON rp.role_id = ur.role_id
        JOIN permissions p
            ON p.id = rp.permission_id
        WHERE u.id = $1
          AND p.name = $2
          AND u.deleted_at IS NULL
        "#,
        user_id,
        permission
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(count.unwrap_or(0) > 0)
}
```

---

# Decisiones técnicas adoptadas

| Decisión              | Motivo                       |
| --------------------- | ---------------------------- |
| UUID v7               | Mejor rendimiento de índices |
| JSONB en auditoría    | Flexibilidad y búsqueda      |
| Soft delete           | Preservación histórica       |
| RBAC N:M              | Múltiples roles              |
| Sessions separadas    | Mejor control                |
| Auditoría persistente | Evidencia histórica          |

---

# Simplificaciones aplicadas

Para mantener el estándar Código 3026
NO se incluyen:

* ABAC complejo,
* motores externos de políticas,
* ACL dinámicas,
* multi-tenant,
* OAuth enterprise,
* permisos jerárquicos complejos,
* integración LDAP inicial.

El sistema usa:

* RBAC clásico,
* sesiones simples,
* permisos explícitos,
* y auditoría integrada.

---

# Herramientas aprobadas

| Herramienta      | Uso                     |
| ---------------- | ----------------------- |
| `sqlx`           | Persistencia            |
| `argon2id`       | Hash de passwords       |
| `tower-sessions` | Gestión de sesiones     |
| `uuid` v7        | IDs                     |
| `zxcvbn`         | Validación de passwords |
| `moka`           | Caché de permisos       |

---

# Alternativas descartadas

| Opción                 | Motivo                 |
| ---------------------- | ---------------------- |
| OAuth complejo inicial | Sobrecarga innecesaria |
| LDAP corporativo       | No requerido           |
| JWT stateless puro     | Revocación complicada  |
| ABAC completo          | Mayor complejidad      |

---

# Consecuencias

## ✅ Positivas

* Control granular desde el inicio
* Auditoría completa
* Escalabilidad limpia
* Permisos reutilizables
* Mejor seguridad operativa
* Fácil mantenimiento

---

## ⚠️ Trade-offs

* Más tablas y joins
* Mayor complejidad inicial
* Requiere índices adecuados

---

# Impacto regional

El sistema permite:

* trazabilidad administrativa,
* control de operadores,
* auditoría institucional,
* y monitoreo seguro
  en oficinas regionales.

La separación de permisos
reduce riesgos operativos
en infraestructura gubernamental.

---

# Resultado esperado

Un sistema de identidad:

* seguro,
* auditable,
* mantenible,
* extensible,
* y preparado para producción.
