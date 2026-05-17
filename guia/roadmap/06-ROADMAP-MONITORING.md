# 06-ROADMAP-MONITORING.md
# Roadmap — Módulo de Monitoreo de Infraestructura Regional

> **Fase del proyecto:** Monitoreo de Infraestructura Regional  
> **Dependencias:** Génesis (estructura de crates), Backend I (migraciones base), Backend II (API REST base), Backend III (Auth + RBAC)  
> **ADR Principal:** ADR-0020, ADR-0022, ADR-0023, ADR-0024, ADR-0025  
> **Timestamp:** 2026-05-16  
> **Estado:** ⏳ Pendiente — 0%

---

## Visión General

Este roadmap cubre el **dominio principal** del proyecto: el monitoreo activo de la infraestructura de red institucional de la Gobernación del Beni. Incluye descubrimiento de dispositivos, recolección de métricas, detección de anomalías, mapeo de topología, agentes distribuidos en sedes remotas y sincronización offline.

**Principio rector:** El monitoreo no es un módulo más; **es la razón de existir del sistema**. Por tanto, sus decisiones arquitectónicas (protocolo de red, formato de métricas, retención de datos) tienen prioridad sobre optimizaciones de frontend o DX.

---

## Dependencias de Fases Anteriores

| Fase | Requisito | ¿Bloqueante? |
|------|-----------|:------------:|
| Génesis | Workspace compilando, `crates/monitoring` y `crates/sync` creados | ✅ Sí |
| Backend I | Migraciones de `sedes`, `devices`, `device_links` aplicadas | ✅ Sí |
| Backend II | API REST base funcionando (healthcheck, errores estandarizados) | ✅ Sí |
| Backend III | Auth PASETO + RBAC operativo. Tabla `audit_logs` lista. | ✅ Sí |
| Backend IV | Jobs con Apalis configurados (para tareas de fondo de monitoreo) | ⚠️ Parcial |
| Frontend II | Stores de TanStack Query, tipos compartidos, vista de sedes lista | ⚠️ Parcial |

**Regla de paralelización válida:**
- Monitoreo I puede iniciarse en paralelo con Backend IV (jobs), pero no antes.
- Monitoreo II (agente) puede desarrollarse en paralelo con Frontend II/III, pero requiere que Monitoreo I haya definido el modelo de datos de red.

---

## Progreso General

```text
[░░░░░░░░░░░░░░░░░░] 0% — Monitoreo I: Fundamentos de Red
[░░░░░░░░░░░░░░░░░░] 0% — Monitoreo II: Agente y Recolección
[░░░░░░░░░░░░░░░░░░] 0% — Monitoreo III: Alertas, Topología e Intrusos
[░░░░░░░░░░░░░░░░░░] 0% — Monitoreo IV: Sync Offline y Resiliencia
```

---

## Monitoreo I — Fundamentos de Red y Modelo de Datos

**Objetivo:** Definir cómo el sistema representa la red, qué protocolos usa para descubrirla y establecer el contrato de datos entre agente, API y base de datos.

**Progreso:** 0% → objetivo 100% para cerrar esta fase.

### Tareas

#### 1. Modelo de dominio de red en `crates/domain`

- [ ] Crear `crates/domain/src/entities/device.rs`
  - Entidad `Device` con: `id`, `sede_id`, `hostname`, `ip_address`, `mac_address`, `device_type` (enum: Switch, Router, AccessPoint, Firewall, Server, Ups, Camera, Unknown), `status` (Active, Offline, Maintenance, Unreachable), `snmp_community` (opcional, encriptado), `created_at`, `updated_at`, `deleted_at` (Soft Delete).
  - Value object `MacAddress` con validación de formato IEEE 802 (`xx:xx:xx:xx:xx:xx`).
  - Value object `IpAddress` con validación IPv4/IPv6.
  - Value object `DeviceType` como enum con Display y FromStr.

- [ ] Crear `crates/domain/src/entities/sede.rs` (extensión)
  - Añadir a `Sede`: `network_cidr` (rango de red para descubrimiento), `agent_endpoint` (URL o identificador del agente local), `last_seen_at`.

