# ADR 0005 — Migraciones SQLx y Seeding Idempotente

> **Última revisión de versiones:** 2026-05-16  
> Se actualizaron las versiones de herramientas tras auditoría contra crates.io, GitHub y repositorios oficiales.

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0002 (Configuración), ADR 0004 (PostgreSQL), ADR 0006 (RBAC), ADR 0012 (Herramientas) |

---

## Contexto

A medida que el sistema de monitoreo crece,
modificar la base de datos manualmente
introduce riesgos operacionales y pérdida de consistencia.

Necesitamos una estrategia que permita:

* evolucionar el esquema de forma segura,
* mantener sincronizados los entornos,
* automatizar despliegues,
* poblar datos de desarrollo,
* poblar datos mínimos de sistema en producción (roles, permisos, admin),
* y garantizar reproducibilidad.

El sistema debe poder reconstruirse completamente
desde cero mediante comandos automatizados.

---

## Decisión

Usar:

* SQLx Migrations (versionadas, reversibles)
* migraciones con `up.sql` + `down.sql`
* seeds idempotentes (sin race conditions)
* automatización mediante `just` + CLI
* separación clara entre dev (automático) y prod (manual)

---

## Estructura aprobada

```txt
data/
├── migrations/
│   ├── 20260101000001_create_users_table/
│   │   ├── up.sql
│   │   └── down.sql
│   ├── 20260101000002_create_rbac_tables/
│   │   ├── up.sql
│   │   └── down.sql
│   ├── 20260101000003_create_devices_table/
│   │   ├── up.sql
│   │   └── down.sql
│   └── 20260101000004_create_metrics_table/
│       ├── up.sql
│       └── down.sql
│
├── seeds/
│   ├── system/              # Seeds de sistema (roles, permisos, admin)
│   │   ├── 001_roles.sql
│   │   ├── 002_permissions.sql
│   │   └── 003_admin_user.sql
│   └── development/         # Seeds de demo (solo dev)
│       ├── 001_demo_sedes.sql
│       ├── 002_demo_devices.sql
│       └── 003_demo_metrics.sql
│
└── .sqlx/                   # Queries compile-time checked (en repo)
    └── query-*.json
```

---

## Reglas de migraciones

| Regla | Descripción |
|-------|-------------|
| R1 | Cada migración tiene `up.sql` (aplicar) y `down.sql` (revertir) |
| R2 | Las migraciones son inmutables después de ejecutarse en cualquier entorno |
| R3 | Nunca modificar una migración ya ejecutada — crear migración correctiva nueva |
| R4 | `down.sql` solo para desarrollo y staging, **nunca en producción** |
| R5 | Migraciones de esquema separadas de seeds de datos |
| R6 | `sqlx prepare` ejecutado en CI antes de cada build |
| R7 | `sqlx migrate info` verificado en CI (todas las migraciones aplicadas) |
| R8 | Migraciones aplicadas **automáticamente** en dev/test, **manualmente** en prod |

---

## Migraciones SQLx

### Crear migración reversible

```bash
# Crear migración con down.sql automático
sqlx migrate add --reversible create_devices_table

# Esto genera:
# data/migrations/2026XXXXXX_create_devices_table/
#   ├── up.sql
#   └── down.sql
```

### Ejecutar migraciones

```bash
# Desarrollo (automático via justfile)
just migrate

# Producción (manual, con backup previo)
just migrate-prod

# Verificar estado
sqlx migrate info
```

---

## Ejemplo de migración completa

```sql
-- data/migrations/2026XXXXXX_create_devices_table/up.sql

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE device_type AS ENUM (
    'switch', 'access_point', 'router', 'firewall',
    'server', 'ups', 'camera', 'wireless_link'
);

CREATE TYPE device_status AS ENUM (
    'active', 'offline', 'maintenance'
);

CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    sede_id UUID NOT NULL REFERENCES sedes(id) ON DELETE CASCADE,
    hostname TEXT NOT NULL,
    ip_address INET NOT NULL,
    mac_address MACADDR NOT NULL,
    device_type device_type NOT NULL,
    status device_status NOT NULL DEFAULT 'active',
    last_seen_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Índice parcial para MAC única entre activos
CREATE UNIQUE INDEX IF NOT EXISTS idx_devices_mac_active
ON devices(mac_address)
WHERE deleted_at IS NULL;

-- Índice para búsqueda por sede + estado
CREATE INDEX IF NOT EXISTS idx_devices_sede_status
ON devices(sede_id, status)
WHERE deleted_at IS NULL;

-- Índice para búsqueda por IP
CREATE INDEX IF NOT EXISTS idx_devices_ip
ON devices(ip_address);

-- Trigger para updated_at automático
CREATE OR REPLACE FUNCTION trg_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_devices_updated_at
    BEFORE UPDATE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION trg_updated_at();
```

