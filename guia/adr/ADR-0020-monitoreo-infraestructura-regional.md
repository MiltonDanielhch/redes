# ADR 0020 — Módulo de Monitoreo de Infraestructura Regional

| Campo               | Valor                                                                                                                                                  |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                                                                             |
| **Fecha**           | 2026-05-16                                                                                                                                             |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                                                       |
| **Versión**         | 2.1 (Corrección 2026-05-16)                                                                                                                            |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0003 (Axum), ADR 0004 (PostgreSQL), ADR 0006 (RBAC), ADR 0008 (PASETO), ADR 0015 (Jobs Apalis), ADR 0017 (Frontend SvelteKit), ADR 0021 (Local-First Sync Offline), ADR 0022 (Agentes Monitoreo Distribuidos) |

---

# Contexto

La Gobernación del Beni requiere una plataforma centralizada para:

* monitorear infraestructura de red institucional
* visualizar sedes regionales
* detectar fallas rápidamente
* identificar dispositivos no autorizados
* analizar consumo de ancho de banda
* generar auditorías y reportes técnicos

Actualmente la información de infraestructura:

* está dispersa
* no tiene trazabilidad
* depende de revisiones manuales
* no posee monitoreo histórico
* dificulta detectar problemas de conectividad

Además, muchas sedes operan con conectividad inestable, por lo que el sistema debe tolerar:

* latencia alta
* cortes de internet
* sincronización diferida
* operación offline parcial

---

# Decisión

Construir un módulo dedicado de:

# Monitoreo de Infraestructura Regional

capaz de:

* mapear dispositivos físicos
* visualizar topología de red
* analizar tráfico institucional
* detectar anomalías
* generar alertas
* producir reportes históricos
* operar sobre arquitectura Local-First

---

# Objetivos del módulo

## Técnicos

* observabilidad centralizada
* monitoreo multi-sede
* baja latencia
* realtime eficiente
* consumo mínimo de recursos

## Operativos

* auditoría institucional
* detección temprana de problemas
* control de activos tecnológicos
* trazabilidad histórica
* reducción de tiempos de diagnóstico

---

# Arquitectura general

```text
Sedes Regionales
   ↓
Agentes/Sensores Locales (apps/agent) ← ADR 0022
   ↓
API Axum (apps/api) ← ADR 0003
   ↓
Jobs de procesamiento (crates/jobs - Apalis) ← ADR 0015
   ↓
PostgreSQL + métricas históricas (crates/database) ← ADR 0004
   ↓
Dashboard SvelteKit realtime (apps/web) ← ADR 0017
```

**Nota:** PostgreSQL es la base de datos principal del backend (ADR 0004). SQLite se usa únicamente en el frontend/browser para operación offline (ADR 0021 — Local-First), nunca como DB principal del servidor.

---

# Componentes del módulo

| Componente  | Responsabilidad                   | Crate/App |
| ----------- | --------------------------------- | --------- |
| `inventory` | Inventario físico de dispositivos | crates/inventory |
| `topology`  | Mapeo visual de conexiones        | crates/topology |
| `metrics`   | Métricas de red                   | crates/database |
| `alerts`    | Detección de anomalías            | crates/jobs |
| `audit`     | Bitácora y reportes               | crates/database |
| `agents`    | Recolección distribuida           | apps/agent |
| `sync`      | Sincronización offline            | crates/sync |

---

# Funcionalidades principales

# 1 — Inventario físico de dispositivos

## Objetivo

Registrar toda la infraestructura tecnológica institucional.

## Entidades monitoreadas

* switches
* access points
* routers
* firewalls
* servidores
* enlaces WAN
* UPS
* cámaras IP
* enlaces inalámbricos

---

## Información registrada

| Campo     | Descripción                    |
| --------- | ------------------------------ |
| hostname  | Nombre del dispositivo         |
| IP        | Dirección IP (IPv4)            |
| MAC       | Dirección física (IEEE 802)    |
| ubicación | Oficina física                 |
| sede      | Dependencia institucional      |
| vendor    | Fabricante                     |
| modelo    | Modelo                         |
| firmware  | Versión                        |
| estado    | Activo / caído / mantenimiento |

---

# 2 — Topología de red

## Objetivo

Visualizar gráficamente las conexiones físicas y lógicas.

---

## Capacidades

* mapa por sede
* dependencias jerárquicas
* enlaces entre switches
* uplinks WAN
* APs conectados
* estado visual por color
* detección de single points of failure

---

## Estados visuales

| Estado        | Color    |
| ------------- | -------- |
| Online        | Verde    |
| Advertencia   | Amarillo |
| Offline       | Rojo     |
| Mantenimiento | Azul     |

