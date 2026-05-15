# Roadmap — Backend (Monitoreo de Infraestructura Regional)

> **Stack:** Rust 2024 · Axum 0.8 · SQLx 0.8 · SQLite WAL · PASETO v4 · Apalis · Utoipa
>
> **Proyecto:** Monitoreo de Infraestructura Regional - Gobernación del Beni
> **ADRs clave:** 0035 (Monitoreo Regional) · 0001 (Arquitectura) · 0003 (Axum) · 0004 (SQLite) · 
> 0006 (RBAC) · 0007 (Errores) · 0008 (Auth) · 0009 (Rate Limit) · 0010 (Testing) · 0024 (Local-First)

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| I | Fundación — Dominio + DB + RBAC | [ ] |
| II | API — Axum + Middleware + Errores | [ ] |
| III | Seguridad — Auth + RBAC + Audit | [ ] |
| IV | OpenAPI + Scalar | [ ] |
| V | Async — Jobs + Cache + Email | [ ] |
| VI | Observabilidad | [ ] |
| VII | Monitoreo — Inventario + Métricas + Topología | [ ] |
| **Backend Core** | | [ ] |

---

## Bloque I — Fundación (Dominio + DB + RBAC) 🔥

> **NO pasar al Bloque II sin el dominio compilando limpio.**
> **ADR 0035, 0001, 0004, 0006**

### I.1 — Pool SQLite con PRAGMAs

> **Referencia:** ADR 0004, ADR 0035

```
[ ] crates/database/src/pool.rs — create_pool():
    [ ] PRAGMA journal_mode = WAL
    [ ] PRAGMA synchronous  = NORMAL
    [ ] PRAGMA temp_store   = MEMORY
    [ ] PRAGMA mmap_size    = 30000000000
    [ ] PRAGMA foreign_keys = ON
    [ ] PRAGMA cache_size   = -64000
    [ ] max_connections = 10, min_connections = 2
    [ ] acquire_timeout = 5s
    [ ] log_slow_statements → LevelFilter::Warn (100ms)
[ ] Verificar que el pool conecta al arrancar
```

### I.2 — Migraciones del sistema (ADR 0006, ADR 0035)

> **Referencia:** ADR 0006, ADR 0005, ADR 0035

```
[ ] data/migrations/20260305135148_create_users_table.sql
    [ ] tabla users con Soft Delete (deleted_at)
    [ ] UNIQUE INDEX parcial en email WHERE deleted_at IS NULL
    [ ] trigger trg_users_updated_at
    [ ] tabla user_roles (N:M)

[ ] data/migrations/20260305135149_create_rbac.sql
    [ ] tabla roles
    [ ] tabla permissions (formato "recurso:acción")
    [ ] tabla role_permissions (N:M)

[ ] data/migrations/20260305135150_create_tokens.sql
    [ ] tabla tokens (verificación email + reset)

[ ] data/migrations/20260305135151_create_audit_logs.sql
    [ ] tabla audit_logs
    [ ] FOREIGN KEY user_id ON DELETE SET NULL
    [ ] INDEX idx_audit_resource_search

[ ] data/migrations/20260305135152_seed_system_data.sql
    [ ] INSERT OR IGNORE usuarios admin
    [ ] INSERT OR IGNORE roles Admin + User
    [ ] INSERT OR IGNORE permissions

[ ] data/migrations/20260305135153_create_sessions.sql
    [ ] tabla sessions (IP + UA + expiry)

[ ] Migraciones específicas de Monitoreo (ADR 0035):
    [ ] data/migrations/create_sedes.sql
        [ ] tabla sedes (nombre, ubicación, secretaría)
    [ ] data/migrations/create_devices.sql
        [ ] tabla devices (hostname, IP, MAC, tipo, estado, sede_id)
        [ ] tipos: switch, access_point, router, firewall, server, ups, camera, wireless_link
    [ ] data/migrations/create_device_links.sql
        [ ] tabla device_links (conexiones entre dispositivos)
    [ ] data/migrations/create_metric_readings.sql
        [ ] tabla metric_readings (bandwidth_rx, bandwidth_tx, latency, packet_loss)
    [ ] data/migrations/create_alerts.sql
        [ ] tabla alerts (tipo, severidad, device_id, mensaje)
    [ ] data/migrations/create_intrusion_events.sql
        [ ] tabla intrusion_events (mac, ip, detected_at, status)
```

