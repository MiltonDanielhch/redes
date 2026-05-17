# Roadmap — Backend (Monitoreo de Infraestructura Regional)

> **Stack:** Rust 2024 · Axum 0.8 · SQLx 0.8 · PostgreSQL · PASETO v4 · Apalis · Utoipa
>
> **Proyecto:** Monitoreo de Infraestructura Regional - Gobernación del Beni
> **ADRs clave:** 0020 (Monitoreo Regional) · 0001 (Arquitectura) · 0003 (Axum) · 0004 (PostgreSQL) · 
> 0006 (RBAC) · 0007 (Errores) · 0008 (Auth) · 0009 (Rate Limit) · 0010 (Testing) · 0015 (Jobs)

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
> **ADR 0020, 0001, 0004, 0006**

### I.1 — Pool PostgreSQL

> **Referencia:** ADR 0004, ADR 0020

```
[ ] crates/database/src/pool.rs — create_pool():
    [ ] Conexión PostgreSQL vía sqlx::postgres::PgPoolOptions
    [ ] max_connections = 20 (ajustar según instancia)
    [ ] min_connections = 5
    [ ] acquire_timeout = 5s
    [ ] idle_timeout = 300s
    [ ] max_lifetime = 1800s
    [ ] log_slow_statements → LevelFilter::Warn (100ms)
    [ ] application_name = "redes-api"
    [ ] sslmode = prefer (configurable vía env)
[ ] Verificar que el pool conecta al arrancar (fail-fast)
```

**Nota:** PostgreSQL no usa PRAGMAs. Los parámetros de rendimiento (WAL, cache, etc.) se configuran en `postgresql.conf` del servidor, no en la conexión del cliente.

### I.2 — Migraciones del sistema (ADR 0006, ADR 0020)

> **Referencia:** ADR 0006, ADR 0005, ADR 0020

