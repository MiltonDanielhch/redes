# ADR 0022 — Agentes de Monitoreo Distribuidos

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Versión** | 2.1 (Corrección 2026-05-16) |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0020 (Monitoreo Regional), ADR 0021 (Local-First), ADR 0008 (PASETO), ADR 0015 (Apalis), ADR 0013 (Docker Compose), ADR 0019 (Coolify Deploy) |

---

## Contexto

La Gobernación del Beni tiene múltiples sedes regionales dispersas geográficamente. Cada sede tiene:

* Switches, routers, access points y otros dispositivos de red
* Conectividad inestable hacia el centro de datos central
* Necesidad de monitoreo continuo incluso durante cortes
* Recursos de hardware limitados (mini PCs, Raspberry Pi, o VMs pequeñas)

No es viable ni seguro exponer los dispositivos de red de cada sede directamente a internet. Se necesita un **agente ligero** por sede que actúe como puente seguro entre la red local y la API central.

---

## Decisión

Desplegar un **agente de monitoreo en Rust** (`apps/agent/`) en cada sede regional.

El agente:

1. Escanea la red local (ARP, SNMP, ICMP)
2. Recolecta métricas de dispositivos
3. Detecta dispositivos no autorizados
4. Almacena datos localmente cuando no hay conectividad
5. Sincroniza con la API central cuando hay conectividad
6. Opera con mínimo consumo de recursos

---

## Stack del Agente

| Componente | Tecnología | Versión fijada | Justificación |
|------------|-----------|----------------|---------------|
| Lenguaje | Rust (Edition 2024) | 1.95.0 | Seguridad, performance, binario único |
| DB local | SQLite (sqlx) | `0.8.6` | Ligero, sin servidor, sync offline (ADR 0021) |
| HTTP client | reqwest | `0.13.2` | Estándar en ecosistema Rust |
| Async runtime | tokio | `1.52.3` | Consistencia con backend (ADR 0003) |
| Config | toml + env | `0.8.22` | Simple, legible, fail-fast (ADR 0002) |
| Logging | tracing + tracing-subscriber | `0.1.44` / `0.3.23` | Observabilidad consistente con API |
| SNMP | snmp2 | `0.5.0` | Recolección de métricas de dispositivos |
| ICMP | surge-ping | `0.8.4` | Heartbeat y detección de dispositivos |
| Sync | crates/sync/ (Rust) | workspace | Reutiliza lógica de sync con API central |
| Auth | pasetors | `0.7.8` | Consistencia con stack (ADR 0008) |

**Nota (2026-05-16):** El agente reutiliza `crates/sync/` (Rust) para la lógica de sincronización offline, compartiendo patrones con el frontend (ADR 0021). Sin embargo, `crates/sync/` está diseñado para el **agente** (Rust), no para el frontend (TypeScript/SvelteKit). El frontend usa `apps/web/src/lib/sync/` (TypeScript).

**Cambios respecto a v2.0:**
- `reqwest` actualizado a `0.13.2` (latest estable, 2025-12-15). Soporta HTTP/3 (h3), hyper-rustls, y mejora de performance sobre 0.12.x citeweb_search:16#19
- `tokio` fijado a `1.52.3` (latest estable, 2026-05-08). LTS release hasta marzo 2027, MSRV 1.70 citeweb_search:16#1
- `sqlx` fijado a `0.8.6` (current stable, 2025-05-19). Alpha 0.9.0 en desarrollo con soporte `smol` citeweb_search:16#14web_search:16#15
- `tracing` fijado a `0.1.44` (dic 2025), `tracing-subscriber` a `0.3.23` (mar 2026). MSRV 1.65 citeweb_search:16#5web_search:16#8
- `surge-ping` fijado a `0.8.4` (dic 2025). ICMP async con tokio citeweb_search:17#18
- `snmp2` (fork de snmp original) fijado a `0.5.0` (mar 2026). SNMP v1/v2/v3 sync/async con traps y MIB support. Reemplaza `async-snmp` (pre-1.0, breaking changes probables) y `snmp` (abandonado, 0.2.2 de 2017) citeweb_search:17#22
- `pasetors` fijado a `0.7.8` (feb 2026). MSRV 1.88.0, compatible con Rust 1.95.0 citeweb_search:17#7web_search:17#11
- `toml` fijado a `0.8.22` (abr 2026). Native Rust encoder/decoder TOML 1.1.0 spec citeweb_search:17#38
- Se elimina ambigüedad de `snmp` genérico — se especifica `snmp2` como crate oficial

