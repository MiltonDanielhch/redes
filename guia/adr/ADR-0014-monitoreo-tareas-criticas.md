# ADR 0014 — Monitoreo: Healthchecks.io + Patrón Dead Man's Switch

| Campo               | Valor                                                                       |
| ------------------- | --------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                  |
| **Fecha**           | 2026                                                                        |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                            |
| **Relacionado con** | ADR 0004 (Litestream backups), ADR 0015 (Apalis jobs), ADR 0013 (Caddy TLS), ADR 0020 (Monitoreo Regional) |

---

# Contexto

En un VPS de $5 sin equipo de monitoreo 24/7, necesitamos saber si una tarea crítica dejó
de correr — antes de que el problema se vuelva un desastre.

El monitoreo tradicional (un agente que pregunta "¿estás vivo?") tiene un punto ciego fatal:
si el VPS se apaga o pierde internet, el agente interno tampoco puede avisar.

Necesitamos:

* **Alertas si el VPS muere completamente** — no solo si un proceso falla internamente
* **Costo $0** — capa gratuita suficiente para el inicio del proyecto
* **Impacto en RAM/CPU: 0** — no instalar nada en el servidor

---

# Decisión

Usar **Healthchecks.io** con el patrón de **"Dead Man’s Switch"** (interruptor del hombre muerto):
el servidor avisa que completó una tarea; si el aviso no llega, Healthchecks asume que algo falló y alerta.

---

# 1 — Patrón de monitoreo: “si no llega el ping, algo murió”

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

# 2 — Monitoreo de backups Litestream

Relacionado con ADR 0004.

Verifica que exista un snapshot actualizado en S3.

```bash
# Ejecutar cada hora
litestream snapshots s3://bucket/boilerplate/db \
  | grep -q "$(date +%Y-%m-%d)" \
  && curl -fsS -m 10 https://hc-ping.com/${HC_LITESTREAM_UUID}
```

## Qué detecta

* Litestream detenido
* Backups corruptos
* Problemas de acceso a S3
* VPS apagado
* Cron detenido

---

# 3 — Heartbeat del worker Apalis

Relacionado con ADR 0015.

```rust
// apps/api/src/jobs/worker.rs

async fn worker_heartbeat(hc_url: &str) {
    if let Err(error) = reqwest::get(hc_url).await {
        tracing::warn!(
            error = ?error,
            "healthcheck ping failed"
        );
    }
}
```

## Estrategia

* Ping cada 5 minutos
* Timeout agresivo
* Error no bloqueante
* Si el worker deja de procesar → no hay ping → alerta inmediata

---

# 4 — Verificación de TLS

Relacionado con ADR 0013.

```bash
# Verifica que el certificado no expire en menos de 30 días
openssl s_client -connect tudominio.com:443 2>/dev/null \
  | openssl x509 -noout -checkend 2592000 \
  && curl -fsS https://hc-ping.com/${HC_TLS_UUID}
```

## Beneficio

Evita caídas por certificados expirados.

---

# 5 — Monitoreo de deploys

```makefile
# justfile

deploy:
    just audit
    just test
    kamal deploy
    curl -fsS ${HC_DEPLOY_UUID:+https://hc-ping.com/$HC_DEPLOY_UUID} || true
```

## Resultado

Healthchecks mantiene historial automático de deploys exitosos.

---

# Configuración recomendada

| Check             | Intervalo | Grace Period | Severidad   |
| ----------------- | --------- | ------------ | ----------- |
| Litestream backup | 1h        | 15 min       | Alta        |
| Worker Apalis     | 5 min     | 2 min        | Crítica     |
| TLS               | 24h       | 2h           | Media       |
| Deploy            | Manual    | —            | Informativa |

---

# Variables de entorno

```bash
# .env.example

HC_LITESTREAM_UUID=
HC_DEPLOY_UUID=
HC_TLS_UUID=
HC_WORKER_UUID=
```

Todas son opcionales.

El sistema debe funcionar incluso si no están configuradas.

---

# Wrapper helper recomendado

Centralizar los pings evita repetición.

```rust
// crates/monitoring/src/healthchecks.rs

use std::time::Duration;

pub async fn ping(url: &str) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("failed to build reqwest client");

    if let Err(error) = client.get(url).send().await {
        tracing::warn!(
            error = ?error,
            "healthcheck ping failed"
        );
    }
}
```

---

# Arquitectura final

```text
┌─────────────────────────────┐
│         VPS ($5)            │
│                             │
│  ┌──────────────────────┐   │
│  │     Rust API         │───┼──── ping ───▶ Healthchecks.io
│  ├──────────────────────┤   │
│  │     Apalis Worker    │───┤
│  ├──────────────────────┤   │
│  │     Litestream       │───┤
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

# Alternativas consideradas

| Opción                    | Motivo de descarte            |
| ------------------------- | ----------------------------- |
| UptimeRobot               | Solo monitorea HTTP público   |
| Prometheus + Alertmanager | Muy pesado para 1GB RAM       |
| Scripts propios           | Mismo punto ciego del VPS     |
| Better Uptime             | Menor capa gratuita           |
| Grafana stack             | Complejidad excesiva para MVP |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta      | Propósito                                          |
| ---------------- | -------------------------------------------------- |
| `reqwest`        | Cliente HTTP robusto con timeout y retries         |
| `Sentry Crons`   | Integrar errores + cron monitoring en un dashboard |
| `Better Stack`   | Alternativa moderna todo-en-uno                    |
| Telegram Bot API | Alertas push inmediatas                            |

---

# Consecuencias

## ✅ Positivas

* Costo ≈ $0
* RAM ≈ 0
* CPU ≈ 0
* Detecta caída total del VPS
* Sin agentes residentes
* Integración extremadamente simple
* Historial automático de tareas y deploys

---

## ⚠️ Negativas / Trade-offs

### Dependencia externa

Si Healthchecks.io cae, no habrá alertas.

### Mitigación

* Configurar checks duplicados críticos
* UptimeRobot o Better Stack como backup
* Mantener logs locales estructurados

---

### No reemplaza observabilidad completa

Healthchecks dice:

> “algo dejó de correr”

pero NO explica:

> “por qué falló”

### Mitigación

Combinar con:

* tracing JSON
* request_id
* logs estructurados
* Sentry (futuro)

---

# Decisiones derivadas

* Las UUIDs viven en `.env.local`
* Nunca hardcodear URLs de Healthchecks
* El ping ocurre SOLO si la tarea fue exitosa
* El worker heartbeat es obligatorio en producción
* `just deploy` registra automáticamente deploys exitosos
* Los checks son opcionales en desarrollo local