### I.3 — Dominio puro — crates/domain/ (ADR 0001, ADR 0035)

> **Regla:** crates/domain/Cargo.toml solo tiene thiserror, uuid, time, serde.
> Si sqlx o axum aparecen aquí → la arquitectura está rota.

```
[ ] Cargo.toml: thiserror, uuid, time, serde, async-trait — NADA MÁS

[ ] entities/user.rs
    [ ] struct User { id, email, password_hash, name, is_active, ... }

[ ] entities/role.rs + entities/session.rs + entities/audit_log.rs

[ ] ENTIDADES DE MONITOREO (ADR 0035):
    [ ] entities/sede.rs
        [ ] struct Sede { id, nombre, ubicacion, secretaria, created_at }
    [ ] entities/device.rs
        [ ] enum DeviceType { Switch, AccessPoint, Router, Firewall, Server, Ups, Camera, WirelessLink }
        [ ] enum DeviceStatus { Active, Offline, Maintenance }
        [ ] struct Device { id, hostname, ip, mac, device_type, status, sede_id, last_seen_at }
    [ ] entities/metric.rs
        [ ] struct MetricReading { id, device_id, bandwidth_rx, bandwidth_tx, latency_ms, packet_loss, anomaly, created_at }
    [ ] entities/alert.rs
        [ ] enum AlertType { DeviceOffline, BandwidthSaturation, PacketLoss, Intrusion, TopologyChange, HighTraffic }
        [ ] enum AlertSeverity { Critical, High, Medium, Low }
        [ ] struct Alert { id, alert_type, severity, device_id, message, created_at, acknowledged_at }
    [ ] entities/intrusion.rs
        [ ] enum IntrusionStatus { Detected, Investigating, Resolved, FalsePositive }
        [ ] struct IntrusionEvent { id, mac, ip, device_id, status, detected_at }

[ ] value_objects/user_id.rs, email.rs, password_hash.rs, permission.rs

[ ] ports/user_repository.rs, session_repository.rs, audit_repository.rs, token_repository.rs

[ ] PUERTOS DE MONITOREO (ADR 0035):
    [ ] ports/device_repository.rs
        [ ] find_by_id, find_by_sede, find_by_type, save, update_status, list
    [ ] ports/sede_repository.rs
        [ ] find_all, find_by_id, save
    [ ] ports/metrics_repository.rs
        [ ] save, find_by_device, find_by_date_range, get_recent
    [ ] ports/alert_repository.rs
        [ ] save, find_active, acknowledge, find_by_device
    [ ] ports/intrusion_repository.rs
        [ ] save, find_pending, update_status

[ ] errors.rs
    [ ] DomainError variants para todos los casos
```

### I.4 — Casos de uso — crates/application/ (ADR 0001)

```
[ ] Cargo.toml: domain + thiserror + anyhow + tokio — sin sqlx, sin axum

[ ] use_cases/auth/register.rs, login.rs, logout.rs, refresh.rs

[ ] use_cases/users/get_user.rs, list_users.rs, update_user.rs, soft_delete_user.rs

[ ] USE CASES DE MONITOREO (ADR 0035):
    [ ] use_cases/monitoring/register_device.rs
    [ ] use_cases/monitoring/update_device_status.rs
    [ ] use_cases/monitoring/list_devices.rs
    [ ] use_cases/monitoring/get_device_metrics.rs
    [ ] use_cases/monitoring/create_alert.rs
    [ ] use_cases/monitoring/acknowledge_alert.rs
    [ ] use_cases/monitoring/detect_intrusion.rs
    [ ] use_cases/monitoring/get_topology.rs
    [ ] use_cases/monitoring/aggregate_metrics.rs
```

