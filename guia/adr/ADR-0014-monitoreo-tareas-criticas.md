# ADR 0014 — Monitoreo: Healthchecks.io + Patrón Dead Man's Switch

| Campo               | Valor                                                                       |
| ------------------- | --------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                  |
| **Fecha**           | 2026-05-16                                                                   |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                            |
| **Relacionado con** | ADR 0004 (PostgreSQL backups), ADR 0015 (Apalis jobs), ADR 0013 (Docker Compose), ADR 0020 (Monitoreo Regional) |
| **Última revisión** | 2026-05-16 — Corrección: PostgreSQL backup (no Litestream) + Sentry postergado |

---

## Contexto

En un VPS de $5 sin equipo de monitoreo 24/7, necesitamos saber si una tarea crítica dejó
de correr — antes de que el problema se vuelva un desastre.

El monitoreo tradicional (un agente que pregunta "¿estás vivo?") tiene un punto ciego fatal:
si el VPS se apaga o pierde internet, el agente interno tampoco puede avisar.

Necesitamos:

* **Alertas si el VPS muere completamente** — no solo si un proceso falla internamente
* **Costo $0** — capa gratuita suficiente para el inicio del proyecto
* **Impacto en RAM/CPU: 0** — no instalar nada en el servidor

---

## Decisión

Usar **Healthchecks.io** con el patrón de **"Dead Man's Switch"** (interruptor del hombre muerto):
el servidor avisa que completó una tarea; si el aviso no llega, Healthchecks asume que algo falló y alerta.

---

## 1 — Patrón de monitoreo: "si no llega el ping, algo murió"

```bash
# Ping SOLO si la tarea terminó correctamente
./tarea_critica.sh && curl -fsS -m 10 --retry 5 https://hc-ping.com/{uuid}

# Si la tarea falla:
# → no hay ping
# → Healthchecks alerta automáticamente
```

## Ventajas del patrón

* Detecta caída completa del VPS
* Detecta cron jobs detenidos
* Detecta workers congelados
* No requiere agentes residentes
* RAM ≈ 0
* CPU ≈ 0

---

## 2 — Monitoreo de backups PostgreSQL

**Actualizado:** El proyecto usa PostgreSQL (ADR 0004), no SQLite/Litestream.

Verifica que exista un backup actualizado (via `pg_dump` o herramienta nativa).

```bash
# Ejecutar cada hora
# Verificar que el último backup no tenga más de 25 horas
find /backups/redes -name "*.sql.gz" -mtime -1 | grep -q .   && curl -fsS -m 10 https://hc-ping.com/${HC_POSTGRES_BACKUP_UUID}
```

## Qué detecta

* Backup detenido
* Backups corruptos (archivo vacío o incompleto)
* Problemas de acceso a storage
* VPS apagado
* Cron detenido

**Nota:** Para backups automatizados con pgBackRest o similar, adaptar el script de verificación.

---

## 3 — Heartbeat del worker Apalis

Relacionado con ADR 0015.

```rust
//! Ubicación: `crates/monitoring/src/healthchecks.rs`
//!
//! Descripción: Wrapper para pings a Healthchecks.io con tracing integration.
//!              Usa reqwest 0.13 con timeout agresivo.
//!
//! ADRs: 0014, 0015

use std::time::Duration;
use tracing::{info, warn};

pub async fn ping(hc_url: &str, job_name: &str) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("failed to build reqwest client");

    match client.get(hc_url).send().await {
        Ok(response) if response.status().is_success() => {
            info!(
                job = %job_name,
                status = %response.status(),
                "healthcheck ping succeeded"
            );
        }
        Ok(response) => {
            warn!(
                job = %job_name,
                status = %response.status(),
                "healthcheck ping returned non-success"
            );
        }
        Err(error) => {
            warn!(
                job = %job_name,
                error = ?error,
                "healthcheck ping failed"
            );
        }
    }
}

/// Ping con /start para jobs de larga duración
pub async fn ping_start(hc_url: &str, job_name: &str) {
    let start_url = format!("{}/start", hc_url);
    ping(&start_url, &format!("{}_start", job_name)).await;
}

/// Ping con /fail para reportar fallo explícito
pub async fn ping_fail(hc_url: &str, job_name: &str) {
    let fail_url = format!("{}/fail", hc_url);
    ping(&fail_url, &format!("{}_fail", job_name)).await;
}
```

## Estrategia

* Ping cada 5 minutos para workers
* Usar `/start` al iniciar job largo + `/` (success) al terminar
* Usar `/fail` si el job detecta error interno pero aún puede hacer HTTP
* Timeout agresivo (5s)
* Error no bloqueante
* Si el worker deja de procesar → no hay ping → alerta inmediata

---

## 4 — Verificación de TLS

Relacionado con ADR 0013 (Docker Compose + Coolify).

Coolify gestiona TLS automáticamente via Caddy interno, pero verificación manual como respaldo:

