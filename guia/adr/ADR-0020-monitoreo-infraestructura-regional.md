# ADR 0020 — Módulo de Monitoreo de Infraestructura Regional

| Campo               | Valor                                                                                                                                                  |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                                                                             |
| **Fecha**           | 2026                                                                                                                                                   |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                                                       |
| **Relacionado con** | ADR 0001 (Monolito Modular), ADR 0003 (Axum), ADR 0017 (SvelteKit + ConnectRPC), ADR 0015 (Jobs + Apalis) |

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
Agentes/Sensores Locales
   ↓
API Axum (ADR 0003)
   ↓
Jobs de procesamiento (Apalis)
   ↓
SQLite + métricas históricas
   ↓
Dashboard SvelteKit realtime
```

---

# Componentes del módulo

| Componente  | Responsabilidad                   |
| ----------- | --------------------------------- |
| `inventory` | Inventario físico de dispositivos |
| `topology`  | Mapeo visual de conexiones        |
| `metrics`   | Métricas de red                   |
| `alerts`    | Detección de anomalías            |
| `audit`     | Bitácora y reportes               |
| `agents`    | Recolección distribuida           |
| `sync`      | Sincronización offline            |

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
| IP        | Dirección IP                   |
| MAC       | Dirección física               |
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

* tráfico RX/TX
* uso por secretaría
* saturación WAN
* throughput histórico
* utilización pico
* latencia
* packet loss
* jitter

---

## Estrategia realtime

Por defecto:

* SSE (ADR 0017)
* polling adaptativo
* WebSocket solo si realmente es necesario

---

# 4 — Detección de dispositivos intrusos

## Objetivo

Detectar dispositivos no autorizados dentro de la red institucional.

---

## Estrategia

### Descubrimiento

* ARP scan
* SNMP
* ICMP discovery
* DHCP logs
* MAC learning

---

## Validación

Comparar:

```text
Dispositivo detectado
VS
Whitelist institucional
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
| Dashboard | Tiempo real           |
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
}

pub enum DeviceStatus {
    Active,
    Offline,
    Maintenance,
}

pub struct Device {
    pub id:             DeviceId,
    pub hostname:       String,
    pub ip:             IpAddr,
    pub mac:            MacAddress,
    pub device_type:    DeviceType,
    pub status:         DeviceStatus,
    pub sede_id:        SedeId,
    pub last_seen_at:   OffsetDateTime,
}
```

---

## Métrica

```rust
pub struct MetricReading {
    pub id:            ReadingId,
    pub device_id:     DeviceId,
    pub bandwidth_rx:  i64,
    pub bandwidth_tx:  i64,
    pub latency_ms:    f32,
    pub packet_loss:   f32,
    pub anomaly:       bool,
    pub created_at:    OffsetDateTime,
}
```

---

# Arquitectura frontend

## Stack

| Tecnología     | Uso                 |
| -------------- | ------------------- |
| SvelteKit      | Dashboard SSR       |
| Svelte 5 Runes | Reactividad         |
| LayerChart     | Gráficos            |
| SSE            | Realtime            |
| TailwindCSS    | UI                  |
| ConnectRPC     | Comunicación tipada |

---

# Estrategia realtime

```text
Agente
   ↓
Axum
   ↓
Apalis procesa métricas
   ↓
SSE emite actualización
   ↓
Dashboard actualiza gráficos
```

---

# Estrategia offline

* SQLite Wasm local
* sync queue
* caché de métricas recientes
* dashboards parcialmente offline

---

# Base de datos

## Tablas principales

```text
sedes
devices
device_links
metric_readings
alerts
intrusion_events
audit_logs
network_snapshots
```

---

# Estrategia de almacenamiento

| Tipo de dato      | Estrategia             |
| ----------------- | ---------------------- |
| Inventario        | Persistente            |
| Métricas realtime | Time-series            |
| Alertas           | Persistente            |
| Logs              | Retención configurable |

---

# Retención de métricas

| Tipo               | Retención  |
| ------------------ | ---------- |
| Métricas crudas    | 30 días    |
| Métricas agregadas | 1 año      |
| Alertas            | Permanente |
| Auditoría          | Permanente |

---

# Agentes de monitoreo

## Objetivo

Recolectar métricas desde sedes remotas.

---

## Capacidades

* heartbeat
* métricas SNMP
* escaneo ARP
* monitoreo ICMP
* detección de dispositivos
* buffering offline

---

## Estrategia

```text
Sede sin internet
   ↓
Agente almacena localmente
   ↓
Reconexión
   ↓
Sync automático
```

---

# Seguridad

## Reglas obligatorias

* RBAC institucional
* auditoría obligatoria
* PASETO auth
* rate limiting
* cifrado TLS
* logs firmados
* detección de anomalías

---

# Roles RBAC

| Rol          | Permisos         |
| ------------ | ---------------- |
| SuperAdmin   | Acceso total     |
| Auditor      | Solo lectura     |
| NetworkAdmin | Gestión técnica  |
| Operador     | Monitoreo básico |

---

# Auditoría

Toda acción crítica genera log:

```text
usuario
acción
IP
timestamp
antes/después
```

---

# Reportes

## Exportación

* PDF
* CSV
* JSON
* snapshots históricos

---

## Reportes disponibles

* consumo por secretaría
* disponibilidad mensual
* incidentes críticos
* topología actual
* inventario institucional
* intrusiones detectadas

---

# Integración con Apalis

## Jobs

| Job                     | Función                       |
| ----------------------- | ----------------------------- |
| `MetricsAggregationJob` | Agregación histórica          |
| `IntrusionDetectionJob` | Análisis de anomalías         |
| `AlertDispatchJob`      | Envío de alertas              |
| `CleanupMetricsJob`     | Limpieza de métricas antiguas |

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

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta  | Propósito                 |
| ------------ | ------------------------- |
| `snmp`       | Recolección SNMP          |
| `surge-ping` | ICMP async                |
| `pcap`       | Captura de tráfico        |
| `LayerChart` | Visualización realtime    |
| `netdev`     | Información de interfaces |
| `tokio`      | Concurrencia async        |
| `Apalis`     | Procesamiento background  |
| `SSE`        | Streaming eficiente       |
| `tracing`    | Observabilidad            |
| `sentry`     | Monitoreo de errores      |

---

# Consecuencias

## ✅ Positivas

* Visibilidad completa de infraestructura regional
* Detección temprana de incidentes
* Inventario centralizado
* Auditoría institucional
* Realtime eficiente
* Operación tolerante a conectividad inestable
* Integración natural con el stack Rust/Svelte

---

## ⚠️ Negativas / Trade-offs

### Requiere agentes distribuidos

Cada sede necesita algún mecanismo de captura.

→ Mitigación:

* agentes ultra ligeros
* buffering offline
* despliegue automatizado

---

### Métricas históricas pueden crecer rápidamente

El almacenamiento de time-series aumenta con el tiempo.

→ Mitigación:

* agregación histórica
* retención automática
* compactación periódica

---

### Detección de intrusos no es perfecta

ARP/SNMP pueden no detectar todos los casos.

→ Mitigación:

* múltiples fuentes de detección
* correlación de eventos
* alertas heurísticas

---

# Decisiones derivadas

* SSE es preferido sobre WebSockets
* Todos los dispositivos tienen `updated_at`
* El dashboard funciona parcialmente offline
* Las métricas críticas se agregan vía Apalis
* RBAC es obligatorio para vistas técnicas
* El sistema debe tolerar sedes sin internet temporalmente
* SQLite sigue siendo la base oficial del proyecto
* El monitoreo está diseñado para VPS pequeños y bajo consumo RAM