---

# 3 — Monitoreo de ancho de banda

## Métricas recolectadas

* tráfico RX/TX (bytes)
* uso por secretaría
* saturación WAN
* throughput histórico
* utilización pico
* latencia (ms)
* packet loss (%)
* jitter (ms)

---

## Estrategia realtime

Por defecto:

* **SSE** (Server-Sent Events) — preferido (ADR 0017)
* Polling adaptativo (fallback si SSE no disponible)
* WebSocket solo si realmente es necesario (casos excepcionales)

**Nota:** La comunicación entre frontend y backend es REST + SSE. No se usa ConnectRPC ni gRPC en el stack web.

---

# 4 — Detección de dispositivos intrusos

## Objetivo

Detectar dispositivos no autorizados dentro de la red institucional.

---

## Estrategia

### Descubrimiento

* ARP scan
* SNMP discovery
* ICMP sweep
* DHCP logs (si disponible)
* MAC learning en switches

---

## Validación

Comparar:

```text
Dispositivo detectado
VS
Whitelist institucional (device_whitelist)
VS
Inventario conocido (devices)
```

---

## Resultado

| Estado         | Acción              |
| -------------- | ------------------- |
| Conocido       | Registrar heartbeat |
| Desconocido    | Generar alerta      |
| Duplicado      | Marcar anomalía     |
| MAC sospechosa | Escalar a auditoría |

---

# 5 — Alertas y anomalías

## Eventos críticos

* dispositivo offline
* saturación WAN
* packet loss alto
* AP desconectado
* intrusión detectada
* switch sin respuesta
* cambios de topología
* exceso de tráfico

---

## Canales de alerta

| Canal     | Uso                   |
| --------- | --------------------- |
| Dashboard | Tiempo real (SSE)     |
| Email     | Incidentes críticos   |
| Logs      | Auditoría             |
| Webhook   | Integraciones futuras |

---

# Modelo de dominio

## Sede

```rust
pub struct Sede {
    pub id:          SedeId,
    pub nombre:      String,
    pub ubicacion:   String,
    pub secretaria:  String,
    pub created_at:  OffsetDateTime,
    pub updated_at:  OffsetDateTime,
    pub deleted_at:  Option<OffsetDateTime>,  // Soft Delete (ADR 0006)
}
```

---

## Dispositivo

```rust
pub enum DeviceType {
    Switch,
    AccessPoint,
    Router,
    Firewall,
    Server,
    Ups,           // Uninterruptible Power Supply
    Camera,        // Cámara IP
    WirelessLink,  // Enlace inalámbrico/WAN
}

pub enum DeviceStatus {
    Active,
    Offline,
    Maintenance,
}

pub struct Device {
    pub id:             DeviceId,
    pub hostname:       String,
    pub ip_address:     String,           // Validación IPv4 en value object
    pub mac_address:    String,           // Validación IEEE 802 en value object
    pub device_type:    DeviceType,
    pub status:         DeviceStatus,
    pub sede_id:        SedeId,
    pub last_seen_at:   Option<OffsetDateTime>,
    pub created_at:     OffsetDateTime,
    pub updated_at:     OffsetDateTime,
    pub deleted_at:     Option<OffsetDateTime>,  // Soft Delete (ADR 0006)
}
```

---

## Enlace entre dispositivos

```rust
pub enum LinkType {
    Ethernet,
    Fiber,
    Wireless,
    Serial,
}

pub struct DeviceLink {
    pub id:               LinkId,
    pub source_device_id: DeviceId,
    pub target_device_id: DeviceId,
    pub link_type:        LinkType,
    pub bandwidth_mbps:   Option<i32>,
    pub status:           DeviceStatus,
    pub created_at:       OffsetDateTime,
    pub updated_at:       OffsetDateTime,
}
```

---

## Métrica

```rust
pub struct MetricReading {
    pub id:                 ReadingId,
    pub device_id:          DeviceId,
    pub bandwidth_rx_bytes: i64,
    pub bandwidth_tx_bytes: i64,
    pub latency_ms:         Option<i32>,      // Precisión suficiente, evita f32
    pub packet_loss_percent: Option<f64>,     // Doble precisión para agregaciones
    pub anomaly_detected:   bool,
    pub created_at:         OffsetDateTime,
}
```

---

## Alerta