- [ ] Crear `crates/domain/src/entities/metric_reading.rs`
  - Entidad `MetricReading`: `id`, `device_id`, `timestamp`, `bandwidth_rx_bps`, `bandwidth_tx_bps`, `latency_ms`, `packet_loss_percent`, `jitter_ms`, `anomaly_score` (f64, 0.0-1.0).
  - **Regla:** Esta entidad NO implementa Soft Delete. Implementa `RetentionPolicy` con `recorded_at` y TTL de 90 días (ADR-0025).

- [ ] Crear `crates/domain/src/entities/alert.rs`
  - Entidad `Alert`: `id`, `alert_type` (ConnectivityLoss, HighLatency, BandwidthSaturation, PacketLoss, IntrusionDetected, DeviceOffline), `severity` (Critical, High, Medium, Low), `device_id`, `sede_id`, `message`, `metadata` (JSONB lógico, como `serde_json::Value`), `status` (Open, Acknowledged, Resolved, FalsePositive), `created_at`, `resolved_at`, `acknowledged_by` (user_id opcional), `deleted_at` (Soft Delete).

- [ ] Crear `crates/domain/src/entities/intrusion_event.rs`
  - Entidad `IntrusionEvent`: `id`, `detected_mac`, `detected_ip`, `sede_id`, `device_id` (donde se detectó), `status` (Detected, Investigating, Resolved, FalsePositive), `first_seen_at`, `last_seen_at`, `notes`, `deleted_at` (Soft Delete).

- [ ] Crear `crates/domain/src/entities/network_snapshot.rs`
  - Entidad `NetworkSnapshot`: `id`, `sede_id`, `captured_at`, `topology_json` (grafo de red serializado), `device_count`, `link_count`.
  - **Regla:** Hard delete con retención de 30 días (ADR-0025). No soft delete.

- [ ] Crear `crates/domain/src/value_objects/mod.rs` y exportar todos los VOs.

- [ ] Tests unitarios para cada value object (validación de MAC, IP, CIDR).

#### 2. Migraciones de base de datos (PostgreSQL)

- [ ] Crear migración `migrations/2026XXXXXX_create_devices_table.sql`
  - Índices: `idx_devices_sede_id`, `idx_devices_mac` (UNIQUE parcial donde `deleted_at IS NULL`), `idx_devices_ip`, `idx_devices_status`.
  - Constraint: `device_type` como ENUM de PostgreSQL o CHECK.

- [ ] Crear migración `migrations/2026XXXXXX_create_device_links_table.sql`
  - Tabla `device_links`: `id`, `source_device_id`, `target_device_id`, `link_type` (Ethernet, Fiber, Wireless), `bandwidth_mbps`, `status`, `created_at`, `deleted_at`.
  - Índice: `idx_device_links_source_target`.

- [ ] Crear migración `migrations/2026XXXXXX_create_metric_readings_table.sql`
  - Tabla `metric_readings`: `id`, `device_id`, `recorded_at`, `bandwidth_rx_bps`, `bandwidth_tx_bps`, `latency_ms`, `packet_loss_percent`, `jitter_ms`, `anomaly_score`.
  - **Particionamiento:** Considerar particionamiento por `recorded_at` (rango mensual) desde el inicio. Si no, al menos índice BRIN o B-Tree en `recorded_at`.
  - Índice: `idx_metric_readings_device_recorded`.
  - **NO** incluir `deleted_at`. Incluir política de retención documentada.

- [ ] Crear migración `migrations/2026XXXXXX_create_alerts_table.sql`
  - Incluir `deleted_at` (Soft Delete).
  - Índices: `idx_alerts_status`, `idx_alerts_severity`, `idx_alerts_device_id`, `idx_alerts_created_at`.

- [ ] Crear migración `migrations/2026XXXXXX_create_intrusion_events_table.sql`
  - Incluir `deleted_at` (Soft Delete).
  - Índice: `idx_intrusion_mac_status`.

- [ ] Crear migración `migrations/2026XXXXXX_create_network_snapshots_table.sql`
  - Incluir `retention_until` (timestamp) en lugar de `deleted_at`.
  - Índice: `idx_snapshots_sede_captured`.