### I.5 — Repositorios SQLx — crates/database/ (ADR 0004)

```
[ ] Cargo.toml: domain + sqlx + moka + uuid + time + tracing

[ ] models/user_row.rs, audit_row.rs

[ ] MODELOS DE MONITOREO (ADR 0035):
    [ ] models/sede_row.rs
    [ ] models/device_row.rs
    [ ] models/metric_row.rs
    [ ] models/alert_row.rs
    [ ] models/intrusion_row.rs

[ ] repositories/sqlite_user_repository.rs, cached_user_repository.rs

[ ] REPOSITORIOS DE MONITOREO (ADR 0035):
    [ ] repositories/sqlite_sede_repository.rs
    [ ] repositories/sqlite_device_repository.rs
    [ ] repositories/sqlite_metrics_repository.rs
    [ ] repositories/sqlite_alert_repository.rs
    [ ] repositories/sqlite_intrusion_repository.rs
```

**Verificación Bloque I:**
```bash
just migrate
cargo nextest run -p domain
cargo nextest run -p application
cargo nextest run -p database
```

---

## Bloque II — API (Axum + Middleware + Errores) — ADR 0003, 0007

### II.1 — Setup Axum

```
[ ] apps/api/src/main.rs
    [ ] load config (fail-fast)
    [ ] init telemetry
    [ ] create_pool()
    [ ] migrate automático al arrancar
    [ ] build_state()
    [ ] serve con graceful shutdown

[ ] apps/api/src/setup.rs — build_state() composition root
    [ ] user_repo, session_repo, audit_repo, lead_repo
    [ ] paseto, mailer
    [ ] sede_repo, device_repo, metrics_repo, alert_repo, intrusion_repo

[ ] apps/api/src/router.rs — router modular
    [ ] GET  /health
    [ ] POST /auth/register, /auth/login, /auth/refresh, /auth/logout
    [ ] Rutas de MONITOREO (ADR 0035):
        [ ] GET/POST /api/v1/sedes
        [ ] GET/POST /api/v1/devices
        [ ] GET/PUT /api/v1/devices/:id
        [ ] GET /api/v1/devices/:id/metrics
        [ ] GET /api/v1/alerts
        [ ] POST /api/v1/alerts/:id/acknowledge
        [ ] GET /api/v1/topology/:sede_id
        [ ] GET /api/v1/intrusions
        [ ] POST /api/v1/intrusions/:id/resolve
    [ ] GET /docs, GET /openapi.json
```

### II.2 — Middleware en orden (ADR 0003, 0009)

```
[ ] 1. request_id_middleware   → x-request-id
[ ] 2. trace_middleware        → logging
[ ] 3. CompressionLayer        → gzip + brotli
[ ] 4. CorsLayer               → configurable
[ ] 5. TimeoutLayer            → 30 segundos
[ ] 6. Rate limiting (opcional)
```

### II.3 — Manejo de errores (ADR 0007)

```
[ ] DomainError en crates/domain/src/errors.rs
[ ] ApiError + IntoResponse en apps/api/src/error.rs
    [ ] 400 → InvalidEmail, InvalidPassword, Validation
    [ ] 401 → InvalidToken, InvalidCredentials
    [ ] 403 → Forbidden, MissingPermission
    [ ] 404 → NotFound
    [ ] 409 → EmailAlreadyExists
    [ ] 422 → Validation
    [ ] 500 → Database, Internal (sin detalles)
```

---

## Bloque III — Seguridad (Auth + RBAC + Audit) — ADR 0008, 0006

### III.1 — crates/auth/ — argon2id + PASETO v4