```
[ ] data/migrations/20260305135148_create_users_table.sql
    [ ] tabla users con Soft Delete (deleted_at TIMESTAMP WITH TIME ZONE)
    [ ] UNIQUE INDEX parcial en email WHERE deleted_at IS NULL
    [ ] trigger trg_users_updated_at (updated_at = NOW())
    [ ] tabla user_roles (N:M users ↔ roles)

[ ] data/migrations/20260305135149_create_rbac.sql
    [ ] tabla roles (id, name, description, created_at)
    [ ] tabla permissions (id, name formato "recurso:acción", description)
    [ ] tabla role_permissions (N:M roles ↔ permissions)

[ ] data/migrations/20260305135150_create_tokens.sql
    [ ] tabla tokens (id, user_id, token_hash, purpose, expires_at, created_at)
    [ ] purpose: email_verification, password_reset
    [ ] INDEX idx_tokens_user_purpose

[ ] data/migrations/20260305135151_create_audit_logs.sql
    [ ] tabla audit_logs (id, user_id, action, resource, resource_id, details, ip_address, user_agent, created_at)
    [ ] FOREIGN KEY user_id ON DELETE SET NULL
    [ ] INDEX idx_audit_resource_search (resource, created_at DESC)
    [ ] INDEX idx_audit_user (user_id, created_at DESC)

[ ] data/migrations/20260305135152_seed_system_data.sql
    [ ] INSERT INTO roles ... ON CONFLICT DO NOTHING
    [ ] INSERT INTO permissions ... ON CONFLICT DO NOTHING
    [ ] INSERT INTO users (admin) ... ON CONFLICT DO NOTHING
    [ ] INSERT INTO user_roles ... ON CONFLICT DO NOTHING

[ ] data/migrations/20260305135153_create_sessions.sql
    [ ] tabla sessions (id, user_id, token_hash, ip_address, user_agent, expires_at, created_at)
    [ ] FOREIGN KEY user_id ON DELETE CASCADE
    [ ] INDEX idx_sessions_user (user_id)
    [ ] INDEX idx_sessions_expiry (expires_at) para cleanup

[ ] Migraciones específicas de Monitoreo (ADR 0020):
    [ ] data/migrations/20260305135154_create_sedes.sql
        [ ] tabla sedes (id, nombre, ubicacion, secretaria, created_at, updated_at)
        [ ] INDEX idx_sedes_nombre (nombre)
    [ ] data/migrations/20260305135155_create_devices.sql
        [ ] tabla devices (id, hostname, ip_address, mac_address, device_type, status, sede_id, last_seen_at, deleted_at, created_at, updated_at)
        [ ] device_type: switch, access_point, router, firewall, server, ups, camera, wireless_link
        [ ] status: active, offline, maintenance
        [ ] FOREIGN KEY sede_id REFERENCES sedes(id)
        [ ] UNIQUE INDEX idx_devices_mac (mac_address) WHERE deleted_at IS NULL
        [ ] INDEX idx_devices_sede (sede_id, status)
    [ ] data/migrations/20260305135156_create_device_links.sql
        [ ] tabla device_links (id, source_device_id, target_device_id, link_type, bandwidth_mbps, status, created_at)
        [ ] link_type: ethernet, fiber, wireless, serial
        [ ] FOREIGN KEY source_device_id REFERENCES devices(id)
        [ ] FOREIGN KEY target_device_id REFERENCES devices(id)
        [ ] CHECK source_device_id != target_device_id
    [ ] data/migrations/20260305135157_create_metric_readings.sql
        [ ] tabla metric_readings (id, device_id, bandwidth_rx_bytes, bandwidth_tx_bytes, latency_ms, packet_loss_percent, anomaly_detected, created_at)
        [ ] FOREIGN KEY device_id REFERENCES devices(id)
        [ ] INDEX idx_metrics_device_time (device_id, created_at DESC)
        [ ] INDEX idx_metrics_time (created_at DESC)
        [ ] Particionamiento opcional por rango (created_at) para escalabilidad
    [ ] data/migrations/20260305135158_create_alerts.sql
        [ ] tabla alerts (id, alert_type, severity, device_id, message, details, status, acknowledged_by, acknowledged_at, created_at, updated_at)
        [ ] alert_type: device_offline, bandwidth_saturation, packet_loss, intrusion, topology_change, high_traffic
        [ ] severity: critical, high, medium, low
        [ ] status: active, acknowledged, resolved
        [ ] FOREIGN KEY device_id REFERENCES devices(id) ON DELETE SET NULL
        [ ] FOREIGN KEY acknowledged_by REFERENCES users(id) ON DELETE SET NULL
        [ ] INDEX idx_alerts_status_severity (status, severity, created_at DESC)
        [ ] INDEX idx_alerts_device (device_id, created_at DESC)
    [ ] data/migrations/20260305135159_create_intrusion_events.sql
        [ ] tabla intrusion_events (id, mac_address, ip_address, device_id, status, detected_at, resolved_at, notes, created_at, updated_at)
        [ ] status: detected, investigating, resolved, false_positive
        [ ] FOREIGN KEY device_id REFERENCES devices(id) ON DELETE SET NULL
        [ ] INDEX idx_intrusions_status (status, detected_at DESC)
```

### I.3 — Dominio puro — crates/domain/ (ADR 0001, ADR 0020)

> **Regla:** crates/domain/Cargo.toml solo tiene thiserror, uuid, time, serde.
> Si sqlx, axum, tokio o async-trait aparecen aquí → la arquitectura está rota.