- [ ] Ejecutar `sqlx migrate run` y verificar que todas las migraciones aplican sin error.

- [ ] Generar `sqlx-data.json` o verificar `sqlx prepare` para queries en `crates/database`.

#### 3. Repositorios en `crates/database`

- [ ] Crear `crates/database/src/repositories/device_repository.rs`
  - Métodos: `create`, `find_by_id`, `find_by_sede`, `find_by_mac`, `update_status`, `soft_delete`.
  - Queries SQLx con `#[sqlx::query_as]`.

- [ ] Crear `crates/database/src/repositories/metric_repository.rs`
  - Métodos: `record_batch`, `find_by_device_and_range`, `aggregate_by_hour`, `prune_older_than` (hard delete).
  - **Optimización:** `record_batch` usa `UNNEST` o inserción múltiple para reducir round-trips.

- [ ] Crear `crates/database/src/repositories/alert_repository.rs`
  - Métodos: `create`, `find_open_by_severity`, `acknowledge`, `resolve`, `find_by_device`.

- [ ] Crear `crates/database/src/repositories/intrusion_repository.rs`
  - Métodos: `create`, `find_by_mac`, `update_status`, `find_recent`.

- [ ] Tests de integración para cada repositorio con `sqlx::test`.

#### 4. Definición del protocolo de descubrimiento de red (ADR-0024)

- [ ] Documentar en ADR-0024:
  - **Métodos de descubrimiento:** Ping sweep (ICMP) para identificar hosts activos en el CIDR de la sede.
  - **SNMP:** Versión 3 obligatoria (no v2c por seguridad). OIDs base a consultar: `1.3.6.1.2.1.1.5` (sysName), `1.3.6.1.2.1.1.1` (sysDescr), `1.3.6.1.2.1.2.2.1.6` (ifPhysAddress/MAC), `1.3.6.1.2.1.1.3` (sysUpTime).
  - **LLDP/CDP:** Si el dispositivo lo soporta, extraer vecinos para topología (`1.0.8802.1.1.2.1.4.1.1` para LLDP).
  - **ARP table:** Escaneo ARP local para detectar MACs no registradas (intrusos).
  - **Frecuencia:** Descubrimiento completo cada 6 horas. Heartbeat de métricas cada 60 segundos.

- [ ] Crear `crates/monitoring/src/discovery/mod.rs` — módulo de descubrimiento.

- [ ] Implementar `PingSweep` usando `surge-ping` (ICMP).
  - Función `discover_active_hosts(cidr: &str) -> Vec<IpAddress>`.
  - Timeout por host: 2 segundos. Concurrencia controlada (max 50 pings paralelos).

- [ ] Implementar `SnmpProbe` usando `snmp` crate.
  - Función `probe_device(ip: &IpAddress, credentials: &SnmpV3Credentials) -> Result<DeviceInfo, MonitoringError>`.
  - Manejo de errores: timeout, authentication failure, unsupported OID.

- [ ] Tests unitarios con mocks para SNMP (simular respuestas OID).

---

**Verificación de cierre de Monitoreo I:**

```bash
# Verificar que domain no tiene dependencias externas
cargo tree -p domain --depth 1
# Esperado: solo domain + thiserror + uuid + time + serde

# Verificar que monitoring no depende de database directamente
# (solo a través de application/ports)
cargo tree -p monitoring --depth 2 | grep database
# Esperado: ninguna línea que muestre database como dependencia directa

# Ejecutar tests
cargo test -p domain -p database -p monitoring
```

---

## Monitoreo II — Agente de Sede y Recolección de Métricas

**Objetivo:** Construir el agente Rust ligero que corre en cada sede remota, recolecta métricas y las envía al servidor central.

**Progreso:** 0%

### Tareas

#### 1. Estructura del crate `apps/agent`

- [ ] Crear `apps/agent/Cargo.toml`
  - Dependencias: `tokio`, `surge-ping`, `snmp`, `serde`, `serde_json`, `time`, `uuid`, `reqwest` (o `hyper` si se elige HTTP/2), `rusqlite` (SQLite embebido para buffer offline), `tracing`, `config` (ADR-0002).
  - **NO** depende de `domain` ni `database` directamente. El agente es autónomo. Define sus propios tipos serializables o usa un crate `monitoring-protocol` compartido.