```
[ ] Cargo.toml: domain + argon2 + pasetors + secrecy — SIN jsonwebtoken

[ ] password.rs
    [ ] hash_password(password) → argon2id, parámetros OWASP 2024
    [ ] verify_password(password, hash)

[ ] paseto.rs — PasetoService
    [ ] generate_access_token(user_id, 15min) → "v4.local.xxx"
    [ ] verify(token) → TokenClaims
    [ ] Rechaza tokens JWT ("eyJ")

[ ] token.rs
    [ ] generate_opaque_token() → 32 bytes aleatorios
    [ ] hash_token(raw) → SHA-256
```

### III.2 — Endpoints de autenticación

```
[ ] POST /auth/register
[ ] POST /auth/login
[ ] POST /auth/refresh
[ ] POST /auth/logout
```

### III.3 — Middleware de Auth + RBAC

```
[ ] apps/api/src/middleware/auth.rs
    [ ] extrae Bearer token
    [ ] rechaza JWT (tokens que empiecen con "eyJ")
    [ ] verifica PASETO v4
    [ ] inyecta AuthClaims en request.extensions

[ ] apps/api/src/middleware/rbac.rs
    [ ] rbac_middleware — verifica permiso
    [ ] helpers: require_*

[ ] apps/api/src/middleware/audit.rs
    [ ] fire-and-forget a audit.log()
```

---

## Bloque IV — OpenAPI + Scalar (ADR 0021)

```
[ ] #[derive(ToSchema)] en todos los DTOs de request/response
[ ] #[utoipa::path] en cada handler
[ ] apps/api/src/docs.rs — ApiDoc central
    [ ] paths para auth, users, sedes, devices, metrics, alerts, topology, intrusions
    [ ] components: todos los DTOs de monitoreo
    [ ] Security: bearer_format = "PASETO"
    [ ] tags: auth, users, monitoring, network
[ ] /docs → Scalar UI
[ ] /openapi.json → disponible
```

---

## Bloque V — Async (Jobs + Cache + Email) — ADR 0018, 0019

### V.1 — Cache Moka Decorator

```
[ ] crates/database/src/repositories/cached_*.rs
    [ ] TTL 5min, max_capacity 10_000
    [ ] cache.invalidate() en save() — CRÍTICO
```

### V.2 — Email con Resend + LogMailer (ADR 0016)

```
[ ] crates/mailer/src/resend_adapter.rs
    [ ] LogMailer: imprime en tracing::info
    [ ] ResendMailer: implementación
    [ ] build_mailer(): selecciona según ENVIRONMENT
```

### V.3 — Jobs con Apalis (ADR 0018, ADR 0035)

```
[ ] apps/api/src/jobs/metrics_aggregation_job.rs
    [ ] Agregación de métricas por hora/día
[ ] apps/api/src/jobs/alert_dispatch_job.rs
    [ ] Envío de alertas por email/Telegram
[ ] apps/api/src/jobs/intrusion_detection_job.rs
    [ ] Análisis de anomalías
[ ] apps/api/src/jobs/cleanup_metrics_job.rs
    [ ] Limpieza de métricas antiguas (30 días)
[ ] apps/api/src/jobs/worker.rs
    [ ] SqliteStorage para jobs
    [ ] RetryLayer::new(RetryPolicy::retries(3))
```

---

## Bloque VI — Observabilidad (ADR 0015, ADR 0035)

```
[ ] apps/api/src/setup.rs — init_telemetry()
    [ ] tracing JSON subscriber con EnvFilter
    [ ] request_id en cada span

[ ] Sentry SDK
    [ ] sentry::init() con SENTRY_DSN (opcional)

[ ] Healthchecks.io (ADR 0015, ADR 0035)
    [ ] HC_WORKER_UUID para worker de métricas
    [ ] HC_LITESTREAM_UUID para backups
```

---

## Bloque VII — Monitoreo (ADR 0035)