```rust
pub enum AlertType {
    DeviceOffline,
    BandwidthSaturation,
    PacketLoss,
    Intrusion,
    TopologyChange,
    HighTraffic,
}

pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
}

pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
}

pub struct Alert {
    pub id:              AlertId,
    pub alert_type:      AlertType,
    pub severity:        AlertSeverity,
    pub device_id:       Option<DeviceId>,
    pub message:         String,
    pub details:         Option<String>,
    pub status:          AlertStatus,
    pub acknowledged_by: Option<UserId>,
    pub acknowledged_at: Option<OffsetDateTime>,
    pub created_at:      OffsetDateTime,
    pub updated_at:      OffsetDateTime,
}
```

---

## Evento de intrusión

```rust
pub enum IntrusionStatus {
    Detected,
    Investigating,
    Resolved,
    FalsePositive,
}

pub struct IntrusionEvent {
    pub id:           IntrusionId,
    pub mac_address:  String,
    pub ip_address:   Option<String>,
    pub device_id:    Option<DeviceId>,
    pub status:       IntrusionStatus,
    pub detected_at:  OffsetDateTime,
    pub resolved_at:  Option<OffsetDateTime>,
    pub notes:        Option<String>,
    pub created_at:   OffsetDateTime,
    pub updated_at:   OffsetDateTime,
}
```

---

## Whitelist de dispositivos

```rust
pub struct DeviceWhitelist {
    pub id:          WhitelistId,
    pub mac_address: String,
    pub description: String,
    pub approved_by: UserId,
    pub approved_at: OffsetDateTime,
    pub created_at:  OffsetDateTime,
}
```

---

# Arquitectura frontend

## Stack

| Tecnología     | Uso                 | Versión referencia |
| -------------- | ------------------- | ------------------ |
| SvelteKit      | Dashboard SSR       | 2.x (latest)       |
| Svelte 5 Runes | Reactividad         | 5.x (latest)       |
| LayerChart     | Gráficos            | latest             |
| SSE            | Realtime            | nativo (EventSource) |
| TailwindCSS    | UI                  | 4.x                |
| TanStack Query | Caching de datos    | 5.x                |
| ArkType        | Validación runtime  | 2.x                |

**Nota:** La comunicación frontend-backend es REST (JSON) + SSE para realtime. OpenAPI/Utoipa (ADR 0016) genera la documentación de la API. No se usa ConnectRPC ni gRPC.

---

# Estrategia realtime

```text
Agente (sedes)
   ↓
Axum API (apps/api)
   ↓
Apalis procesa métricas (crates/jobs)
   ↓
PostgreSQL almacena
   ↓
SSE emite actualización (endpoint /api/v1/stream)
   ↓
Dashboard SvelteKit actualiza gráficos
```

---

# Estrategia offline (Local-First)

> **Referencia:** ADR 0021

* SQLite Wasm en browser (cache local de lectura)
* Sync queue para acciones pendientes (IndexedDB)
* Reconciliación cuando vuelve la conexión
* Dashboard parcialmente funcional sin internet
* Métricas recientes cacheadas localmente

---

# Base de datos

## Tablas principales

```text
sedes
devices
device_links          -- conexiones entre dispositivos
device_whitelist      -- MACs aprobadas
metric_readings
metric_aggregations   -- rollup por hora/día (jobs)
alerts
intrusion_events
audit_logs
sessions
tokens                -- verificación email + reset password
users
roles
permissions
role_permissions
user_roles
```

---

# Estrategia de almacenamiento

| Tipo de dato      | Estrategia             | Tecnología |
| ----------------- | ---------------------- | ---------- |
| Inventario        | Persistente (PostgreSQL) | PostgreSQL 17+ |
| Métricas realtime | Time-series (PostgreSQL) | PostgreSQL 17+ |
| Alertas           | Persistente (PostgreSQL) | PostgreSQL 17+ |
| Logs              | Retención configurable (PostgreSQL) | PostgreSQL 17+ |
| Cache frontend    | SQLite Wasm (Local-First) | sqlite-wasm |
| Sync queue        | IndexedDB (browser)    | browser API |

---

# Retención de métricas

| Tipo               | Retención  |
| ------------------ | ---------- |
| Métricas crudas    | 30 días    |
| Métricas agregadas | 1 año      |
| Alertas            | Permanente |
| Auditoría          | Permanente |
| Intrusiones        | Permanente |

---

# Agentes de monitoreo

## Objetivo

Recolectar métricas desde sedes remotas.

---

## Capacidades

* heartbeat periódico al API
* métricas SNMP (switches, routers)
* escaneo ARP (detección de dispositivos)
* monitoreo ICMP (latencia, packet loss)
* detección de dispositivos nuevos
* buffering offline (almacena localmente si sin conexión)
* sync automático al reconectar

---

## Estrategia