- [ ] Crear `apps/agent/src/main.rs`
  - Inicialización: leer configuración (URL del servidor central, sede_id, credenciales SNMP, intervalos).
  - Fail-fast si falta `SERVER_URL` o `SEDE_ID`.

- [ ] Crear `apps/agent/src/config.rs`
  - Configuración tipeada: `AgentConfig { sede_id: Uuid, server_url: String, api_key: String, snmp: SnmpConfig, intervals: IntervalConfig }`.

#### 2. Buffer local y sync offline del agente

- [ ] Crear `apps/agent/src/storage/mod.rs`
  - SQLite embebido con esquema:
    - `pending_metrics`: id, device_id, recorded_at, payload_json, retry_count, created_at.
    - `pending_alerts`: id, alert_type, severity, device_id, message, payload_json, retry_count.
    - `pending_heartbeats`: id, sent_at, status, response_code.
  - **Límite:** Buffer máximo de 10,000 métricas. Si se excede, descartar las más antiguas (FIFO).

- [ ] Implementar `MetricBuffer::enqueue(metric: MetricPayload) -> Result<(), BufferError>`.

- [ ] Implementar `MetricBuffer::dequeue_batch(limit: usize) -> Vec<MetricPayload>`.

- [ ] Implementar `MetricBuffer::mark_sent(ids: Vec<i64>)` y `MetricBuffer::mark_failed(ids: Vec<i64>, increment_retry)`.

- [ ] Política de reintentos: 3 reintentos con backoff exponencial (5s, 25s, 125s). Después, mover a `dead_letter` tabla local para inspección manual.

#### 3. Recolección de métricas

- [ ] Crear `apps/agent/src/collectors/mod.rs`.

- [ ] Implementar `PingCollector`
  - Para cada `Device` conocido (lista sincronizada desde servidor), ejecutar ping cada 60s.
  - Métricas: `latency_ms`, `packet_loss` (0% o 100% por ping), `reachable` (bool).

- [ ] Implementar `SnmpCollector`
  - Para dispositivos con SNMP habilitado, consultar OIDs de tráfico:
    - `1.3.6.1.2.1.2.2.1.10.*` (ifInOctets) → bandwidth_rx.
    - `1.3.6.1.2.1.2.2.1.16.*` (ifOutOctets) → bandwidth_tx.
  - Calcular delta de octets entre lecturas para obtener bps.
  - Manejar counter wrap (32-bit vs 64-bit counters).

- [ ] Implementar `ArpCollector`
  - Leer tabla ARP local del sistema operativo (`/proc/net/arp` en Linux, `arp -a` en Windows/Mac).
  - Comparar MACs detectadas contra lista de MACs autorizadas (sincronizada desde servidor).
  - Si MAC desconocida → generar `IntrusionEvent` y enviar inmediatamente (no esperar batch).

#### 4. Envío al servidor central

- [ ] Implementar `apps/agent/src/sender/mod.rs`
  - **Protocolo:** HTTP/2 POST con compresión gzip (decisión provisional hasta ADR-0023).
  - Endpoints:
    - `POST /api/v1/agents/heartbeat` — cada 60s. Payload: `{sede_id, timestamp, agent_version, buffer_status}`.
    - `POST /api/v1/agents/metrics` — batch de métricas. Payload: `Vec<MetricPayload>`.
    - `POST /api/v1/agents/alerts` — alertas urgentes (intrusos, offline crítico).
  - Headers: `Authorization: Bearer <api_key>` (API key de larga duración, no PASETO por request para reducir overhead — evaluar mTLS en ADR-0023).
  - Timeout: 10s. Reintentos con backoff (ver buffer local).

- [ ] Implementar compresión gzip del payload antes de envío.

- [ ] Implementar `sender.rs` con `reqwest::Client` reutilizable y connection pooling.

#### 5. Sincronización de configuración descendente