```sql
-- data/migrations/2026XXXXXX_create_devices_table/down.sql

DROP TRIGGER IF EXISTS trg_devices_updated_at ON devices;
DROP FUNCTION IF EXISTS trg_updated_at();

DROP INDEX IF EXISTS idx_devices_ip;
DROP INDEX IF EXISTS idx_devices_sede_status;
DROP INDEX IF EXISTS idx_devices_mac_active;

DROP TABLE IF EXISTS devices;

DROP TYPE IF EXISTS device_status;
DROP TYPE IF EXISTS device_type;
```

---

## Seeds de sistema (ejecutables en producción)

> **Regla:** Los seeds de sistema (roles, permisos, admin) se ejecutan **una sola vez** durante el setup inicial de producción. Son idempotentes.

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
    (uuid_generate_v7(), 'devices:read', 'Ver dispositivos', NOW()),
    (uuid_generate_v7(), 'devices:write', 'Crear/editar dispositivos', NOW()),
    (uuid_generate_v7(), 'devices:delete', 'Archivar dispositivos', NOW()),
    (uuid_generate_v7(), 'alerts:read', 'Ver alertas', NOW()),
    (uuid_generate_v7(), 'alerts:write', 'Acknowledge/resolve alertas', NOW()),
    (uuid_generate_v7(), 'intrusions:read', 'Ver intrusiones', NOW()),
    (uuid_generate_v7(), 'intrusions:write', 'Resolver intrusiones', NOW()),
    (uuid_generate_v7(), 'audit:read', 'Ver audit logs', NOW()),
    (uuid_generate_v7(), 'audit:export', 'Exportar audit logs', NOW()),
    (uuid_generate_v7(), 'topology:read', 'Ver topología', NOW()),
    (uuid_generate_v7(), 'topology:write', 'Refrescar topología', NOW()),
    (uuid_generate_v7(), 'agents:read', 'Ver agentes', NOW()),
    (uuid_generate_v7(), 'agents:write', 'Configurar agentes', NOW())
ON CONFLICT (name) DO NOTHING;
```

```sql
-- data/seeds/system/003_role_permissions.sql

-- Admin: todos los permisos
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin'
ON CONFLICT DO NOTHING;

-- Operator: lectura + alertas + intrusiones + agentes
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'operator'
  AND p.name IN (
    'devices:read', 'alerts:read', 'alerts:write',
    'intrusions:read', 'intrusions:write',
    'topology:read', 'agents:read', 'agents:write'
  )
ON CONFLICT DO NOTHING;

-- Viewer: solo lectura
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'viewer'
  AND p.name LIKE '%:read'
ON CONFLICT DO NOTHING;

-- Agent: solo métricas y lectura de dispositivos
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'agent'
  AND p.name IN ('devices:read')
ON CONFLICT DO NOTHING;
```

```sql
-- data/seeds/system/004_admin_user.sql

-- Crear usuario admin (password debe cambiarse en primer login)
INSERT INTO users (
    id, email, password_hash, name, is_active, email_verified, created_at, updated_at
)
VALUES (
    uuid_generate_v7(),
    'admin@redes.local',
    -- Hash de 'Cambiar123!' con argon2id (debe regenerarse en producción)
    '$argon2id$v=19$m=19456,t=2,p=1$...',
    'Administrador',
    true,
    true,
    NOW(),
    NOW()
)
ON CONFLICT (email) DO NOTHING;

-- Asignar rol admin
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
WHERE u.email = 'admin@redes.local'
  AND r.name = 'admin'
ON CONFLICT DO NOTHING;
```

---

## Seeds de desarrollo (solo dev/test)

> **Regla:** Los seeds de desarrollo **nunca** se ejecutan en producción. Son para demos, testing y desarrollo local.

```sql
-- data/seeds/development/001_demo_sedes.sql

