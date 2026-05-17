# Roadmap — Backend (Monitoreo de Infraestructura Regional)

> **Última revisión de versiones:** 2026-05-17
> Se actualizaron las versiones de dependencias tras auditoría contra crates.io, GitHub, docs.rs y repositorios oficiales.
>
> **Stack:** Rust 1.95 · Axum 0.8 · SQLx 0.9.0-alpha.1 · PostgreSQL 17.10 · PASETO v4 · Utoipa 5.5
>
> **Proyecto:** Monitoreo de Infraestructura Regional - Gobernación del Beni
> **ADRs clave:** 0020 (Monitoreo Regional) · 0001 (Arquitectura) · 0003 (Axum) · 0004 (PostgreSQL) ·
> 0006 (RBAC) · 0007 (Errores) · 0008 (Auth) · 0009 (Rate Limit) · 0010 (Testing)

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| I | Fundación — Dominio + DB + RBAC | [x] B.1 Entities + Ports |
| | | [x] B.2 Migraciones SQLx |
| II | API — Axum + Middleware + Errores | [ ] |
| III | Seguridad — Auth + RBAC + Audit | [ ] |
| IV | OpenAPI + Scalar | [ ] |
| V | Async — Jobs + Cache + Email | [ ] |
| VI | Observabilidad | [ ] |
| VII | Monitoreo — Inventario + Métricas + Topología | [ ] |
| **Backend Core** | | [x] 20% |

---

## Bloque I — Fundación (Dominio + DB + RBAC) 🔥

> **NO pasar al Bloque II sin el dominio compilando limpio.**
> **ADR 0020, 0001, 0004, 0006**

### I.1 — Pool PostgreSQL

> **Referencia:** ADR 0004, ADR 0020

```
[x] crates/database/src/pool.rs — create_pool():
    [x] Conexión PostgreSQL vía sqlx::postgres::PgPoolOptions
    [x] max_connections = 20
    [x] min_connections = 5
    [x] acquire_timeout = 5s
    [x] idle_timeout = 300s
    [x] max_lifetime = 1800s
    [x] application_name = "redes-api"
[x] Verificar que el pool conecta al arrancar (fail-fast)
```

### I.2 — Migraciones SQLx (Completado ✅ 2026-05-17)

> **Referencia:** ADR 0006, ADR 0005, ADR 0020

```
[x] data/migrations/001_create_users.sql
    [x] users con Soft Delete (deleted_at)
    [x] user_roles N:M
    [x] trigger updated_at

[x] data/migrations/002_create_rbac.sql
    [x] roles
    [x] permissions (15 predefinidas)
    [x] role_permissions N:M
    [x] seed: admin, operator, viewer

[x] data/migrations/003_create_sessions.sql
    [x] sessions con expiry

[x] data/migrations/004_create_tokens.sql
    [x] tokens para email/password reset
    [x] ENUM token_purpose

[x] data/migrations/005_create_audit_logs.sql
    [x] audit_logs con JSONB details
    [x] índices para búsqueda

[x] data/migrations/006_seed_system_data.sql
    [x] usuario admin inicial

[x] data/migrations/010_create_sedes.sql
    [x] sedes del Beni (8 predefinidas)

[x] data/migrations/011_create_devices.sql
    [x] devices con ENUMs device_type, device_status

[x] data/migrations/012_create_device_links.sql
    [x] device_links con CHECK constraint

[x] data/migrations/013_create_metrics.sql
    [x] metric_readings para monitoreo

[x] data/migrations/014_create_alerts.sql
    [x] alerts con ENUMs alert_type, severity, status

[x] data/migrations/015_create_intrusions.sql
    [x] intrusion_events con ENUM intrusion_status
```

### I.3 — Entidades de Dominio (Completado ✅)

> **Completado:** 2026-05-17

**Entidades creadas en `crates/domain/src/entities/`:**

| Entidad | Descripción | ADRs |
|---------|-------------|------|
| User | Usuario del sistema con Soft Delete | 0001, 0006 |
| Role | Roles RBAC | 0001, 0006 |
| Session | Sesiones de usuario | 0001, 0008 |
| AuditLog | Auditoría de acciones | 0001, 0006 |
| Token | Tokens email/password reset | 0001, 0008 |
| Alert | Alertas de monitoreo | 0001, 0020 |
| IntrusionEvent | Eventos de intrusión | 0001, 0020 |
| Device | Dispositivos de red | 0001, 0020 |
| MetricReading | Métricas de dispositivos | 0001, 0020 |
| DeviceLink | Conexiones entre dispositivos | 0001, 0020 |
| Sede | Sede regional | 0001, 0020 |

**Puertos creados en `crates/domain/src/ports/`:**

| Puerto | Descripción |
|--------|-------------|
| UserRepository | CRUD de usuarios |
| SessionRepository | Gestión de sesiones |
| AuditRepository | Logging de auditoría |
| TokenRepository | Tokens de auth |
| DeviceRepository | CRUD de dispositivos |
| SedeRepository | CRUD de sedes |
| MetricsRepository | Métricas |
| AlertRepository | Alertas |
| IntrusionRepository | Intrusiones |

### I.4 — Próximos Pasos

```
1. [x] Crear migraciones SQLx para PostgreSQL ✅
2. [ ] Implementar repositories concretos en database/
3. [ ] Crear handlers HTTP en infrastructure/
4. [ ] Middleware de auth (PASETO)
```

---

## Commits Realizados

| Commit | Descripción |
|--------|-------------|
| `6e7fd3e` | feat(genesis): workspace completo con 12 crates y 3 apps |
| `7731871` | feat(backend): B.1 entidades de dominio + puertos/repositorios |
| `0895b26` | feat(backend): B.2 migraciones SQLx |

---

## Siguiente Fase

**Bloque II:** API — Axum + Middleware + Errores

```
B.3 Handlers HTTP (CRUD sedes, dispositivos, métricas)
B.4 OpenAPI con Utoipa + Scalar
B.5 Middleware auth (PASETO)
```