```text
Sede sin internet
   ↓
Agente almacena métricas localmente (SQLite/file)
   ↓
Reconexión
   ↓
Sync automático vía API (batch insert)
   ↓
Confirmación de recepción
```

---

# Seguridad

## Reglas obligatorias

* RBAC institucional (ADR 0006)
* auditoría obligatoria (ADR 0006)
* PASETO v4 auth (ADR 0008) — JWT prohibido
* rate limiting (ADR 0009)
* cifrado TLS (HTTPS)
* logs firmados (opcional)
* detección de anomalías

---

# Roles RBAC

| Rol       | Permisos         |
| --------- | ---------------- |
| Admin     | Acceso total     |
| Operator  | Lectura + ack alerts + write intrusions |
| Viewer    | Solo lectura     |
| Agent     | Write metrics + read devices (para apps/agent) |

**Nota:** Alineado con ADR 0006. Los permisos se definen como "recurso:acción" (ej: "devices:read", "alerts:write").

---

# Auditoría

Toda acción crítica genera log:

```text
user_id
action (ej: device_created, alert_acknowledged)
resource (ej: devices, alerts)
resource_id
details (JSON)
ip_address
user_agent
timestamp
```

---

# Reportes

## Exportación

* PDF
* CSV
* JSON
* snapshots históricos de topología

---

## Reportes disponibles

* consumo por secretaría
* disponibilidad mensual (uptime %)
* incidentes críticos
* topología actual
* inventario institucional
* intrusiones detectadas
* métricas agregadas por período

---

# Integración con Apalis

## Jobs

| Job                     | Función                       | Frecuencia |
| ----------------------- | ----------------------------- | ---------- |
| `MetricsAggregationJob` | Agregación histórica (hourly/daily) | Cada hora |
| `IntrusionDetectionJob` | Análisis de anomalías y whitelist | Cada 5 min |
| `AlertDispatchJob`      | Envío de alertas por email      | On-demand (event-driven) |
| `CleanupMetricsJob`     | Limpieza de métricas crudas > 30 días | Diario |
| `SyncOfflineJob`        | Procesar sync_queue de sedes offline | Cada 2 min |

---

# Alternativas consideradas

| Opción                   | Motivo de descarte                      |
| ------------------------ | --------------------------------------- |
| Zabbix completo          | Muy pesado para VPS pequeños            |
| LibreNMS                 | Excelente pero menos integrado al stack |
| Grafana + Prometheus     | Overkill para MVP regional              |
| PRTG                     | Licenciamiento propietario              |
| Cloud monitoring externo | Dependencia de internet permanente      |

---

# Herramientas y Librerías para Optimizar (Edición 2026-05-16)

| Herramienta  | Propósito                 | Crate/Package | Versión fijada |
| ------------ | ------------------------- | ------------- | -------------- |
| `async-snmp` | Recolección SNMP async    | crates/snmp   | `0.12.0`       |
| `surge-ping` | ICMP async                | crates/snmp   | `0.8.4`        |
| `tokio`      | Concurrencia async        | workspace     | `1.51` (LTS)   |
| `Apalis`     | Procesamiento background  | crates/jobs   | `1.0.0-rc.9`   |
| `SSE`        | Streaming eficiente       | apps/api      | nativo Axum    |
| `tracing`    | Observabilidad            | workspace     | `0.1.44`       |
| `sentry`     | Monitoreo de errores      | apps/api      | `0.47.0`       |
| `LayerChart` | Visualización realtime    | apps/web      | latest         |
| `sqlx`       | PostgreSQL queries        | crates/database | `0.8.6`      |
| `pasetors`   | PASETO v4 tokens          | crates/auth   | `0.7.8`        |
| `argon2`     | Password hashing          | crates/auth   | `0.5.3`        |
| `utoipa`     | OpenAPI generation        | crates/infrastructure | `5.5.0` |

**Cambios respecto a v2.0:**
- Se fijan versiones exactas de todas las dependencias del módulo basadas en latest estable al 2026-05-16
- `async-snmp` 0.12.0 (abr 2026) — async-first SNMP client; requiere Rust 1.88+ (compatible con Rust 1.95.0 del Containerfile, ADR 0019)
- `surge-ping` 0.8.4 (mar 2026) — ICMP async con tokio
- `tokio` 1.51 (LTS hasta marzo 2027, MSRV 1.70)
- `tracing` 0.1.44 (dic 2025)
- `sentry` 0.47.0 (mar 2026, MSRV 1.88.0)
- `sqlx` 0.8.6 (feb 2026, current stable según referencias 2026; alpha 0.9.0 en desarrollo)
- `pasetors` 0.7.8 (feb 2026)
- `argon2` (RustCrypto) 0.5.3 — pure Rust Argon2id
- `utoipa` 5.5.0 (may 2026)
- `Apalis` 1.0.0-rc.9 (release candidate, estable para uso previsto en ADR 0015)
- Se elimina ambigüedad de `snmp` como crate genérico; se especifica `async-snmp` como dependencia oficial
- Se agregan versiones de referencia para el stack frontend (SvelteKit 2.x, Svelte 5.x, TailwindCSS 4.x, TanStack Query 5.x, ArkType 2.x)

