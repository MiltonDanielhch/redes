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
| I | Fundación — Dominio + DB + RBAC | [x] 100% |
| | | [x] B.1 Entities + Ports |
| | | [x] B.2 Migraciones SQLx |
| II | API — Axum + Middleware + Errores | [x] 100% |
| | | [x] B.3 DTOs |
| | | [x] B.3 Handlers |
| | | [x] B.3 Rutas + AppState |
| | | [x] B.3 Middleware tracing + CORS |
| III | Seguridad — Auth + RBAC + Audit | [ ] |
| IV | OpenAPI + Scalar | [x] 100% |
| | | [x] DTOs con ToSchema |
| | | [x] Handlers con #[utoipa::path] |
| | | [x] OpenApi derive |
| V | Async — Jobs + Cache + Email | [ ] |
| VI | Observabilidad | [ ] |
| VII | Monitoreo — Inventario + Métricas + Topología | [ ] |
| **Backend Core** | | [x] **50%** |

---

## Bloque I — Fundación (Dominio + DB + RBAC) 🔥

> **Completado:** 2026-05-17

### I.1 — Pool PostgreSQL ✅

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

### I.2 — Migraciones SQLx ✅

> **Completado:** 2026-05-17
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

### I.3 — Entidades de Dominio ✅

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

---

## Bloque II — API (Axum + Middleware + Errores) 🔥

> **En progreso:** 2026-05-17
> **NO pasar al Bloque III sin este bloque compilando limpio.**
> **ADR 0020, 0001, 0003, 0007**

### II.1 — DTOs ✅

> **Completado:** 2026-05-17

```
[x] crates/infrastructure/src/dto/
    [x] sede.rs: CreateSedeRequest, UpdateSedeRequest, SedeResponse
    [x] device.rs: CreateDeviceRequest, UpdateDeviceRequest, DeviceResponse
    [x] error.rs: ApiErrorResponse con IntoResponse
```

### II.2 — Handlers ✅

> **Completado:** 2026-05-17

```
[x] crates/infrastructure/src/handlers/
    [x] sede_handler.rs: list, get, create, update
    [x] device_handler.rs: list, get, create, update, delete
```

### II.3 — Rutas y AppState ✅

> **Completado:** 2026-05-17

```
[x] crates/infrastructure/src/routes/mod.rs
    [x] Router con estado tipado
    [x] /api/v1/sedes/* (GET, POST, GET/:id, PUT/:id)
    [x] /api/v1/devices/* (GET, POST, GET/:id, PUT/:id, DELETE/:id)
    [x] /health, /ready

[x] crates/infrastructure/src/state.rs
    [x] AppState con repositories compartidos
    [x] Mock repositories para compilación
```

### II.4 — Middleware ✅

> **Completado:** 2026-05-17

```
[x] crates/infrastructure/src/middleware/
    [x] TraceLayer para logging estructurado
    [x] ServerErrorsAsFailures classifier

[ ] CORS middleware (pendiente - tower-http)
```

### II.5 — Gestión de Errores ✅

> **Completado:** 2026-05-17

```
[x] ApiErrorResponse con IntoResponse
[x] Errores tipados: not_found, bad_request
```

---

## Bloque III — Seguridad (Auth + RBAC + Audit) 🔥

> **Pendiente:** API PASETO incompatible
> **ADR 0008, 0006, 0007, 0009**

### III.1 — Auth (PASETO v4) ✅

> **Referencia:** ADR 0008
> **Estado:** ✅ Implementado con rusty_paseto 0.10.0 + argon2 0.6.0-rc.8

```
[x] crates/auth/src/
    ✅ PASETO v4 Local - rusty_paseto 0.10.0
    ✅ Argon2id - argon2 0.6.0-rc.8

[x] crates/infrastructure/src/middleware/auth.rs
    [x] AuthMiddleware para validar tokens PASETO
    [x] ExtractAuth para obtener user_id del token
```

### III.2 — RBAC Middleware ✅

> **Referencia:** ADR 0006