```
[ ] Cargo.toml: thiserror, uuid, time, serde — NADA MÁS (sin async-trait, sin tokio)

[ ] entities/user.rs
    [ ] struct User { id, email, password_hash, name, is_active, created_at, updated_at, deleted_at }
    [ ] Validaciones: email formato, name no vacío

[ ] entities/role.rs + entities/session.rs + entities/audit_log.rs
    [ ] Role: id, name, description
    [ ] Session: id, user_id, token_hash, ip_address, user_agent, expires_at
    [ ] AuditLog: id, user_id, action, resource, resource_id, details, ip_address, user_agent, created_at

[ ] entities/token.rs
    [ ] struct Token { id, user_id, token_hash, purpose, expires_at, created_at }
    [ ] enum TokenPurpose { EmailVerification, PasswordReset }

[ ] ENTIDADES DE MONITOREO (ADR 0020):
    [ ] entities/sede.rs
        [ ] struct Sede { id: SedeId, nombre: String, ubicacion: String, secretaria: String, created_at: OffsetDateTime }
    [ ] entities/device.rs
        [ ] enum DeviceType { Switch, AccessPoint, Router, Firewall, Server, Ups, Camera, WirelessLink }
        [ ] enum DeviceStatus { Active, Offline, Maintenance }
        [ ] struct Device { id: DeviceId, hostname: String, ip_address: String, mac_address: String, device_type: DeviceType, status: DeviceStatus, sede_id: SedeId, last_seen_at: Option<OffsetDateTime>, created_at: OffsetDateTime, updated_at: OffsetDateTime, deleted_at: Option<OffsetDateTime> }
    [ ] entities/metric.rs
        [ ] struct MetricReading { id: MetricId, device_id: DeviceId, bandwidth_rx_bytes: i64, bandwidth_tx_bytes: i64, latency_ms: Option<i32>, packet_loss_percent: Option<f64>, anomaly_detected: bool, created_at: OffsetDateTime }
    [ ] entities/alert.rs
        [ ] enum AlertType { DeviceOffline, BandwidthSaturation, PacketLoss, Intrusion, TopologyChange, HighTraffic }
        [ ] enum AlertSeverity { Critical, High, Medium, Low }
        [ ] enum AlertStatus { Active, Acknowledged, Resolved }
        [ ] struct Alert { id: AlertId, alert_type: AlertType, severity: AlertSeverity, device_id: Option<DeviceId>, message: String, details: Option<String>, status: AlertStatus, acknowledged_by: Option<UserId>, acknowledged_at: Option<OffsetDateTime>, created_at: OffsetDateTime, updated_at: OffsetDateTime }
    [ ] entities/intrusion.rs
        [ ] enum IntrusionStatus { Detected, Investigating, Resolved, FalsePositive }
        [ ] struct IntrusionEvent { id: IntrusionId, mac_address: String, ip_address: Option<String>, device_id: Option<DeviceId>, status: IntrusionStatus, detected_at: OffsetDateTime, resolved_at: Option<OffsetDateTime>, notes: Option<String>, created_at: OffsetDateTime, updated_at: OffsetDateTime }

[ ] value_objects/
    [ ] user_id.rs — Newtype UUID v7 con prefix "usr_"
    [ ] sede_id.rs — Newtype UUID v7 con prefix "sede_"
    [ ] device_id.rs — Newtype UUID v7 con prefix "dev_"
    [ ] email.rs — Validación formato RFC 5322
    [ ] password_hash.rs — Wrapper opaco
    [ ] permission.rs — Formato "recurso:acción"

[ ] ports/ (traits — síncronos, sin async-trait)
    [ ] ports/user_repository.rs — find_by_id, find_by_email, save, soft_delete, list
    [ ] ports/session_repository.rs — create, find_by_token_hash, delete_by_user_id, delete_expired
    [ ] ports/audit_repository.rs — log, find_by_resource, find_by_user
    [ ] ports/token_repository.rs — create, find_by_hash, delete_by_user_id, delete_expired
    [ ] ports/device_repository.rs — find_by_id, find_by_sede, find_by_type, save, update_status, soft_delete, list
    [ ] ports/sede_repository.rs — find_all, find_by_id, save
    [ ] ports/metrics_repository.rs — save, find_by_device, find_by_date_range, get_recent
    [ ] ports/alert_repository.rs — save, find_active, acknowledge, find_by_device, find_by_status
    [ ] ports/intrusion_repository.rs — save, find_pending, update_status, find_by_status

[ ] errors.rs
    [ ] DomainError enum con variants:
        [ ] Validation(String)
        [ ] NotFound(String)
        [ ] AlreadyExists(String)
        [ ] Unauthorized
        [ ] Forbidden
        [ ] Internal(String)
```

### I.4 — Casos de uso — crates/application/ (ADR 0001)