```bash
# Verifica que el certificado no expire en menos de 30 días
openssl s_client -connect tudominio.com:443 2>/dev/null   | openssl x509 -noout -checkend 2592000   && curl -fsS https://hc-ping.com/${HC_TLS_UUID}
```

## Beneficio

Evita caídas por certificados expirados (fallback si Coolify falla).

---

## 5 — Monitoreo de deploys

```makefile
# justfile (alineado con ADR 0012, ADR 0019)

deploy:
    just quality
    just test-all
    just build
    # Deploy via Coolify (webhook o dashboard)
    curl -fsS ${HC_DEPLOY_UUID:+https://hc-ping.com/$HC_DEPLOY_UUID} || true
```

## Resultado

Healthchecks mantiene historial automático de deploys exitosos.

---

## Configuración recomendada

| Check | Intervalo | Grace Period | Severidad |
| ----------------- | --------- | ------------ | ----------- |
| PostgreSQL backup | 1h | 15 min | Alta |
| Worker Apalis | 5 min | 2 min | Crítica |
| TLS | 24h | 2h | Media |
| Deploy | Manual | — | Informativa |

---

## Variables de entorno

```bash
# .env.example

HC_API_KEY=              # Opcional — para API de gestión de checks
HC_POSTGRES_BACKUP_UUID=
HC_DEPLOY_UUID=
HC_TLS_UUID=
HC_WORKER_UUID=
```

Todas son opcionales.

El sistema debe funcionar incluso si no están configuradas.

---

## Crate de monitoreo

```text
crates/
└── monitoring/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── healthchecks.rs   # Wrapper de pings + tracing
```

---

## Dependencias

```toml
# crates/monitoring/Cargo.toml

[dependencies]
reqwest = { version = "0.13", features = ["json"], default-features = false }
tokio = { version = "1.45", features = ["time"] }
tracing = "0.1"
```

**Nota:** `reqwest 0.13` usa `rustls` como TLS backend por defecto. Si se necesita `native-tls`, usar feature explícita.

---

## Arquitectura final

```text
┌─────────────────────────────┐
│         VPS / Coolify       │
│                             │
│  ┌──────────────────────┐   │
│  │     Rust API         │───┼──── ping ───▶ Healthchecks.io
│  ├──────────────────────┤   │
│  │     Apalis Worker    │───┤
│  ├──────────────────────┤   │
│  │   PostgreSQL Backup │───┤
│  └──────────────────────┘   │
│                             │
└─────────────────────────────┘

Si NO llega el ping:
→ email
→ Telegram
→ Slack
→ WhatsApp
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
| ------------------------- | ----------------------------- |
| UptimeRobot | Solo monitorea HTTP público |
| Prometheus + Alertmanager | Muy pesado para 1GB RAM |
| Scripts propios | Mismo punto ciego del VPS |
| Better Uptime | Menor capa gratuita |
| Grafana stack | Complejidad excesiva para MVP |
| Sentry Crons | Requiere Rust 1.88+ (postergado) |

---

## Herramientas y Librerías (Edición 2026)

| Herramienta | Propósito | Versión | Estado |
| ---------------- | -------------------------------------------------- | -------- | ------ |
| `reqwest` | Cliente HTTP robusto con timeout y retries | 0.13.2 | ✅ Activa |
| `tracing` | Observabilidad estructurada | workspace | ✅ Activa |
| Sentry Crons | Integrar errores + cron monitoring en un dashboard | 0.48.2 | ⏳ Requiere Rust 1.88+ |
| Telegram Bot API | Alertas push inmediatas | — | ✅ Activa |

---

## Consecuencias

### ✅ Positivas

* Costo ≈ $0 (capa gratuita: 20 checks)
* RAM ≈ 0
* CPU ≈ 0
* Detecta caída total del VPS
* Sin agentes residentes
* Integración extremadamente simple
* Historial automático de tareas y deploys

### ⚠️ Negativas / Trade-offs

**Dependencia externa**

Si Healthchecks.io cae, no habrá alertas.

**Mitigación**

* Configurar checks duplicados críticos
* UptimeRobot o Better Stack como backup
* Mantener logs locales estructurados

**No reemplaza observabilidad completa**

Healthchecks dice:

> "algo dejó de correr"

pero NO explica:

> "por qué falló"

**Mitigación**

Combinar con:

* tracing JSON
* request_id
* logs estructurados
* Sentry (futuro, postergado hasta Rust 1.88+)

---

## Decisiones derivadas

* Las UUIDs viven en `.env.local`
* `HC_API_KEY` opcional para API de gestión
* Nunca hardcodear URLs de Healthchecks
* El ping ocurre SOLO si la tarea fue exitosa (o /fail si falló explícitamente)
* El worker heartbeat es obligatorio en producción
* `just deploy` registra automáticamente deploys exitosos
* Los checks son opcionales en desarrollo local
* El wrapper vive en `crates/monitoring/`
* `reqwest 0.13` es la versión oficial para HTTP client
* Sentry Crons se evaluará cuando el toolchain alcance Rust 1.88+
