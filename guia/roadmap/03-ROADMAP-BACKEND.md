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
| II | API — Axum + Middleware + Errores | [ ] |
| III | Seguridad — Auth + RBAC + Audit | [ ] |
| IV | OpenAPI + Scalar | [ ] |
| V | Async — Jobs + Cache + Email | [ ] |
| VI | Observabilidad | [ ] |
| VII | Monitoreo — Inventario + Métricas + Topología | [ ] |
| **Backend Core** | | [x] 10% |

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

### I.2 — Migraciones del sistema (ADR 0006, ADR 0020)

> **Referencia:** ADR 0006, ADR 0005, ADR 0020

```
[x] Entidades de dominio creadas:
    - User, Role, Session, AuditLog, Token (auth)
    - Alert, IntrusionEvent (monitoreo)
    - Device, MetricReading, DeviceLink (inventario)
    - Sede (estructura regional)

[x] Puertos/Repositorios (traits síncronos):
    - UserRepository, SessionRepository, AuditRepository, TokenRepository
    - DeviceRepository, SedeRepository, MetricsRepository
    - AlertRepository, IntrusionRepository

[ ] Migraciones SQLx:
    [ ] data/migrations/20260305135148_create_users_table.sql
    [ ] data/migrations/20260305135149_create_rbac.sql
    [ ] data/migrations/20260305135150_create_tokens.sql
    [ ] data/migrations/20260305135151_create_audit_logs.sql
    [ ] data/migrations/20260305135152_seed_system_data.sql
    [ ] data/migrations/20260305135153_create_sessions.sql
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
1. [ ] Crear migraciones SQLx para PostgreSQL
2. [ ] Implementar repositories concretos en database/
3. [ ] Crear handlers HTTP en infrastructure/
4. [ ] Middleware de auth (PASETO)
```

---

## Diagrama de Dependencias Actualizado

```
┌─────────────────────────────────────────────────────────────┐
│  crates/domain                                              │
│  (thiserror, uuid, time, serde)                          │
│  └─ 11 entidades + 9 puertos                              │
│  └─ ADR 0001, 0006, 0008, 0020                          │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  crates/application                                         │
│  (domain + uuid)                                           │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  crates/database (impl pending)                            │
│  (domain + sqlx)                                           │
└─────────────────────────────────────────────────────────────┘
```

---

## Commit

```
feat(backend): B.1 entidades de dominio + puertos/repositorios

- 11 entidades: User, Role, Session, AuditLog, Token, Alert, IntrusionEvent, Device, MetricReading, DeviceLink, Sede
- 9 puertos traits síncronos: UserRepository, SessionRepository, AuditRepository, TokenRepository, DeviceRepository, SedeRepository, MetricsRepository, AlertRepository, IntrusionRepository
- Domain limpio: sin sqlx ni axum
- Soft Delete preparado en todas las entidades
```