```
[ ] Cargo.toml: domain + thiserror — sin sqlx, sin axum, sin tokio, sin anyhow

[ ] use_cases/auth/
    [ ] register.rs — validar email único, hashear password, crear user, emitir evento
    [ ] login.rs — verificar credenciales, crear sesión
    [ ] logout.rs — invalidar sesión
    [ ] refresh.rs — rotar token (si aplica en PASETO)

[ ] use_cases/users/
    [ ] get_user.rs
    [ ] list_users.rs — filtrar deleted_at IS NULL
    [ ] update_user.rs
    [ ] soft_delete_user.rs — UPDATE deleted_at = NOW(), nunca DELETE

[ ] USE CASES DE MONITOREO (ADR 0020):
    [ ] use_cases/monitoring/register_device.rs
    [ ] use_cases/monitoring/update_device_status.rs
    [ ] use_cases/monitoring/list_devices.rs — filtrar soft delete
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

[ ] models/
    [ ] models/user_row.rs — mapeo sqlx::FromRow ↔ User
    [ ] models/role_row.rs
    [ ] models/session_row.rs
    [ ] models/audit_row.rs
    [ ] models/token_row.rs
    [ ] models/sede_row.rs
    [ ] models/device_row.rs
    [ ] models/metric_row.rs
    [ ] models/alert_row.rs
    [ ] models/intrusion_row.rs

[ ] repositories/
    [ ] repositories/user_repository.rs — impl UserRepository para Postgres
    [ ] repositories/cached_user_repository.rs — decorador Moka
    [ ] repositories/role_repository.rs
    [ ] repositories/session_repository.rs
    [ ] repositories/audit_repository.rs
    [ ] repositories/token_repository.rs
    [ ] repositories/sede_repository.rs
    [ ] repositories/device_repository.rs — incluye soft delete logic
    [ ] repositories/metrics_repository.rs
    [ ] repositories/alert_repository.rs
    [ ] repositories/intrusion_repository.rs
```

**Verificación Bloque I:**
```bash
just migrate
cargo nextest run -p domain
cargo nextest run -p application
cargo nextest run -p database
grep -r "sqlx" crates/domain/ --include="*.toml" && echo "ERROR: sqlx en domain" || echo "OK"
grep -r "jsonwebtoken" . --include="*.toml" && echo "ERROR: JWT detectado" || echo "OK"
```

---

## Bloque II — API (Axum + Middleware + Errores) — ADR 0003, 0007

### II.1 — Setup Axum

```
[ ] apps/api/src/main.rs
    [ ] load config (fail-fast) — si falta variable, panic inmediato
    [ ] init telemetry
    [ ] create_pool() — conexión PostgreSQL
    [ ] NO migrar automáticamente en producción (usar just migrate manual)
    [ ] build_state() — composition root
    [ ] serve con graceful shutdown

[ ] apps/api/src/setup.rs — build_state() composition root
    [ ] user_repo, session_repo, audit_repo, token_repo
    [ ] sede_repo, device_repo, metrics_repo, alert_repo, intrusion_repo
    [ ] paseto_service, mailer_service
    [ ] state: Arc<AppState> compartido entre handlers

[ ] apps/api/src/router.rs — router modular
    [ ] GET  /health
    [ ] POST /auth/register, /auth/login, /auth/refresh, /auth/logout
    [ ] Rutas de MONITOREO (ADR 0020):
        [ ] GET/POST /api/v1/sedes
        [ ] GET/POST /api/v1/devices
        [ ] GET/PUT /api/v1/devices/:id
        [ ] PUT /api/v1/devices/:id/archive  ← Soft delete (no DELETE)
        [ ] GET /api/v1/devices/:id/metrics
        [ ] GET /api/v1/metrics
        [ ] GET /api/v1/alerts
        [ ] POST /api/v1/alerts/:id/acknowledge
        [ ] GET /api/v1/topology/:sede_id
        [ ] GET /api/v1/intrusions
        [ ] POST /api/v1/intrusions/:id/resolve
    [ ] GET /docs, GET /openapi.json
```

### II.2 — Middleware en orden (ADR 0003, 0009)

```
[ ] 1. trace_middleware        → logging con request_id
[ ] 2. request_id_middleware   → x-request-id (después de trace para que se propague)
[ ] 3. CorsLayer               → configurable por env
[ ] 4. RateLimitLayer          ← NO opcional (ADR 0009)
[ ] 5. TimeoutLayer            → 30 segundos
[ ] 6. CompressionLayer        → gzip + brotli (último, sobre el body final)
```

### II.3 — Manejo de errores (ADR 0007)

```
[ ] DomainError en crates/domain/src/errors.rs
[ ] AppError + IntoResponse en apps/api/src/error.rs
    [ ] 400 → Validation, InvalidEmail, InvalidPassword
    [ ] 401 → InvalidToken, InvalidCredentials, SessionExpired
    [ ] 403 → Forbidden, MissingPermission
    [ ] 404 → NotFound
    [ ] 409 → EmailAlreadyExists, MacAlreadyExists
    [ ] 422 → UnprocessableEntity
    [ ] 500 → Database, Internal (sin detalles al cliente)
[ ] Cada error 500 loggea full stacktrace via tracing::error!
```