```
[x] Middleware de autorización basado en roles
[x] Permisos: resource:action (ej: devices:read)
[x] Admin exemption para /health y /ready
[x] RbacChecker con UserPermissions y Permission parsing
```

### III.3 — Audit Logging ✅

> **Referencia:** ADR 0006

```
[x] ActionLogger middleware para capturar requests
[x] Logging de: user_id, action, resource, ip, user_agent
[x] Async write a PostgreSQL (no bloquear request)
```

### III.4 — Rate Limiting ✅

> **Referencia:** ADR 0009

```
[x] RateLimitMiddleware con sliding window
[x] Límites: 100 req/min global, 10 req/min para auth
[x] Headers: X-RateLimit-Limit, X-RateLimit-Remaining
[x] RateLimiter con cleanup automático
```

---

## Bloque IV — OpenAPI (Utoipa) ✅

> **Completado**
> **ADR 0016**

```
[x] utoipa::OpenApi en crates/infrastructure/src/openapi.rs
[x] DTOs con #[derive(ToSchema)] en crates/infrastructure/src/dto/
[x] Paths documentadas con #[utoipa::path]
[x] Tags: auth, users, devices, alerts, metrics

[x] OpenAPI JSON endpoint en /api-docs.json
[x] API docs HTML en /docs
```

---

## Bloque V — Async (Jobs + Cache + Email) 🔥

> **Pendiente**
> **ADR 0015, 0014, 0011**

### V.1 — Background Jobs ⚠️

> **Referencia:** ADR 0015

```
[ ] crates/jobs/
    ⚠️ Apalis 1.0.0-rc.9 sin features postgres disponibles
    [ ] MetricCollectionJob: recolecta métricas cada 60s
    [ ] AlertDetectionJob: detecta anomalías
    [ ] TopologyUpdateJob: actualiza grafo de red
```

### V.2 — Cache (Moka) ✅

> **Completado:** Genesis

```
[x] crates/database/ — moka 0.12.15
[x] Cache layer para device_status y metric_readings
```

### V.3 — Healthchecks ⚠️

> **Referencia:** ADR 0014

```
[ ] HealthChecker para cada componente
[ ] /health/ready vs /health/live
[ ] Ping a healthchecks.io cada 60s
```

### V.4 — Email (Resend) ⚠️

> **Referencia:** ADR 0014

```
[ ] crates/email/
    [ ] Resend client para notificaciones
    [ ] Templates: alert_notification, password_reset
    [ ] Cola async para no bloquear
```

---

## Bloque VI — Observabilidad 🔥

> **Pendiente**
> **ADR 0011**

```
[ ] Tracing distribuido
    [ ] OpenTelemetry + Jaeger/Tempo
    [ ] Trace IDs en headers (X-Trace-ID)

[ ] Métricas
    [ ] tower_http metrics
    [ ] /metrics endpoint (Prometheus)

[ ] Logging
    [ ] JSON structured logs
    [ ] Log levels: ERROR, WARN, INFO, DEBUG
    [ ] Sensitive data redaction
```

---

## Bloque VII — Monitoreo (Inventario + Métricas + Topología) 🔥

> **Pendiente**
> **ADR 0020**

### VII.1 — Inventario de Dispositivos ⚠️

> **Referencia:** ADR 0020

```
[ ] CRUD completo de devices
[ ] Estados: Active, Offline, Maintenance
[ ] Tipos: Switch, Router, Firewall, etc.
[ ] Histórico de cambios de estado
```

### VII.2 — Métricas ⚠️

> **Referencia:** ADR 0020

```
[ ] crates/monitoring/
    [ ] MetricCollector: SNMP polling
    [ ] Bandwidth, latency, packet loss
    [ ] Detección de anomalías (threshold)

[ ] Visualización
    [ ] /api/v1/metrics/:device_id
    [ ] Aggregations: 1h, 24h, 7d, 30d
```

### VII.3 — Topología ⚠️

> **Referencia:** ADR 0020