INSERT INTO sedes (id, nombre, ubicacion, secretaria, network_cidr, created_at, updated_at)
VALUES
    (uuid_generate_v7(), 'Sede Central', 'Trinidad, Beni', 'Gobernación', '192.168.1.0/24', NOW(), NOW()),
    (uuid_generate_v7(), 'Sede Riberalta', 'Riberalta, Beni', 'Secretaría de Obras', '192.168.2.0/24', NOW(), NOW()),
    (uuid_generate_v7(), 'Sede Guayaramerín', 'Guayaramerín, Beni', 'Secretaría de Salud', '192.168.3.0/24', NOW(), NOW())
ON CONFLICT DO NOTHING;
```

---

## Automatización con justfile

> **Nota:** `just` 1.51.0 (mayo 2026) incluye mejoras en manejo de módulos, atributos `[no-cd]` y funciones de path. No hay breaking changes desde 1.40.

```makefile
# ── Migraciones ───────────────────────────────────────────

# Desarrollo: migraciones automáticas + seeds de sistema + seeds de dev
migrate:
    sqlx migrate run
    cargo run --bin cli seed-system
    cargo run --bin cli seed-development

# Producción: migraciones manuales (con confirmación)
[migrate-prod]
[confirm("¿Backup realizado? Esto es PRODUCCIÓN.")]
migrate-prod:
    sqlx migrate run
    cargo run --bin cli seed-system

# Verificar estado de migraciones
migrate-status:
    sqlx migrate info

# Revertir última migración (solo dev)
migrate-revert:
    @echo "⚠️  Revirtiendo última migración..."
    sqlx migrate revert

# ── Seeds ─────────────────────────────────────────────────

# Seeds de sistema (roles, permisos, admin) — idempotentes
seed-system:
    cargo run --bin cli seed-system

# Seeds de desarrollo (demo data) — solo dev
seed-development:
    cargo run --bin cli seed-development

# ── Reset de base de datos ────────────────────────────────

# Reset SOLO de la base de datos (sin destruir volúmenes de otros servicios)
db-reset:
    @echo "⚠️  Esto destruirá la base de datos redes"
    @read -p "¿Continuar? (yes/no): " confirm && [ $$confirm = "yes" ] || exit 1
    docker compose stop postgres
    docker compose rm -f postgres
    docker volume rm redes_postgres_data || true
    docker compose up -d postgres
    sleep 5
    sqlx migrate run
    just seed-system
    just seed-development

# Reset COMPLETO de infraestructura (todos los volúmenes)
infra-reset:
    @echo "⚠️  Esto destruirá TODOS los datos"
    @read -p "¿Continuar? (yes/no): " confirm && [ $$confirm = "yes" ] || exit 1
    docker compose down -v
    docker compose up -d
    sleep 10
    sqlx migrate run
    just seed-system
    just seed-development

# ── SQLx prepare (CI) ─────────────────────────────────────

# Preparar queries para compilación sin DB (CI)
prepare:
    cargo sqlx prepare --workspace -- --all-targets --all-features

# Verificar que prepare está actualizado
check-prepare:
    cargo sqlx prepare --workspace --check -- --all-targets --all-features
```

---

## Workflow de CI para migraciones

```yaml
# .github/workflows/migrations.yml
name: Verify Migrations

on:
  push:
    paths:
      - 'data/migrations/**'
      - 'crates/database/**'

jobs:
  verify:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:17.10-alpine
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Install sqlx-cli
        run: cargo install sqlx-cli --features postgres --locked

      - name: Verify migrations apply cleanly
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/redes
        run: |
          sqlx database create
          sqlx migrate run
          sqlx migrate info

      - name: Verify sqlx prepare is up to date
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/redes
        run: |
          cargo sqlx prepare --workspace --check
```

---

## Principios adoptados

| Principio | Descripción |
|-----------|-------------|
| Migraciones inmutables | Nunca modificar migraciones ejecutadas en cualquier entorno |
| Reversibilidad | Cada migración tiene `down.sql` (para dev/staging) |
| Reproducibilidad | La DB puede reconstruirse automáticamente desde cero |
| Fail-fast | El sistema falla si la migración falla (en dev) |
| Idempotencia | Seeds ejecutables múltiples veces sin duplicados |
| Separación de entornos | Dev (automático) vs Prod (manual con backup) |
| SQL explícito | Máximo control y visibilidad |
| Compile-time checking | `sqlx prepare` valida queries en CI |

---

## Corrección de migraciones en producción

Si una migración en producción contiene errores:

```txt
❌ Nunca modificar migraciones ejecutadas

✅ Crear una nueva migración correctiva:
   sqlx migrate add fix_devices_column