---

## Bloque III — Seguridad (Auth + RBAC + Audit) — ADR 0008, 0006

### III.1 — crates/auth/ — argon2id + PASETO v4

```
[ ] Cargo.toml: domain + argon2 + pasetors + secrecy — SIN jsonwebtoken

[ ] password.rs
    [ ] hash_password(password) → argon2id, parámetros OWASP 2024
        [ ] memory = 19456, iterations = 2, parallelism = 1 (mínimo recomendado)
    [ ] verify_password(password, hash) → bool + timing-safe

[ ] paseto.rs — PasetoService
    [ ] generate_access_token(user_id, 15min) → "v4.local.xxx"
    [ ] verify(token) → TokenClaims { user_id, exp }
    [ ] Rechaza explícitamente tokens JWT (strings que empiecen con "eyJ")
    [ ] Secret key desde PASETO_SECRET (32+ bytes, fail-fast si falta)

[ ] token.rs
    [ ] generate_opaque_token() → 32 bytes aleatorios (rand::thread_rng)
    [ ] hash_token(raw) → SHA-256 para almacenamiento
```

### III.2 — Endpoints de autenticación

```
[ ] POST /auth/register
    [ ] Validar email único (case-insensitive)
    [ ] Hash password con argon2id
    [ ] Crear user + asignar rol "User"
    [ ] Log audit: user_registered

[ ] POST /auth/login
    [ ] Verificar credenciales
    [ ] Crear sesión (token hash en DB)
    [ ] Retornar PASETO v4.local token
    [ ] Log audit: user_login

[ ] POST /auth/refresh
    [ ] Verificar PASETO vigente
    [ ] Generar nuevo token (rotación opcional)

[ ] POST /auth/logout
    [ ] Invalidar sesión en DB
    [ ] Log audit: user_logout
```

### III.3 — Middleware de Auth + RBAC

```
[ ] apps/api/src/middleware/auth.rs
    [ ] extrae Bearer token del header Authorization
    [ ] rechaza JWT (tokens que empiecen con "eyJ") → 401
    [ ] verifica PASETO v4 via pasetors
    [ ] inyecta AuthClaims { user_id, roles } en request.extensions
    [ ] 401 si token inválido o expirado

[ ] apps/api/src/middleware/rbac.rs
    [ ] rbac_middleware(permission: &str) → verifica que user tenga permiso
    [ ] helpers: require_permission!("device:read")
    [ ] Cache de permisos en Moka (TTL 5min) para evitar consultas repetidas

[ ] apps/api/src/middleware/audit.rs
    [ ] fire-and-forget a audit.log()
    [ ] Captura: user_id, action, resource, ip, ua, timestamp
    [ ] No bloquear response si audit falla (loggear error localmente)
```

---

## Bloque IV — OpenAPI + Scalar (ADR 0016)

```
[ ] #[derive(ToSchema)] en todos los DTOs de request/response
    [ ] DTOs en crates/infrastructure/src/dtos/ (o apps/api/src/dtos/)
    [ ] Separar RequestDTO y ResponseDTO

[ ] #[utoipa::path] en cada handler
    [ ] tags organizados: Auth, Users, Sedes, Devices, Metrics, Alerts, Topology, Intrusions

[ ] apps/api/src/docs.rs — ApiDoc central
    [ ] paths: auth, users, sedes, devices, metrics, alerts, topology, intrusions
    [ ] components: todos los DTOs de monitoreo + auth
    [ ] SecurityScheme: bearer_format = "PASETO", scheme = "Bearer"
    [ ] tags con descripciones

[ ] /docs → Scalar UI (servido estáticamente o vía tower-http)
[ ] /openapi.json → spec completa generada en runtime
```

---

## Bloque V — Async (Jobs + Cache + Email) — ADR 0015, ADR 0016

### V.1 — Cache Moka Decorator

```
[ ] crates/database/src/repositories/cached_*.rs
    [ ] TTL 5min, max_capacity 10_000
    [ ] cache.invalidate() en save() / update() / soft_delete() — CRÍTICO
    [ ] Cache key: "{entity}:{id}" o "{entity}:list:{hash_of_filters}"
    [ ] Solo cachear lecturas frecuentes: user_by_id, device_by_id, sede_list
```

### V.2 — Email con Resend (ADR 0016)