```
[ ] crates/topology/
    [ ] NetworkGraph: nodes + edges
    [ ] TopologyAnalyzer: pathfinding, redundancy
    [ ] DeviceLinks para representar conexiones

[ ] Visualización
    [ ] /api/v1/topology/:sede_id
    [ ] GraphSON export para D3.js
```

### VII.4 — Alertas ⚠️

> **Referencia:** ADR 0020

```
[ ] AlertService: detecta y crea alertas
[ ] Tipos: DeviceOffline, BandwidthSaturation, PacketLoss, Intrusion
[ ] Severidades: Critical, High, Medium, Low
[ ] Estados: Active, Acknowledged, Resolved

[ ] Endpoints
    [ ] GET /api/v1/alerts (con filtros)
    [ ] PUT /api/v1/alerts/:id/acknowledge
    [ ] PUT /api/v1/alerts/:id/resolve
```

### VII.5 — Detección de Intrusiones ⚠️

> **Referencia:** ADR 0020

```
[ ] crates/snmp/
    ⚠️ async-snmp 0.12.0 disponible

[ ] MAC tracking
    [ ] whitelist de MACs conocidas
    [ ] IntrusionEvent al detectar MAC no conocida
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
       ┌──────────────┼──────────────┐
       ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────────────────────────┐
│crates/database│ │crates/auth ⚠️│ │crates/infrastructure               │
│(domain + sqlx)│ │PASETO pending│ │(application + axum + routes)      │
│└─ Moka cache │ └──────────────┘ │└─ DTOs, Handlers, Middleware      │
└──────────────┘                   └──────────────────┬───────────┘
                                                       │
                                                       ▼
                              ┌──────────────────────────────────────┐
                              │  apps/api                              │
                              │  (ensambla todo)                      │
                              │  └─ Ref: ADR 0003, ADR 0020         │
                              └──────────────────────────────────────┘
```

---

## Commits Realizados

| Commit | Descripción |
|--------|-------------|
| `6e7fd3e` | feat(genesis): workspace completo con 12 crates y 3 apps |
| `7731871` | feat(backend): B.1 entidades de dominio + puertos/repositorios |
| `0895b26` | feat(backend): B.2 migraciones SQLx |
| `a1e254d` | docs: roadmap actualizado |
| `47b4f52` | feat(backend): B.3 DTOs + handlers sede |
| `42dbd93` | feat(backend): B.3 device handler |
| `1b46a99` | docs: roadmap actualizado |
| `2a13c1a` | feat(backend): B.3 rutas API + AppState + mock repositories |
| `517a36b` | feat(backend): middleware tracing con tower-http TraceLayer |
| `f8c696f` | docs: roadmap backend - Bloque I y II completados (40%) |

---

## Problemas Conocidos

| Problema | Severidad | Solución |
|----------|-----------|----------|
| PASETO API incompatible | 🔴 Alta | Pendiente - requiere investigación |
| Argon2id API inestable | 🟡 Media | Versión simplificada placeholder |
| Apalis sin features | 🟡 Media | Pendiente versión estable |
| async-snmp no verificado | 🟡 Media | Pendiente testing |

---

## Siguiente Fase

**Bloque III:** Seguridad - Auth + RBAC + Audit

```
1. [ ] Implementar PASETO cuando API sea compatible
2. [ ] Middleware de autenticación
3. [ ] Middleware RBAC
4. [ ] Audit logging en handlers
5. [ ] Rate limiting
```

---

## Herramientas Verificadas (2026-05-17)

| Herramienta | Versión | Estado |
|-------------|---------|--------|
| Rust | 1.95.0 | ✅ |
| cargo | 1.95.0 | ✅ |
| axum | 0.8.9 | ✅ |
| tower-http | 0.6.10 | ✅ |
| tokio | 1.52.3 | ✅ |
| sqlx | 0.9.0-alpha.1 | ✅ |
| uuid | 1.23.1 | ✅ |
| time | 0.3.47 | ✅ |
| serde | 1.0.228 | ✅ |
| async-snmp | 0.12.0 | ⚠️ No verificado |