### VII.1 — Inventario de Dispositivos

```
[ ] GET/POST /api/v1/sedes
    [ ] CRUD de sedes institucionales

[ ] GET/POST/PUT/DELETE /api/v1/devices
    [ ] CRUD de dispositivos
    [ ] tipos: switch, access_point, router, firewall, server, ups, camera, wireless_link
    [ ] estados: active, offline, maintenance

[ ] GET /api/v1/devices/:id/metrics
    [ ] Métricas históricas del dispositivo
```

### VII.2 — Métricas y Monitoreo

```
[ ] GET /api/v1/metrics
    [ ] Filtrado por device_id, date_range
    [ ] Agregación automática

[ ] apps/api/src/jobs/metrics_aggregation_job.rs
    [ ] Agregación por hora
    [ ] Agregación por día

[ ] apps/api/src/jobs/intrusion_detection_job.rs
    [ ] Comparar con whitelist
    [ ] Detectar MACs duplicadas
```

### VII.3 — Topología de Red

```
[ ] GET /api/v1/topology/:sede_id
    [ ] Mapa de conexiones de una sede
    [ ] device_links: source → target
    [ ] Estados visuales: online (verde), warning (amarillo), offline (rojo), maintenance (azul)

[ ] apps/api/src/handlers/topology_handler.rs
    [ ] Construir grafo de red
    [ ] Calcular dependencias jerárquicas
```

### VII.4 — Alertas

```
[ ] GET /api/v1/alerts
    [ ] Filtrado por severidad, device_id, fecha
    [ ] estados: active, acknowledged, resolved

[ ] POST /api/v1/alerts/:id/acknowledge
    [ ] Marcar alerta como vista

[ ] Tipos de alertas:
    [ ] DeviceOffline — dispositivo dejó de responder
    [ ] BandwidthSaturation — más de 80% de capacidad
    [ ] PacketLoss — más de 5% de pérdida
    [ ] Intrusion — dispositivo no autorizado detectado
    [ ] TopologyChange — cambio en la red
    [ ] HighTraffic — consumo anómalo
```

### VII.5 — Detección de Intrusos

```
[ ] GET /api/v1/intrusions
    [ ] Dispositivos detectados no autorizados

[ ] POST /api/v1/intrusions/:id/resolve
    [ ] Marcar como resuelto o falso positivo

[ ] apps/api/src/jobs/intrusion_detection_job.rs
    [ ] Estrategias: ARP scan, SNMP discovery, ICMP, DHCP logs
    [ ] Comparar con whitelist institucional
```

---

## ADRs de referencia por bloque

| Bloque | ADR |
|--------|-----|
| I — Fundación | 0035, 0001, 0004, 0005, 0006 |
| II — API | 0003, 0007, 0009 |
| III — Seguridad | 0008, 0006 |
| IV — OpenAPI | 0021 |
| V — Async | 0018, 0019, 0017 |
| VI — Observabilidad | 0015, 0016 |
| VII — Monitoreo | 0035, 0024 |

---

## Diagrama de Flujo de Bloques