```
[ ] crates/infrastructure/src/mailer/resend_adapter.rs
    [ ] MailerPort trait en domain
    [ ] LogMailer: imprime en tracing::info (dev/test)
    [ ] ResendMailer: implementación real con reqwest + Resend API
    [ ] build_mailer(): selecciona según ENVIRONMENT
    [ ] Templates: email_verification, password_reset, alert_notification
```

**Nota:** El mailer vive en `crates/infrastructure/` (servicio externo), no como crate independiente.

### V.3 — Jobs con Apalis (ADR 0015, ADR 0020)

```
[ ] crates/jobs/src/jobs/metrics_aggregation_job.rs
    [ ] Agregación de métricas por hora/día (rollup)
    [ ] Almacenar en tabla metric_aggregations (opcional)

[ ] crates/jobs/src/jobs/alert_dispatch_job.rs
    [ ] Envío de alertas críticas por email (Resend)
    [ ] Envío opcional por Telegram/Webhook

[ ] crates/jobs/src/jobs/intrusion_detection_job.rs
    [ ] Análisis de anomalías en métricas recientes
    [ ] Comparar con whitelist de MACs
    [ ] Crear alerta + intrusion_event si se detecta intruso

[ ] crates/jobs/src/jobs/cleanup_metrics_job.rs
    [ ] Limpieza de métricas brutas antiguas (> 30 días)
    [ ] Retener agregaciones históricas indefinidamente

[ ] crates/jobs/src/jobs/sync_offline_job.rs
    [ ] Procesar sync_queue de sedes offline (ADR 0021)
    [ ] Reconciliar métricas diferidas

[ ] crates/jobs/src/worker.rs
    [ ] PostgreSQL storage para Apalis (sqlx compatible)
    [ ] RetryLayer::new(RetryPolicy::retries(3))
    [ ] Concurrency control (max 5 workers concurrentes)
```

**Nota:** Los jobs viven en `crates/jobs/`, no en `apps/api/src/jobs/`. La app API solo los encola.

---

## Bloque VI — Observabilidad (ADR 0014, ADR 0015)

```
[ ] apps/api/src/setup.rs — init_telemetry()
    [ ] tracing subscriber JSON para producción
    [ ] EnvFilter configurable vía RUST_LOG
    [ ] request_id en cada span (propagado desde middleware)
    [ ] tracing-opentelemetry opcional

[ ] Sentry SDK
    [ ] sentry::init() con SENTRY_DSN (opcional, fail-soft)
    [ ] Captura panics y errores 500
    [ ] Breadcrumbs para requests

[ ] Healthchecks.io (ADR 0014)
    [ ] HC_API_KEY configurado
    [ ] Ping periódico desde worker de jobs
    [ ] Ping separado para backup verification

[ ] Métricas internas (opcional)
    [ ] Contador de requests por endpoint
    [ ] Histograma de latencia DB
    [ ] Gauge de conexiones al pool
```

**Nota:** PostgreSQL usa herramientas nativas de backup (pg_dump, WAL). No usar Litestream (es solo para SQLite).

---

## Bloque VII — Monitoreo (ADR 0020)

### VII.1 — Inventario de Dispositivos

```
[ ] GET/POST /api/v1/sedes
    [ ] CRUD de sedes institucionales
    [ ] Paginación + búsqueda por nombre

[ ] GET/POST/PUT /api/v1/devices
    [ ] CRUD de dispositivos
    [ ] tipos: switch, access_point, router, firewall, server, ups, camera, wireless_link
    [ ] estados: active, offline, maintenance
    [ ] Validar MAC única (formato IEEE 802)
    [ ] Validar IP válida (IPv4)

[ ] PUT /api/v1/devices/:id/archive  ← Soft delete
    [ ] UPDATE devices SET deleted_at = NOW() WHERE id = $1
    [ ] Invalidar cache
    [ ] Log audit: device_archived

[ ] GET /api/v1/devices/:id/metrics
    [ ] Métricas históricas del dispositivo (últimas 24h default)
    [ ] Paginación temporal
```

### VII.2 — Métricas y Monitoreo