---

## Arquitectura del Agente

```text
┌─────────────────────────────────────────────────────────────┐
│  Sede Regional                                              │
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │  Switch     │◄──►│  Router     │◄──►│  APs        │    │
│  │  (SNMP)     │    │  (SNMP)     │    │  (SNMP)     │    │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘    │
│         │                  │                  │            │
│         └──────────────────┼──────────────────┘            │
│                            │                               │
│                            ▼                               │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  Agente Rust (apps/agent)                           │  │
│  │  ├─ SNMP poller (cada 60s)                         │  │
│  │  ├─ ICMP discovery (cada 300s)                      │  │
│  │  ├─ ARP scan (cada 600s)                          │  │
│  │  ├─ SQLite local (buffering)                        │  │
│  │  ├─ Sync engine (crates/sync/)                      │  │
│  │  │   ├─ detect_connectivity()                     │  │
│  │  │   ├─ enqueue_operation()                        │  │
│  │  │   ├─ process_queue()                           │  │
│  │  │   └─ resolve_conflict()                         │  │
│  │  └─ HTTP client (reqwest 0.13.2)                  │  │
│  └────────────────────────┬────────────────────────────┘  │
│                           │                                │
│                           │ HTTPS (cuando hay internet)   │
│                           ▼                                │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  API Central (apps/api)                             │  │
│  │  ├─ POST /api/v1/agent/heartbeat                   │  │
│  │  ├─ POST /api/v1/agent/metrics                     │  │
│  │  ├─ POST /api/v1/agent/devices                     │  │
│  │  ├─ POST /api/v1/agent/intrusions                  │  │
│  │  ├─ GET  /api/v1/agent/config                      │  │
│  │  └─ POST /api/v1/sync/push (reutilizado desde ADR 0021) │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Comunicación Agente ↔ API

### Protocolo

* **Transporte:** HTTPS REST (no gRPC, no WebSocket — coherencia con ADR 0003, ADR 0016)
* **Serialización:** JSON
* **Auth:** Token de agente (PASETO v4 Local, scope `agent`)
* **Compresión:** gzip para payloads grandes (métricas batch)

### Auth del Agente

El agente usa **PASETO v4 Local** con un token dedicado, generado desde el dashboard administrativo:

```rust
// crates/auth/src/agent_token.rs (nuevo)
use pasetors::local::LocalToken;

pub struct AgentTokenService;

impl AgentTokenService {
    pub fn generate_agent_token(agent_id: &str, sede_id: &str, secret: &[u8]) -> String {
        let claims = Claims::new()
            .with_subject(agent_id)
            .with_custom_claim("sede_id", sede_id)
            .with_custom_claim("scope", "agent")
            .with_expiration(Duration::days(365).into());

        LocalToken::encrypt(secret, &claims, None, None)
            .expect("valid token")
    }