---

# Consecuencias

## ✅ Positivas

* Visibilidad completa de infraestructura regional
* Detección temprana de incidentes
* Inventario centralizado
* Auditoría institucional
* Realtime eficiente (SSE)
* Operación tolerante a conectividad inestable (Local-First)
* Integración natural con el stack Rust/Svelte
* VPS-friendly (bajo consumo RAM)

---

## ⚠️ Negativas / Trade-offs

### Requiere agentes distribuidos

Cada sede necesita algún mecanismo de captura.

→ Mitigación:

* agentes ultra ligeros (Rust, < 10MB RAM)
* buffering offline automático
* despliegue automatizado vía Coolify (ADR 0019)

---

### Métricas históricas pueden crecer rápidamente

El almacenamiento de time-series aumenta con el tiempo.

→ Mitigación:

* agregación histórica automática (Apalis jobs)
* retención automática (30 días crudas, 1 año agregadas)
* particionamiento por rango en PostgreSQL (opcional)
* compactación periódica

---

### Detección de intrusos no es perfecta

ARP/SNMP pueden no detectar todos los casos.

→ Mitigación:

* múltiples fuentes de detección (ARP + SNMP + ICMP + DHCP)
* correlación de eventos
* alertas heurísticas
* whitelist institucional mantenida por admin

---

# Decisiones derivadas

* **SSE es preferido sobre WebSockets** — más simple, compatible con HTTP infraestructura existente, mejor para firewalls institucionales
* **Todos los dispositivos tienen `updated_at` y `deleted_at`** — Soft Delete obligatorio (ADR 0006)
* **El dashboard funciona parcialmente offline** — Local-First via SQLite Wasm + sync queue (ADR 0021)
* **Las métricas críticas se agregan vía Apalis jobs** — no en tiempo real, para no saturar DB
* **RBAC es obligatorio para vistas técnicas** — sin permiso no hay acceso a datos sensibles
* **El sistema debe tolerar sedes sin internet temporalmente** — agentes buffer + sync diferido
* **PostgreSQL es la base de datos principal del backend** — SQLite solo para Local-First en browser (ADR 0004, ADR 0021)
* **El monitoreo está diseñado para VPS pequeños y bajo consumo RAM** — Rust + PostgreSQL optimizado
* **PASETO v4 es el único método de autenticación** — JWT prohibido (ADR 0008)
* **Rate limiting en todos los endpoints de auth** — protección contra brute force (ADR 0009)
* **Soft delete en todas las entidades persistentes** — nunca DELETE físico (ADR 0006)
* **async-snmp 0.12.0** es el crate oficial para recolección SNMP — requiere Rust 1.88+ (satisfecho por Rust 1.95.0)
* **surge-ping 0.8.4** es el crate oficial para ICMP async
* **sqlx 0.8.6** para queries PostgreSQL — verificar compatibilidad con `time` crate (feature `time` activada)
* **sentry 0.47.0** para monitoreo de errores en producción
* **tracing 0.1.44** para observabilidad estructurada en todo el workspace
* **pasetors 0.7.8** para tokens PASETO v4.local
* **argon2 0.5.3** (RustCrypto) para hashing de passwords
* **utoipa 5.5.0** para generación automática de OpenAPI del módulo de monitoreo

---

# Historial de cambios

| Versión | Fecha       | Cambios realizados |
| ------- | ----------- | ------------------ |
| 1.0     | 2026 (orig) | Versión inicial del módulo de monitoreo |
| 2.0     | 2026-05-16  | Revisión general de estructura, adición de Local-First, agentes distribuidos, stack frontend especificado |
| 2.1     | 2026-05-16  | Fija versiones exactas de dependencias: async-snmp 0.12.0, surge-ping 0.8.4, tokio 1.51 (LTS), tracing 0.1.44, sentry 0.47.0, sqlx 0.8.6, pasetors 0.7.8, argon2 0.5.3, utoipa 5.5.0, Apalis 1.0.0-rc.9; especifica crate SNMP oficial; agrega versiones frontend de referencia |