- [ ] Implementar `GET /api/v1/agents/config` cada 5 minutos.
  - El agente descarga: lista de dispositivos a monitorear, credenciales SNMP (encriptadas en tránsito vía TLS 1.3), umbrales de alerta, MACs autorizadas.
  - Almacenar localmente en SQLite para operación offline.

- [ ] Implementar `GET /api/v1/agents/commands` (long-polling o SSE del agente al servidor).
  - El servidor puede enviar comandos: "reescanear ahora", "cambiar intervalo a 30s", "reiniciar".
  - **Nota:** Esto requiere canal bidireccional. Si se usa HTTP/2, puede ser long-polling. Si se usa MQTT, es pub/sub nativo.

---

**Verificación de cierre de Monitoreo II:**

```bash
# Compilar el agente standalone
cargo build -p agent --release
# Verificar que el binario es < 15MB (objetivo para sedes remotas)
ls -lh target/release/agent

# Ejecutar agente en modo dry-run (sin servidor)
./target/release/agent --config apps/agent/config.example.toml --dry-run
# Esperado: descubre dispositivos locales, imprime métricas en stdout, no envía al servidor.
```

---

## Monitoreo III — Alertas, Topología de Red y Detección de Intrusos

**Objetivo:** Transformar métricas brutas en alertas accionables, mantener un grafo de topología de red actualizado y detectar dispositivos no autorizados.

**Progreso:** 0%

### Tareas

#### 1. Motor de alertas en `crates/monitoring`

- [ ] Crear `crates/monitoring/src/alerts/engine.rs`
  - `AlertEngine::evaluate(reading: &MetricReading, thresholds: &ThresholdConfig) -> Vec<Alert>`.
  - Reglas evaluadas:
    - `latency_ms > threshold.latency_critical` → Alerta Critical.
    - `packet_loss_percent > threshold.packet_loss_high` → Alerta High.
    - `bandwidth_tx_bps > threshold.bandwidth_max * 0.95` → Alerta High (BandwidthSaturation).
    - `anomaly_score > 0.85` → Alerta Critical (AnomalyDetected).

- [ ] Crear `crates/monitoring/src/alerts/thresholds.rs`
  - `ThresholdConfig` por tipo de dispositivo y por sede.
  - Umbrales por defecto: Latencia > 100ms (High), > 500ms (Critical). Packet loss > 5% (High), > 20% (Critical).

- [ ] Implementar deduplicación de alertas:
  - No crear alerta duplicada si ya existe una alerta `Open` del mismo tipo para el mismo dispositivo en las últimas 15 minutos.
  - En su lugar, actualizar `updated_at` y añadir nota "Persiste".

- [ ] Implementar escalación automática:
  - Si un dispositivo está Offline por más de 5 minutos → escalar de Medium a High.
  - Si persiste por más de 30 minutos → escalar a Critical.

#### 2. API de alertas en `apps/api`

- [ ] Crear endpoint `GET /api/v1/alerts`
  - Query params: `status`, `severity`, `sede_id`, `device_id`, `from`, `to`.
  - Paginación con cursor (no offset) para grandes volúmenes.
  - Respuesta incluye `total_open`, `total_critical`.

- [ ] Crear endpoint `POST /api/v1/alerts/:id/acknowledge`
  - Body: `{user_id, notes}`.
  - Requiere permiso `alerts:acknowledge` (RBAC ADR-0006).
  - Genera `audit_log` automático.

- [ ] Crear endpoint `POST /api/v1/alerts/:id/resolve`
  - Body: `{resolution_notes}`.
  - Requiere permiso `alerts:resolve`.
  - Genera `audit_log`.

- [ ] Crear endpoint `GET /api/v1/alerts/stats`
  - Métricas agregadas: alertas por severidad, por sede, por tipo, promedio de tiempo de resolución (MTTR).

#### 3. Topología de red

- [ ] Crear `crates/monitoring/src/topology/graph.rs`
  - Estructura `NetworkGraph` usando `petgraph` (crate de grafos en Rust).
  - Nodos: `DeviceNode { device_id, ip, mac, device_type }`.
  - Edges: `DeviceLink { source, target, link_type, bandwidth_mbps }`.