✅ Escribir up.sql con ALTER TABLE / UPDATE / etc.

✅ Escribir down.sql con reversión de cambios

✅ Probar en staging antes de prod
```

Esto preserva:

* trazabilidad,
* auditoría,
* consistencia histórica,
* capacidad de rollback.

---

## Simplificaciones aplicadas

Para mantener simplicidad operacional
NO se incluyen:

* generación automática de schemas (no hay ORM)
* ORMs complejos (Diesel, Prisma)
* migraciones dinámicas (ejecutadas en runtime por la app)
* abstracciones enterprise (Flyway, Liquibase)
* sincronización automática multi-entorno (cada entorno controla sus migraciones)
* auto-migraciones en producción (siempre manual con backup)

El sistema utiliza:

* SQL explícito (máximo control)
* SQLx migrations (versionadas, reversibles)
* PostgreSQL (motor robusto)
* just (automatización)
* sqlx prepare (compile-time checking)

---

## Herramientas aprobadas

| Herramienta | Propósito | Versión |
|-------------|-----------|---------|
| `sqlx-cli` | CLI de migraciones y prepare | `0.8.6` |
| `just` | Automatización de comandos | `1.51.0` |
| `cargo-nextest` | Tests rápidos y paralelos | `0.9.135` |
| `pg_dump` | Backups full | PostgreSQL 17 |
| `docker compose` | Orquestación local | `v5.1+` |

---

## Alternativas descartadas

| Opción | Motivo |
|--------|--------|
| Prisma ORM | Abstracción excesiva, menos control sobre queries |
| Diesel | Mayor fricción async, compile-time más lento |
| Flyway | Overkill para un proyecto Rust (SQLx cubre el caso) |
| Liquibase | XML/JSON complejos, preferimos SQL explícito |
| Migraciones manuales | Riesgo operacional, sin versionado |
| Auto-sync de schema | Riesgo de inconsistencias entre entornos |

---

## Consecuencias

### ✅ Positivas

* Esquema versionado con trazabilidad completa
* Deploy reproducible en cualquier entorno
* Evolución controlada del esquema
* Integración fuerte con Rust (SQLx compile-time checking)
* Fácil reconstrucción del entorno (`just db-reset`)
* Seeds idempotentes (sin race conditions)
* Separación clara entre datos de sistema y datos de demo
* Mayor seguridad operativa (migraciones manuales en prod)

---

### ⚠️ Trade-offs

* Requiere conocer SQL (no hay ORM que genere automáticamente)
* Las migraciones incorrectas requieren corrección adicional (nueva migración)
* Mayor disciplina en cambios de esquema (no modificar migraciones pasadas)
* `sqlx prepare` debe ejecutarse en CI (paso adicional)
* Seeds de sistema requieren password inicial seguro (debe cambiarse en prod)

---

## Impacto regional

La automatización reduce:

* errores humanos en cambios de esquema,
* configuraciones inconsistentes entre oficinas,
* problemas en reinstalaciones remotas,
* tiempo de recuperación ante desastres,
* y soporte remoto innecesario.

Esto es importante para:

* oficinas regionales con conectividad limitada,
* infraestructura distribuida en múltiples sedes,
* y mantenimiento simplificado por personal local.

---

## Resultado esperado

Un sistema de persistencia:

* reproducible (`just db-reset` reconstruye todo),
* seguro (migraciones manuales en prod con backup),
* mantenible (SQL explícito, versionado),
* auditable (historial completo de cambios),
* fácil de desplegar (Docker + SQLx),
* preparado para evolucionar sin pérdida de consistencia,
* y con datos mínimos de sistema listos para producción (roles, permisos, admin).

---

## Registro de cambios de versiones

| Fecha | Componente | Anterior | Actual | Notas |
|-------|------------|----------|--------|-------|
| 2026-05-16 | sqlx-cli | 0.8.5 | **0.8.6** | Patch release con fixes |
| 2026-05-16 | just | 1.40 | **1.51.0** | Nuevas funciones: módulos, `[no-cd]`, path functions. Sin breaking changes. |
| 2026-05-16 | cargo-nextest | 0.9 | **0.9.135** | Actualización de runner de tests |
| 2026-05-16 | PostgreSQL (CI) | 16.4 | **17.10** | Alineado con ADR 0004 |
| 2026-05-16 | docker compose | 2.25+ | **v5.1+** | Alineado con ADR 0013 |