```
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE I — Fundación                                                  │
│  ├─ Pool SQLite (WAL + PRAGMAs)                                       │
│  ├─ Migraciones (users, roles, devices, metrics, alerts, intrusions)  │
│  ├─ crates/domain (entities, value_objects, ports, errors)           │
│  │   └─ Entidades de Monitoreo: Sede, Device, Metric, Alert          │
│  ├─ crates/application (use_cases)                                    │
│  └─ crates/database (repositories SQLx + Moka cache)                   │
│     └─ Ref: ADR 0035, 0001, 0004, 0006                               │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE II — API + Middleware                                          │
│  ├─ Axum setup (main.rs, router, state composition)                  │
│  ├─ Rutas de Monitoreo: sedes, devices, metrics, alerts, topology     │
│  ├─ Middleware ordenado (trace, cors, rate limit, timeout)           │
│  └─ Error handling (DomainError → AppError → IntoResponse)           │
│     └─ Ref: ADR 0003, 0007, 0009, 0035                               │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE III — Seguridad                                                │
│  ├─ crates/auth (argon2id + PASETO v4) — NO JWT                      │
│  ├─ Endpoints /auth/* (register, login, refresh, logout)            │
│  ├─ Middleware auth (verify PASETO → UserId)                          │
│  ├─ Middleware RBAC (has_permission con cache Moka)                   │
│  └─ Middleware audit (logs)                                          │
│     └─ Ref: ADR 0008, 0006                                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE IV — OpenAPI + Scalar                                          │
│  ├─ utoipa ToSchema en todos los DTOs                                │
│  ├─ #[utoipa::path] en cada handler                                  │
│  ├─ /docs → Scalar UI                                                │
│  └─ /openapi.json → spec completa                                     │
│     └─ Ref: ADR 0021                                                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE V — Async (Jobs + Cache + Email)                               │
│  ├─ Moka cache decorator (TTL 5min, invalidación en writes)         │
│  ├─ Apalis jobs (metrics, alerts, intrusion, cleanup)                │
│  └─ Mailer dual (LogMailer en dev, Resend en prod)                    │
│     └─ Ref: ADR 0018, 0019, 0035                                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE VI — Observabilidad                                            │
│  ├─ Tracing JSON subscriber + request_id                             │
│  ├─ Sentry SDK (panics + errors)                                     │
│  ├─ Healthchecks.io (worker, litestream, deploy)                     │
│  └─ Logs estructurados para auditoría                                │
│     └─ Ref: ADR 0015, 0016, 0035                                     │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE VII — Monitoreo (ADR 0035)                                     │
│  ├─ Inventario: CRUD de sedes y dispositivos                         │
│  ├─ Métricas: recolección, agregación, histórico                     │
│  ├─ Topología: mapa de red por sede                                  │
│  ├─ Alertas: detección, notificación, Ack                           │
│  └─ Intrusiones: detección, resolución                                │
│     └─ Ref: ADR 0035, 0024 (Local-First)                             │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta/Crate | URL | Útil para |
|-------------------|-----|-----------|
| **Axum** | https://docs.rs/axum/latest | Router, handlers, middleware |
| **SQLx** | https://docs.rs/sqlx/latest | Queries compile-time checked |
| **PASETO** | https://paseto.io | Tokens v4.local (no JWT) |
| **utoipa** | https://docs.rs/utoipa/latest | OpenAPI generation |
| **Apalis** | https://docs.rs/apalis/latest | Background jobs |
| **snmp** | https://crates.io/crates/snmp | Recolección SNMP |
| **LayerChart** | https://layerchart.com | Gráficos realtime |
| **SvelteKit** | https://kit.svelte.dev | Frontend SSR |
| **Litestream** | https://litestream.io | Backups SQLite |

---

## Troubleshooting — Backend por Bloque

### Bloque I — Fundación

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `cargo check -p domain` falla | sqlx importado en domain | Verificar Cargo.toml no tenga sqlx |
| `just migrate` error | DATABASE_URL no seteada | Exportar DATABASE_URL |
| Entidades de monitoreo no compilan | Falta algún campo | Revisar ADR 0035 |

### Bloque II — API

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `cargo run --bin api` panic | Config inválida o DB no accesible | Revisar .env.local |
| `/health` retorna 500 | Pool no conecta | Verificar create_pool() |

### Bloque VII — Monitoreo

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Métricas no se guardan | Job no encolado | Verificar enqueue en use case |
| Topología vacía | device_links no creados | Crear enlaces entre dispositivos |
| Alertas no se envían | Resend no configurado | Verificar RESEND_API_KEY |

---

**Nota:** Este roadmap está basado en el ADR 0035 (Módulo de Monitoreo de Infraestructura Regional) para la Gobernación del Beni.