- [ ] Implementar `NetworkGraph::build_from_devices_and_links(devices, links) -> Self`.

- [ ] Implementar `NetworkGraph::detect_orphans() -> Vec<DeviceId>` — dispositivos sin conexión.

- [ ] Implementar `NetworkGraph::find_critical_paths() -> Vec<Vec<DeviceId>>` — caminos cuya pérdida desconecta la red (usar articulation points de petgraph).

- [ ] Crear endpoint `GET /api/v1/topology/:sede_id`
  - Devuelve grafo en formato JSON: `{nodes: [...], edges: [...]}`.
  - Incluir `last_updated` (timestamp del último descubrimiento).

- [ ] Crear endpoint `POST /api/v1/topology/:sede_id/refresh`
  - Dispara descubrimiento inmediato (job en Apalis).
  - Requiere permiso `topology:refresh`.

- [ ] Implementar `NetworkSnapshotService`
  - Cada 6 horas, serializar el grafo actual a JSON y guardar en `network_snapshots`.
  - Permitir comparación histórica: "¿Qué cambió en la topología desde ayer?"

#### 4. Detección de intrusos

- [ ] Crear `crates/monitoring/src/intrusion/detector.rs`
  - `IntrusionDetector::check_mac(mac: &MacAddress, authorized_macs: &[MacAddress]) -> bool`.
  - `IntrusionDetector::scan_arp_table(arp_entries, authorized) -> Vec<IntrusionEvent>`.

- [ ] Implementar lista blanca de MACs:
  - Tabla `authorized_macs`: `id`, `sede_id`, `mac_address`, `description`, `created_by`.
  - Endpoint `POST /api/v1/intrusion/whitelist` para añadir MACs conocidas.

- [ ] Implementar endpoint `GET /api/v1/intrusion/events`
  - Query params: `status`, `sede_id`, `from`, `to`.
  - Incluir sugerencia automática: "¿Marcar como False Positive?" si la MAC coincide con un patrón conocido (vendor OUI).

- [ ] Implementar notificación de intrusos:
  - Si `IntrusionEvent` con status `Detected` se crea → enviar email inmediato vía Resend a admins de la sede.
  - Usar job de Apalis para no bloquear el request del agente.

---

**Verificación de cierre de Monitoreo III:**

```bash
# Tests de integración: motor de alertas
cargo test -p monitoring --test alert_engine

# Tests de integración: topología
cargo test -p monitoring --test topology_graph

# Verificar que no hay alertas duplicadas en base de datos
# (query de validación)
sqlx query "SELECT alert_type, device_id, COUNT(*) FROM alerts WHERE status = 'Open' GROUP BY alert_type, device_id HAVING COUNT(*) > 1"
# Esperado: 0 filas.
```

---

## Monitoreo IV — Sincronización Offline y Resiliencia

**Objetivo:** Garantizar que el sistema funcione cuando la conectividad entre sede y central se interrumpe, con reconciliación clara al restablecerse.

**Progreso:** 0%

### Tareas

#### 1. Sync queue del agente (extensión de Monitoreo II)

- [ ] Implementar `apps/agent/src/sync/reconciler.rs`
  - Al reconectar después de corte, el agente debe:
    1. Enviar heartbeats pendientes (prioridad alta — el servidor necesita saber que la sede está viva).
    2. Enviar alertas pendientes (prioridad alta — intrusos no pueden esperar).
    3. Enviar métricas pendientes en batch (prioridad media — pueden perderse algunas lecturas intermedias).
  - Orden: FIFO por tipo de dato, no estrictamente global.

- [ ] Implementar compresión diferencial de métricas:
  - Si el buffer tiene 6 horas de métricas (21,600 lecturas de 1/min), no enviar todas.
  - Enviar: promedio por minuto de las métricas de conectividad, y los picos (max latency, max packet loss).
  - Descartar lecturas "normales" intermedias si el buffer excede 1 hora.
  - **Documentar en ADR-0021:** Política de degradación graceful: "Si offline > 1h, enviar agregados en lugar de lecturas individuales."

#### 2. Sync offline del frontend (SPA)