    pub fn verify_agent_token(token: &str, secret: &[u8]) -> Result<AgentClaims, Error> {
        let claims = LocalToken::decrypt(secret, token, None)?;

        let scope = claims.custom_claim("scope")
            .and_then(|v| v.as_str())
            .ok_or(Error::InvalidClaims)?;

        if scope != "agent" {
            return Err(Error::InvalidScope);
        }

        Ok(AgentClaims {
            agent_id: claims.subject().unwrap_or_default().to_string(),
            sede_id: claims.custom_claim("sede_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        })
    }
}
```

**Nota crítica:** El token de agente tiene **scope `agent`** y **expiración de 365 días** (no 15 minutos como los tokens de usuario). Esto evita que el agente necesite refresh frecuente en sedes con conectividad intermitente. La revocación se maneja vía **lista de tokens revocados en PostgreSQL** o **rotación manual desde dashboard**.

### Endpoints del Agente

```
POST /api/v1/agent/heartbeat
  Headers: Authorization: Bearer v4.local.xxx (agent token)
  Body: {
    agent_id: string,           // ej: "agent-sede-trinidad-01"
    sede_id: string,            // ej: "sede-trinidad"
    timestamp: number,          // epoch ms UTC
    uptime_seconds: number,
    version: string,            // ej: "0.1.0"
    local_db_size_bytes: number,
    pending_sync_count: number,
    system_metrics: {
      cpu_percent: number,
      memory_mb: number,
      disk_free_mb: number
    }
  }
  Response: { 
    status: "ok", 
    commands: AgentCommand[],
    config_hash: string  // hash de config actual, si cambió → agente pide nueva config
  }

POST /api/v1/agent/metrics
  Headers: Authorization: Bearer v4.local.xxx (agent token)
  Body: {
    agent_id: string,
    sede_id: string,
    readings: MetricReading[]   // batch de métricas recolectadas
  }
  Response: { 
    accepted: number, 
    rejected: number,
    rejected_ids: string[]     // IDs de métricas rechazadas (device no existe, formato inválido)
  }

POST /api/v1/agent/devices
  Headers: Authorization: Bearer v4.local.xxx (agent token)
  Body: {
    agent_id: string,
    sede_id: string,
    devices: DeviceDiscovery[]  // dispositivos descubiertos vía SNMP/ARP/ICMP
  }
  Response: { 
    accepted: number, 
    new_devices: number,       // dispositivos nuevos no en inventario
    updated: number,           // dispositivos existentes actualizados (last_seen_at)
    unknown_devices: number    // dispositivos no en whitelist (posibles intrusiones)
  }

POST /api/v1/agent/intrusions
  Headers: Authorization: Bearer v4.local.xxx (agent token)
  Body: {
    agent_id: string,
    sede_id: string,
    events: IntrusionEvent[]   // eventos de intrusión detectados
  }
  Response: { 
    accepted: number,
    duplicates: number        // eventos ya reportados previamente
  }

GET /api/v1/agent/config
  Headers: Authorization: Bearer v4.local.xxx (agent token)
  Response: {
    agent_id: string,
    sede_id: string,
    snmp_interval_seconds: number,      // default: 60
    icmp_interval_seconds: number,      // default: 300
    arp_interval_seconds: number,         // default: 600
    heartbeat_interval_seconds: number,  // default: 60
    metrics_batch_size: number,          // default: 100
    max_sync_retry: number,              // default: 5
    sync_retry_backoff_seconds: number[], // [1, 2, 5, 10, 30]
    whitelist_macs: string[],            // MACs conocidas (evita falsos positivos)
    discovery_ranges: string[],          // rangos IP para scan (ej: ["192.168.1.0/24"])
    snmp_community: string,              // default: "public"
    snmp_timeout_ms: number,              // default: 5000
    icmp_timeout_ms: number,            // default: 3000
    enabled_collectors: string[]         // ["snmp", "icmp", "arp"]
  }
```

**Nota:** Los endpoints `/api/v1/agent/*` son **separados** de los endpoints de sync general (`/api/v1/sync/*` definidos en ADR 0021). Los endpoints de agente manejan la **recolecta de métricas y descubrimiento**, mientras que `/api/v1/sync/push` maneja la **sincronización de operaciones del frontend** (acknowledge alert, resolve intrusion). El agente puede usar ambos: `/agent/*` para métricas y `/sync/push` para operaciones administrativas si es necesario.

### AgentCommand (comandos desde API)

```rust
// crates/domain/src/entities/agent_command.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentCommand {
    Reconfigure { 
        config: AgentConfig,
        config_hash: String 
    },
    Restart,
    UpdateWhitelist { 
        macs: Vec<String>,
        append: bool  // true = agregar, false = reemplazar
    },
    ForceSync,
    Shutdown { 
        delay_seconds: Option<u64>,
        reason: Option<String>
    },
    UpdateVersion {
        version: String,
        download_url: String,
        checksum: String
    },
}
```

---

## Ciclo de Vida del Agente

```text
┌─────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Start  │────►│  Load Config │────►│  Validate   │────►│  Register   │
│         │     │  (fail-fast) │     │  Token      │     │  with API   │
└─────────┘     └─────────────┘     └─────────────┘     └──────┬──────┘
                                                                │
                                                                ▼
┌─────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Sync   │◄────│  Store      │◄────│  Collect    │◄────│  Main Loop  │
│  (push) │     │  (SQLite)   │     │  (SNMP/ICMP)│     │  (tokio)    │
└────┬────┘     └─────────────┘     └─────────────┘     └─────────────┘
     │
     │ online?
     ├── Sí ──► POST /agent/metrics ──► API
     │          (batch de hasta N métricas)
     │
     └── No ──► Queue en SQLite ──► Retry con backoff
                (sync_queue de crates/sync/)
```

### Estados del Agente

| Estado | Descripción | Transición |
|--------|-------------|------------|
| `Initializing` | Carga config, valida token | → `Registering` |
| `Registering` | Primer heartbeat con API | → `Online` o `Offline` |
| `Online` | Conectado, recolectando y enviando | → `Offline` si heartbeat falla |
| `Offline` | Sin conectividad, buffering en SQLite | → `Online` si heartbeat exitoso |
| `Syncing` | Enviando batch pendiente | → `Online` si éxito, `Offline` si falla |
| `Error` | Error crítico (token inválido, config rota) | → `Shutdown` o manual fix |
| `Shutdown` | Apagado controlado | → `Initializing` en restart |

---

## Configuración del Agente

### Variables de Entorno (requeridas)

```bash
# Identidad
AGENT_ID=agent-sede-trinidad-01
SEDE_ID=sede-trinidad

# API Central
API_URL=https://api.redes.gob.bo
AGENT_TOKEN=v4.local.xxxxxx

# Logging
RUST_LOG=info
RUST_LOG_FORMAT=json  # json o pretty

# Paths
DATA_DIR=/var/lib/redes-agent/data
CONFIG_PATH=/etc/redes-agent/agent.toml
```

### Config File (agent.toml) — Opcional, overridea defaults

```toml
[agent]
id = "agent-sede-trinidad-01"
sede_id = "sede-trinidad"
version = "0.1.0"
data_dir = "/var/lib/redes-agent/data"

[api]
url = "https://api.redes.gob.bo"
token = "v4.local.xxxxxx"
heartbeat_interval_seconds = 60
connection_timeout_seconds = 30

[collectors.snmp]
enabled = true
community = "public"
timeout_ms = 5000
interval_seconds = 60
oids = [
    "1.3.6.1.2.1.1.1.0",    # sysDescr
    "1.3.6.1.2.1.2.2.1.10.1", # ifInOctets (interface 1)
    "1.3.6.1.2.1.2.2.1.16.1", # ifOutOctets (interface 1)
    "1.3.6.1.2.1.2.2.1.5.1",  # ifSpeed (interface 1)
]

[collectors.icmp]
enabled = true
timeout_ms = 3000
interval_seconds = 300
targets = ["192.168.1.1", "192.168.1.254", "8.8.8.8"]

[collectors.arp]
enabled = true
interface = "eth0"
interval_seconds = 600

[sync]
batch_size = 100
max_retry = 5
retry_backoff_seconds = [1, 2, 5, 10, 30]
compression = true  # gzip payloads > 10KB

[discovery]
ranges = ["192.168.1.0/24", "192.168.2.0/24"]
whitelist_macs = [
    "00:11:22:33:44:55",  # Switch principal
    "00:11:22:33:44:66",  # Router gateway
]
```

---

## Seguridad del Agente

### Token de Agente

* **Scope dedicado:** `agent` (no `user`, no `admin`)
* **Generación:** Dashboard admin → POST /admin/agents/{id}/rotate-token
* **Almacenamiento:** Archivo `agent.toml` con permisos 600, o variable de entorno
* **Expiración:** 365 días (rotación manual recomendada cada 90 días)
* **Revocación:** Inmediata vía dashboard (marca `revoked_at` en tabla `agent_tokens`)
* **Verificación:** El middleware de auth verifica `scope == "agent"` y que el token no esté en lista de revocados

### Comunicación

* **TLS 1.3 obligatorio** — rechaza conexiones HTTP plano
* **Certificate pinning (opcional Fase 2):** Hash del certificado CA o leaf embedido en el binario
* **No expone puertos entrantes:** Solo outbound HTTPS (no necesita firewall inbound)
* **Rate limiting:** El agente respeta `X-RateLimit-Remaining` y reduce frecuencia si recibe 429

### Aislamiento

* El agente corre en red local, no expuesto a internet directamente
* No tiene acceso a base de datos central (solo API REST)
* Solo envía métricas y descubrimientos, no recibe comandos arbitrarios de ejecución
* Los comandos `Reconfigure`, `UpdateWhitelist`, `UpdateVersion` son **declarativos**, no ejecutan código arbitrario
* Corre bajo usuario dedicado (`redes-agent`) sin privilegios root

---

## Esquema SQLite del Agente

```sql
-- Métricas locales (buffering para sync offline)
CREATE TABLE metric_readings_local (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_ip TEXT NOT NULL,
    device_id TEXT,              -- NULL hasta que se mapee a device conocido
    bandwidth_rx BIGINT,
    bandwidth_tx BIGINT,
    latency_ms REAL,
    packet_loss_percent REAL,
    anomaly_detected BOOLEAN DEFAULT FALSE,
    collected_at INTEGER NOT NULL, -- epoch ms
    synced_at INTEGER,           -- epoch ms, NULL si pendiente
    sync_attempts INTEGER DEFAULT 0,
    api_response TEXT             -- JSON con respuesta de API si fue rechazada
);

CREATE INDEX idx_metrics_synced ON metric_readings_local(synced_at) WHERE synced_at IS NULL;
CREATE INDEX idx_metrics_collected ON metric_readings_local(collected_at);

-- Dispositivos descubiertos (ARP scan, SNMP discovery, ICMP sweep)
CREATE TABLE discovered_devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ip TEXT NOT NULL,
    mac TEXT,
    hostname TEXT,
    vendor TEXT,                 -- OUI lookup o sysDescr
    device_type TEXT,            -- inferred: router, switch, ap, unknown
    first_seen_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    status TEXT DEFAULT 'unknown' CHECK(status IN ('unknown', 'active', 'offline', 'new', 'intrusion')),
    synced BOOLEAN DEFAULT FALSE,
    is_whitelisted BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_discovered_mac ON discovered_devices(mac);
CREATE INDEX idx_discovered_status ON discovered_devices(status) WHERE status = 'new' OR status = 'intrusion';

-- Eventos de intrusión detectados localmente
CREATE TABLE intrusion_events_local (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mac TEXT NOT NULL,
    ip TEXT,
    detected_at INTEGER NOT NULL,
    status TEXT DEFAULT 'detected' CHECK(status IN ('detected', 'reported', 'false_positive', 'resolved')),
    synced BOOLEAN DEFAULT FALSE,
    notes TEXT
);

CREATE INDEX idx_intrusions_synced ON intrusion_events_local(synced_at) WHERE synced_at IS NULL;

-- Heartbeat log (para diagnóstico de conectividad)
CREATE TABLE heartbeat_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sent_at INTEGER NOT NULL,
    response_status INTEGER,     -- HTTP status o NULL si timeout
    api_reachable BOOLEAN,
    commands_received INTEGER DEFAULT 0,
    error_message TEXT
);

CREATE INDEX idx_heartbeat_sent ON heartbeat_log(sent_at DESC);

-- Config cache (última config recibida de API)
CREATE TABLE config_cache (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    fetched_at INTEGER NOT NULL,
    hash TEXT NOT NULL           -- SHA-256 para comparar con API
);

-- Sync queue (operaciones administrativas del agente, no métricas)
-- Las métricas van a metric_readings_local, las operaciones admin van aquí
CREATE TABLE sync_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    op_type TEXT NOT NULL CHECK(op_type IN ('acknowledge_alert', 'resolve_intrusion', 'update_device_status')),
    payload TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    synced_at INTEGER,
    retry_count INTEGER DEFAULT 0
);
```

**Nota:** Las tablas del agente son **independientes** de las tablas del frontend (`apps/web/src/lib/sync/`). El agente usa SQLite nativo vía `sqlx` (Rust), no WebAssembly. La estructura está optimizada para buffering de métricas y descubrimiento de dispositivos.

---

## Despliegue del Agente

### Métodos soportados

| Método | Caso de uso | Complejidad | Requisitos |
|--------|-------------|-------------|------------|
| Docker container | Sede con Docker disponible | Baja | Docker, docker-compose |
| Systemd service | Linux bare metal (mini PC, Raspberry Pi) | Media | systemd, curl/wget |
| Binary directo | Dispositivos embebidos, ARM | Baja | curl, chmod |
| Coolify (remoto) | Despliegue centralizado de múltiples agentes | Media | Coolify instance, VPN |

### Containerfile del Agente

```dockerfile
# Build stage
FROM rust:1.95-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev sqlite-dev
WORKDIR /app
COPY . .
RUN cargo build --release --bin agent --profile release-size

# Runtime stage — distroless ligero
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/agent /redes-agent
# SQLite runtime libs ya están en distroless cc
VOLUME ["/var/lib/redes-agent/data"]
ENV DATA_DIR=/var/lib/redes-agent/data
ENTRYPOINT ["/redes-agent"]
```

**Nota:** Se usa el **perfil `release-size`** definido en `Cargo.toml` workspace (ADR Génesis) para binario mínimo (~5-10MB). Rust 1.95.0 es la última estable al 16 de abril de 2026 (ADR 0019 v2.1).

### Script de instalación (systemd)

```bash
#!/bin/bash
# install-agent.sh
set -euo pipefail

AGENT_VERSION="${AGENT_VERSION:-0.1.0}"
ARCH=$(uname -m)
INSTALL_DIR="/usr/local/bin"
DATA_DIR="/var/lib/redes-agent/data"
CONFIG_DIR="/etc/redes-agent"

# Detectar arquitectura
 case "$ARCH" in
    x86_64)  ARCH_TAG="amd64" ;;
    aarch64) ARCH_TAG="arm64" ;;
    armv7l)  ARCH_TAG="armv7"  ;;
    *)       echo "Arquitectura no soportada: $ARCH"; exit 1 ;;
esac

echo "Instalando redes-agent v${AGENT_VERSION} para ${ARCH_TAG}..."

# Descargar binario
curl -fsSL -o "${INSTALL_DIR}/redes-agent"   "https://releases.redes.gob.bo/agent/${AGENT_VERSION}/redes-agent-linux-${ARCH_TAG}"

chmod +x "${INSTALL_DIR}/redes-agent"

# Crear usuario dedicado (sin shell, sin login)
if ! id -u redes-agent >/dev/null 2>&1; then
    useradd -r -s /bin/false -d "${DATA_DIR}" redes-agent
fi

# Directorios
mkdir -p "${DATA_DIR}" "${CONFIG_DIR}"
chown -R redes-agent:redes-agent "${DATA_DIR}"
chmod 750 "${DATA_DIR}"

# Config (el admin debe editar después)
cat > "${CONFIG_DIR}/agent.toml.example" <<'EOF'
[agent]
id = "agent-sede-EJEMPLO-01"
sede_id = "sede-EJEMPLO"
version = "0.1.0"

[api]
url = "https://api.redes.gob.bo"
token = "v4.local.REEMPLAZAR_CON_TOKEN_REAL"
EOF

chmod 640 "${CONFIG_DIR}/agent.toml.example"

# Systemd service
cat > /etc/systemd/system/redes-agent.service <<'EOF'
[Unit]
Description=Redes Beni — Agente de Monitoreo
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=redes-agent
Group=redes-agent
ExecStart=/usr/local/bin/redes-agent
Restart=always
RestartSec=5
Environment="RUST_LOG=info"
Environment="DATA_DIR=/var/lib/redes-agent/data"
EnvironmentFile=-/etc/redes-agent/agent.env

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/redes-agent/data
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable redes-agent

echo "Instalación completa."
echo "PASOS MANUALES:"
echo "1. Copiar config: sudo cp ${CONFIG_DIR}/agent.toml.example ${CONFIG_DIR}/agent.toml"
echo "2. Editar token: sudo nano ${CONFIG_DIR}/agent.toml"
echo "3. Iniciar: sudo systemctl start redes-agent"
echo "4. Ver logs: sudo journalctl -u redes-agent -f"
```

---

## Consecuencias

### ✅ Positivas

* Cobertura de monitoreo en **todas las sedes** regionales
* Operación continua durante cortes de internet (buffering SQLite + sync offline)
* Seguridad: agente aislado, token dedicado con scope `agent`, sin acceso a DB central
* Bajo consumo de recursos: **< 50MB RAM**, **< 20MB disco** (binario + SQLite)
* Binario único estático, fácil de desplegar en cualquier arquitectura (x86_64, ARM64, ARMv7)
* No requiere infraestructura adicional en sede (solo el agente + red local)
* Detección de intrusiones en red local (ARP scan, MAC no en whitelist)
* Métricas de red recolectadas directamente desde switches/routers vía SNMP

### ⚠️ Trade-offs

* Necesita **despliegue físico o remoto en cada sede** (N agentes = N sedes)
* **Mantenimiento distribuido:** actualización de versión en múltiples agentes
* **Versionado y actualización distribuida:** requiere mecanismo OTA (Over-The-Air) o reinstalación
* **Configuración por sede:** cada agente necesita `sede_id`, rangos IP, community SNMP
* **Dependencia de red local:** si la red local del agente falla, no puede recolectar métricas
* **Complejidad operativa:** monitoreo del estado de los agentes (heartbeat, versión, db size)
* **Seguridad física:** el dispositivo del agente en sede debe ser físicamente seguro

---

## Decisiones derivadas

* El agente es **Rust** (coherencia con stack backend, binario único, seguridad)
* **SQLite local** para buffering offline (coherencia con ADR 0021, pero implementación Rust nativa)
* **Token de agente separado** del token de usuario — scope `agent`, expiración 365 días, rotación manual
* **Comunicación REST/JSON** (no gRPC, no WebSocket) — coherencia con ADR 0003, ADR 0016
* **Heartbeat cada 60s** como señal de vida + canal de comandos
* **SNMP community configurable por sede** (algunas sedes usan community no-default)
* **No se implementa agente en Go/Python** — mantener stack uniforme Rust
* **El agente no procesa alertas** — solo recolecta métricas y detecta intrusiones; las alertas se generan en el servidor (jobs Apalis, ADR 0015)
* **Los comandos desde API son declarativos** (config, whitelist, restart) — nunca ejecutan código arbitrario
* **El agente usa `crates/sync/` (Rust)** para lógica de sync offline, compartiendo patrones con el backend
* **El frontend usa `apps/web/src/lib/sync/` (TypeScript)** para sync offline — implementaciones separadas, estrategia compartida (ADR 0021)
* **Perfil `release-size`** en `Cargo.toml` para binario mínimo del agente
* **Containerfile distroless** para deploy Docker del agente (Rust 1.95.0, ADR 0019 v2.1)
* **Systemd hardening** (ProtectSystem, NoNewPrivileges) para instalación bare metal
* **El dashboard admin muestra estado de agentes** (heartbeat, versión, pending sync, db size) — requiere endpoints `GET /api/v1/agents` (ADR 0020, ROADMAP-BACKEND.md Bloque VII)
* **La API central debe exponer endpoints `/api/v1/agent/*`** separados de `/api/v1/sync/*` — los primeros para métricas, los segundos para operaciones frontend
* **Tabla `agent_tokens` en PostgreSQL** para revocación y rotación de tokens
* **snmp2 0.5.0** es el crate oficial para SNMP — soporta v1/v2/v3, sync/async, traps, MIBs (con feature `mibs`)
* **surge-ping 0.8.4** es el crate oficial para ICMP async
* **reqwest 0.13.2** como HTTP client — soporta HTTP/3, hyper-rustls, gzip automático
* **tokio 1.52.3** como async runtime — LTS hasta marzo 2027
* **sqlx 0.8.6** para SQLite local — async, compile-time checked queries
* **pasetors 0.7.8** para tokens PASETO v4.local — MSRV 1.88.0, compatible con Rust 1.95.0
* **tracing 0.1.44** + **tracing-subscriber 0.3.23** para observabilidad estructurada
* **toml 0.8.22** para parsing de configuración TOML

---

## Historial de cambios

| Versión | Fecha       | Cambios realizados |
| ------- | ----------- | ------------------ |
| 1.0     | 2026 (orig) | Versión inicial con endpoints genéricos, sin scope `agent` en PASETO, sin tabla `agent_tokens`, sin distinguir `/agent/*` vs `/sync/*`, sin Containerfile, sin systemd hardening, sin índices SQLite, sin `release-size` profile |
| 2.0     | 2026-05-16  | Agrega scope `agent` dedicado en PASETO con verificación explícita; agrega tabla `agent_tokens` y revocación; separa endpoints `/api/v1/agent/*` de `/api/v1/sync/*`; agrega Containerfile distroless con `release-size`; agrega script de instalación systemd con hardening; agrega índices SQLite y constraints CHECK; agrega campos `system_metrics` en heartbeat; agrega `UpdateVersion` en AgentCommand; agrega `discovery_ranges` en config; documenta uso de `crates/sync/` (Rust) vs `apps/web/src/lib/sync/` (TS); agrega decisiones derivadas de dashboard admin, tabla agent_tokens, y OTA |
| 2.1     | 2026-05-16  | Fija versiones exactas de todas las dependencias: reqwest `0.13.2`, tokio `1.52.3`, sqlx `0.8.6`, tracing `0.1.44`, tracing-subscriber `0.3.23`, surge-ping `0.8.4`, snmp2 `0.5.0`, pasetors `0.7.8`, toml `0.8.22`; reemplaza `snmp` genérico/ambiguo por `snmp2` como crate oficial; actualiza Containerfile a Rust 1.95.0; actualiza decisiones derivadas con versiones específicas |