```
[ ] GET /api/v1/metrics
    [ ] Query params: device_id, sede_id, from, to, aggregate (hour|day)
    [ ] Si aggregate, leer de metric_aggregations (jobs)
    [ ] Si no, leer de metric_readings

[ ] POST /api/v1/metrics (ingesta desde agentes)
    [ ] Batch insert para eficiencia
    [ ] Validar device_id existente y activo
    [ ] Encolar job de intrusion_detection si anomaly_detected = true

[ ] crates/jobs/src/jobs/metrics_aggregation_job.rs
    [ ] Agregación por hora: AVG, MAX, MIN de bandwidth, latency, packet_loss
    [ ] Agregación por día: rollup de agregaciones horarias
    [ ] Ejecutar vía cron cada hora
```

### VII.3 — Topología de Red

```
[ ] GET /api/v1/topology/:sede_id
    [ ] Grafo de conexiones de una sede
    [ ] Nodos: devices (con status visual)
    [ ] Aristas: device_links (con bandwidth y tipo)
    [ ] Estados visuales: online (verde), warning (amarillo), offline (rojo), maintenance (azul)
    [ ] Incluir métricas actuales en cada nodo

[ ] crates/topology/src/lib.rs
    [ ] Construir grafo desde devices + device_links
    [ ] Calcular dependencias jerárquicas (árbol de spanning)
    [ ] Detectar single points of failure
    [ ] Exportar formato GraphJSON para frontend (LayerChart)
```

**Nota:** La lógica de topología vive en `crates/topology/`, no solo en un handler.

### VII.4 — Alertas

```
[ ] GET /api/v1/alerts
    [ ] Query params: status, severity, device_id, from, to
    [ ] Orden: created_at DESC
    [ ] Paginación

[ ] POST /api/v1/alerts/:id/acknowledge
    [ ] Requiere permiso "alert:write"
    [ ] UPDATE status = 'acknowledged', acknowledged_by = user_id
    [ ] Log audit: alert_acknowledged

[ ] POST /api/v1/alerts/:id/resolve
    [ ] UPDATE status = 'resolved'
    [ ] Log audit: alert_resolved

[ ] Tipos de alertas generadas por jobs:
    [ ] DeviceOffline — last_seen_at > 5 min
    [ ] BandwidthSaturation — bandwidth > 80% de link capacity
    [ ] PacketLoss — packet_loss > 5% por 3 minutos consecutivos
    [ ] Intrusion — MAC no en whitelist
    [ ] TopologyChange — device_link creado/eliminado
    [ ] HighTraffic — bandwidth > 150% del baseline histórico
```

### VII.5 — Detección de Intrusos

```
[ ] GET /api/v1/intrusions
    [ ] Query params: status, from, to
    [ ] Orden: detected_at DESC

[ ] POST /api/v1/intrusions/:id/resolve
    [ ] Body: { status: "resolved" | "false_positive", notes: String }
    [ ] Requiere permiso "intrusion:write"
    [ ] Log audit: intrusion_resolved

[ ] crates/jobs/src/jobs/intrusion_detection_job.rs
    [ ] Estrategias:
        [ ] ARP scan comparativo (MACs nuevas vs whitelist)
        [ ] SNMP discovery de dispositivos no catalogados
        [ ] ICMP sweep de rangos IP
        [ ] Análisis de DHCP logs (si disponible)
    [ ] Whitelist: tabla device_whitelist (mac, description, approved_by)
    [ ] Si MAC detectada no está en whitelist ni en devices activos:
        [ ] Crear IntrusionEvent (status: detected)
        [ ] Crear Alert (severity: high, type: Intrusion)
        [ ] Notificar via alert_dispatch_job
```

---

## ADRs de referencia por bloque

| Bloque | ADRs |
|--------|------|
| I — Fundación | 0001, 0004, 0005, 0006 |
| II — API | 0003, 0007, 0009 |
| III — Seguridad | 0006, 0008 |
| IV — OpenAPI | 0016 |
| V — Async | 0015, 0016 |
| VI — Observabilidad | 0014, 0015 |
| VII — Monitoreo | 0020, 0015 |

---

## Diagrama de Flujo de Bloques