- [ ] Implementar `apps/web/src/lib/sync/` (TypeScript)
  - `SyncEngine` con TanStack Query:
    - Cuando online: todas las queries/mutations van al API.
    - Cuando offline: mutations se encolan en IndexedDB/SQLite Wasm.
    - Al reconectar: reenviar mutations en orden.

- [ ] Implementar `apps/web/src/lib/sync/queue.ts`
  - Estructura de cola: `{id, operation: 'create'|'update'|'delete', entity: 'device'|'alert'|'sede', payload, timestamp, retry_count}`.
  - Límite: 500 operaciones pendientes. Si se excede, bloquear UI y mostrar: "Demasiados cambios pendientes. Conecte a internet para sincronizar."

- [ ] Implementar resolución de conflictos (definir en ADR-0021):
  - **Estrategia propuesta:** Last-Write-Wins por entidad, con ventana de 5 minutos de gracia.
  - Si un `Device` fue editado offline y también editado en el servidor por otro usuario, comparar `updated_at`.
  - Si conflicto real → marcar como `Conflict` y mostrar al usuario para resolución manual.
  - **NO** implementar CRDTs en MVP. Son overkill para este dominio.

- [ ] Implementar indicador visual de estado de sync:
  - Barra superior: "🟢 Sincronizado", "🟡 Cambios pendientes (12)", "🔴 Offline — operando localmente".

#### 3. Resiliencia del servidor central

- [ ] Implementar `crates/infrastructure/src/circuit_breaker.rs`
  - Si un agente no envía heartbeat por más de 3 intervalos (3 minutos si intervalo = 60s), marcar sede como `Unreachable`.
  - Generar alerta automática: `AlertType::SedeOffline`.
  - Al recibir heartbeat nuevamente, marcar como `Reachable` y resolver alerta automáticamente.

- [ ] Implementar `crates/monitoring/src/health/sede_health.rs`
  - `SedeHealthMonitor`: revisa cada 60s el `last_heartbeat_at` de cada sede.
  - Actualiza campo `sedes.connectivity_status` (Online, Degraded, Offline).

- [ ] Implementar endpoint `GET /api/v1/health/sedes`
  - Dashboard de conectividad: lista de sedes con estado, último heartbeat, buffer del agente (cuántas métricas tiene pendientes).

---

**Verificación de cierre de Monitoreo IV:**

```bash
# Simular corte de red del agente
# (usar herramienta de red o configurar server_url a IP inexistente)
AGENT_SERVER_URL=http://0.0.0.0:9999 cargo run -p agent -- --dry-run false
# Esperado: agente recolecta métricas, las guarda en SQLite local, no falla.

# Restaurar conectividad
# Esperado: agente envía batch pendiente, vacía buffer local.

# Verificar frontend offline
# Desconectar navegador de red, crear una alerta (acknowledge), reconectar.
# Esperado: alerta se sincroniza, sin duplicados.
```

---

## Checklist de Validación Arquitectónica Final

Antes de declarar el módulo de Monitoreo como "completo", verificar:

- [ ] `crates/domain` **no** importa `sqlx`, `axum`, `reqwest`, `snmp`, `surge-ping`.
- [ ] SQL **solo** en `crates/database/repositories/`.
- [ ] `apps/agent` es un binario **standalone**. No depende de `domain` ni `database`.
- [ ] `metric_readings` **no** tiene columna `deleted_at`. Tiene política de retención documentada.
- [ ] `alerts` e `intrusion_events` **sí** tienen `deleted_at` (Soft Delete).
- [ ] SSE se usa **solo** para push de alertas al navegador.
- [ ] Agente→servidor usa HTTP/2 POST (o protocolo definido en ADR-0023), no SSE.
- [ ] Todo endpoint de mutación de alertas genera `audit_log`.
- [ ] `cargo test -p monitoring` pasa al 100%.
- [ ] Binario del agente pesa < 20MB en release.

---

**Proyecto:** Monitoreo de Infraestructura Regional — Gobernación del Beni  
**Referencia Principal:** ADR-0020, ADR-0022, ADR-0023, ADR-0024, ADR-0025