```
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE I — Fundación                                                  │
│  ├─ Pool PostgreSQL (PgPoolOptions)                                     │
│  ├─ Migraciones (users, roles, devices, metrics, alerts, intrusions)  │
│  ├─ crates/domain (entities, value_objects, ports, errors)             │
│  │   └─ Entidades de Monitoreo: Sede, Device, Metric, Alert          │
│  ├─ crates/application (use_cases)                                      │
│  └─ crates/database (repositories SQLx + Moka cache)                    │
│     └─ Ref: ADR 0020, 0001, 0004, 0006                               │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE II — API + Middleware                                          │
│  ├─ Axum setup (main.rs, router, state composition)                    │
│  ├─ Rutas de Monitoreo: sedes, devices, metrics, alerts, topology     │
│  ├─ Middleware ordenado (trace, request_id, cors, rate_limit, timeout)│
│  └─ Error handling (DomainError → AppError → IntoResponse)           │
│     └─ Ref: ADR 0003, 0007, 0009                                     │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE III — Seguridad                                                │
│  ├─ crates/auth (argon2id + PASETO v4) — NO JWT                      │
│  ├─ Endpoints /auth/* (register, login, refresh, logout)              │
│  ├─ Middleware auth (verify PASETO → UserId)                          │
│  ├─ Middleware RBAC (has_permission con cache Moka)                    │
│  └─ Middleware audit (logs)                                          │
│     └─ Ref: ADR 0008, 0006                                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE IV — OpenAPI + Scalar                                          │
│  ├─ utoipa ToSchema en todos los DTOs                                  │
│  ├─ #[utoipa::path] en cada handler                                    │
│  ├─ /docs → Scalar UI                                                  │
│  └─ /openapi.json → spec completa                                     │
│     └─ Ref: ADR 0016                                                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE V — Async (Jobs + Cache + Email)                               │
│  ├─ Moka cache decorator (TTL 5min, invalidación en writes)           │
│  ├─ Apalis jobs en crates/jobs/ (metrics, alerts, intrusion, cleanup)│
│  └─ Mailer dual (LogMailer en dev, Resend en prod)                    │
│     └─ Ref: ADR 0015, 0016                                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE VI — Observabilidad                                            │
│  ├─ Tracing JSON subscriber + request_id                               │
│  ├─ Sentry SDK (panics + errors)                                       │
│  ├─ Healthchecks.io (worker, backup verification)                       │
│  └─ Logs estructurados para auditoría                                │
│     └─ Ref: ADR 0014, 0015                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE VII — Monitoreo (ADR 0020)                                     │
│  ├─ Inventario: CRUD de sedes y dispositivos (soft delete)           │
│  ├─ Métricas: recolección, agregación, histórico                     │
│  ├─ Topología: mapa de red por sede (crates/topology/)               │
│  ├─ Alertas: detección, notificación, Ack/Resolve                     │
│  └─ Intrusiones: detección, resolución, whitelist                   │
│     └─ Ref: ADR 0020, 0015 (Jobs)                                    │
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
| **SvelteKit** | https://kit.svelte.dev | Frontend SSR |
| **pgBackRest** | https://pgbackrest.org | Backups PostgreSQL |
| **Healthchecks.io** | https://healthchecks.io | Monitoreo de tareas |

---

## Troubleshooting — Backend por Bloque

### Bloque I — Fundación

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `cargo check -p domain` falla | sqlx/tokio/async-trait importado en domain | Verificar Cargo.toml no tenga dependencias externas |
| `just migrate` error | DATABASE_URL no seteada | Exportar DATABASE_URL=postgres://... |
| Entidades de monitoreo no compilan | Falta algún campo | Revisar ADR 0020 y migraciones SQL |
| `jsonwebtoken` detectado | Violación ADR 0008 | Reemplazar por pasetors |

### Bloque II — API

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `cargo run --bin api` panic | Config inválida o DB no accesible | Revisar .env.local, verificar pool |
| `/health` retorna 500 | Pool no conecta | Verificar create_pool() y DATABASE_URL |
| CORS bloquea frontend | CorsLayer mal configurado | Verificar allow_origins en .env |

### Bloque VII — Monitoreo

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Métricas no se guardan | Job no encolado / agente no reporta | Verificar enqueue en use case, verificar agente |
| Topología vacía | device_links no creados | Crear enlaces entre dispositivos vía API |
| Alertas no se envían | Resend no configurado | Verificar RESEND_API_KEY y ENVIRONMENT |
| Intrusiones no detectadas | Whitelist vacía | Poblar device_whitelist con MACs conocidas |
| Soft delete no funciona | DELETE físico en lugar de archive | Verificar handler usa archive, no DELETE |

---

**Nota:** Este roadmap está basado en el ADR 0020 (Módulo de Monitoreo de Infraestructura Regional) para la Gobernación del Beni.
