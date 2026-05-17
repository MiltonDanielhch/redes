# ADR 0002 — Configuración Tipeada: Fail-Fast y Secretos

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0004 (PostgreSQL), ADR 0008 (Seguridad PASETO), ADR 0012 (Herramientas), ADR 0014 (Healthchecks), ADR 0016 (Mailer Resend), ADR 0020 (Monitoreo Regional), ADR 0021 (Local-First), ADR 0022 (Agentes Distribuidos) |

---

## Contexto

Los problemas más comunes de configuración en producción son:

- Variables inexistentes
- Secretos expuestos accidentalmente
- Errores de tipos
- Configuración inconsistente entre entornos
- Fallos silenciosos en producción

El sistema de monitoreo de red para la Gobernación del Beni
necesita arrancar únicamente si toda la configuración es válida.

La configuración debe ser:

- explícita,
- tipada,
- validada,
- segura,
- predecible,
- fácil de mantener.

---

## Decisión

Usar configuración tipeada centralizada con validación fail-fast
durante el arranque de la aplicación.

La aplicación no inicia si:

- falta una variable obligatoria,
- un tipo es inválido,
- un secreto es incorrecto o inseguro,
- la configuración es inconsistente,
- una URL de servicio externo no es válida.

---

## Estructura de configuración

```rust
// crates/infrastructure/src/config/app_config.rs

use config::{Config, ConfigError, Environment};
use secrecy::{ExposeSecret, SecretBox, SecretString};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    // ── Servidor ─────────────────────────────────────────────
    pub server_port: u16,
    pub environment: AppEnvironment,
    pub rust_log: String,

    // ── Base de datos ──────────────────────────────────────
    pub database_url: SecretString,           // PostgreSQL principal (ADR 0004)
    pub sqlite_url: String,                   // SQLite para Local-First (ADR 0021)

    // ── Seguridad ──────────────────────────────────────────
    pub paseto_secret: SecretBox<[u8]>,       // PASETO v4 — 32+ bytes (ADR 0008)
    pub session_ttl_hours: u64,               // TTL de sesiones (default: 168 = 7 días)

    // ── Rate Limiting ─────────────────────────────────────
    pub rate_limit_requests_per_minute: u32,    // Default: 60 (ADR 0009)

    // ── Mailer ─────────────────────────────────────────────
    pub resend_api_key: SecretString,         // Resend API (ADR 0016)
    pub mail_from: String,                    // ej: "sistemas@beni.gob.bo"

    // ── Storage (S3/MinIO) ───────────────────────────────
    pub aws_endpoint_url_s3: String,          // ej: "https://s3.beni.gob.bo"
    pub aws_access_key_id: SecretString,
    pub aws_secret_access_key: SecretString,
    pub storage_bucket: String,               // ej: "redes-assets"

    // ── Healthchecks.io ────────────────────────────────────
    pub hc_api_key: SecretString,             // Healthchecks API (ADR 0014)
    pub hc_ping_url: String,                  // ej: "https://hc-ping.com/xxx"

    // ── Agente ─────────────────────────────────────────────
    pub agent_server_url: String,             // URL base para agentes reportar (ADR 0022)
    pub agent_api_key: SecretString,          // Auth de agentes (evaluar mTLS en ADR 0008)

    // ── Observabilidad (opcional) ─────────────────────────
    pub sentry_dsn: Option<SecretString>,     // Error tracking (opcional)
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppEnvironment {
    Development,
    Staging,
    Production,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(
                Environment::default()
                    .try_parsing(true)
                    .separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}
```

---

## Validación fail-fast

```rust
#[tokio::main]
async fn main() {
    dotenvy::from_filename(".env.local").ok();

    let config = AppConfig::load()
        .expect("❌ Configuración inválida — verificar .env.local y variables de entorno");

    // ── Validar PASETO secret ──────────────────────────────
    let paseto_bytes = config.paseto_secret.expose_secret();
    assert!(
        paseto_bytes.len() >= 32,
        "PASETO_SECRET debe tener al menos 32 bytes de entropía criptográfica (256 bits)"
    );
    // Opcional: validar entropía mínima (no todos ceros o repetidos)
    let unique_bytes = paseto_bytes.iter().collect::<std::collections::HashSet<_>>().len();
    assert!(
        unique_bytes >= 16,
        "PASETO_SECRET tiene entropía insuficiente — usar generador criptográfico"
    );

    // ── Validar DATABASE_URL ───────────────────────────────
    let db_url = config.database_url.expose_secret();
    assert!(
        db_url.starts_with("postgres://"),
        "DATABASE_URL debe ser PostgreSQL (postgres://) — SQLite no es DB principal"
    );

    // ── Validar SQLite_URL ─────────────────────────────────
    assert!(
        config.sqlite_url.starts_with("file:") || config.sqlite_url.starts_with("sqlite:"),
        "SQLITE_URL debe ser una ruta de archivo SQLite (file:./data/local.db)"
    );

    // ── Validar email ──────────────────────────────────────
    assert!(
        config.mail_from.contains('@'),
        "MAIL_FROM debe ser un email válido"
    );

    // ── Validar S3 endpoint ────────────────────────────────
    assert!(
        config.aws_endpoint_url_s3.starts_with("https://") || config.aws_endpoint_url_s3.starts_with("http://"),
        "AWS_ENDPOINT_URL_S3 debe ser una URL válida"
    );

    // ── Validar Healthchecks URL ───────────────────────────
    assert!(
        config.hc_ping_url.starts_with("https://hc-ping.com/"),
        "HC_PING_URL debe ser una URL de Healthchecks.io válida"
    );

    // ── Validar agent server URL ───────────────────────────
    assert!(
        config.agent_server_url.starts_with("https://") || config.agent_server_url.starts_with("http://"),
        "AGENT_SERVER_URL debe ser una URL válida para agentes"
    );

    // ── Validaciones de producción ─────────────────────────
    if config.environment == AppEnvironment::Production {
        assert!(
            config.sentry_dsn.is_some(),
            "SENTRY_DSN es obligatorio en producción"
        );
        assert!(
            config.resend_api_key.expose_secret().len() > 10,
            "RESEND_API_KEY es obligatorio en producción"
        );
        assert!(
            config.hc_api_key.expose_secret().len() > 10,
            "HC_API_KEY es obligatorio en producción"
        );
        // En producción, forzar HTTPS en URLs públicas
        assert!(
            config.agent_server_url.starts_with("https://"),
            "AGENT_SERVER_URL debe usar HTTPS en producción"
        );
    }

    tracing::info!(
        port = config.server_port,
        environment = ?config.environment,
        "✅ Configuración válida — arrancando sistema"
    );
}
```

---

## Jerarquía de configuración

```txt
1. Variables del sistema (env vars)     ← Producción (Docker, Coolify)
2. .env.local                            ← Desarrollo local
3. .env.example                          ← Plantilla documentada
```

**Regla:** `.env.local` nunca se sube a git. `.env.example` siempre está actualizado.

---

## `.env.example` completo

```bash
# ═══════════════════════════════════════════════════════════
#  SERVIDOR
# ═══════════════════════════════════════════════════════════
SERVER_PORT=8080
ENVIRONMENT=development
RUST_LOG=info

# ═══════════════════════════════════════════════════════════
#  BASE DE DATOS
# ═══════════════════════════════════════════════════════════
# PostgreSQL — base de datos principal (ADR 0004)
DATABASE_URL=postgres://postgres:postgres@localhost:5432/redes

# SQLite — operación offline en browser (ADR 0021)
SQLITE_URL=file:./data/local.db

# ═══════════════════════════════════════════════════════════
#  SEGURIDAD
# ═══════════════════════════════════════════════════════════
# PASETO v4 secret — 32+ bytes de entropía criptográfica
# Generar con: openssl rand -base64 32
PASETO_SECRET=CAMBIAR_POR_32_BYTES_SEGUROS_MINIMO

# TTL de sesiones en horas (default: 168 = 7 días)
SESSION_TTL_HOURS=168

# ═══════════════════════════════════════════════════════════
#  RATE LIMITING (ADR 0009)
# ═══════════════════════════════════════════════════════════
RATE_LIMIT_REQUESTS_PER_MINUTE=60

# ═══════════════════════════════════════════════════════════
#  MAILER — Resend (ADR 0016)
# ═══════════════════════════════════════════════════════════
RESEND_API_KEY=re_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
MAIL_FROM=sistemas@beni.gob.bo

# ═══════════════════════════════════════════════════════════
#  STORAGE — S3/MinIO (ADR 0020)
# ═══════════════════════════════════════════════════════════
AWS_ENDPOINT_URL_S3=https://s3.beni.gob.bo
AWS_ACCESS_KEY_ID=AKIAXXXXXXXXXXXXXXXX
AWS_SECRET_ACCESS_KEY=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
STORAGE_BUCKET=redes-assets

# ═══════════════════════════════════════════════════════════
#  HEALTHCHECKS.IO (ADR 0014)
# ═══════════════════════════════════════════════════════════
HC_API_KEY=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
HC_PING_URL=https://hc-ping.com/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx

# ═══════════════════════════════════════════════════════════
#  AGENTE DE MONITOREO (ADR 0022)
# ═══════════════════════════════════════════════════════════
# URL donde los agentes reportan métricas
AGENT_SERVER_URL=https://api.redes.beni.gob.bo

# API key para autenticación de agentes (evaluar mTLS en producción)
AGENT_API_KEY=agent_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# ═══════════════════════════════════════════════════════════
#  OBSERVABILIDAD (opcional)
# ═══════════════════════════════════════════════════════════
# Sentry DSN — solo en producción
# SENTRY_DSN=https://xxx@xxx.ingest.sentry.io/xxx
```

---

## Principios adoptados

| Principio | Descripción |
|-----------|-------------|
| Fail-fast | El sistema falla antes de aceptar tráfico |
| Configuración explícita | Todo parámetro existe en `AppConfig` |
| Tipado fuerte | Nada importante se maneja como string ambiguo |
| Secretos fuera de git | Nunca se suben `.env` reales |
| Secretos en memoria segura | `SecretString` / `SecretBox` evita exposición en logs/Debug |
| Inmutabilidad | La configuración no cambia en runtime |
| Validación de URLs | Toda URL de servicio externo se valida al arrancar |
| Entropía criptográfica | Secretos validados por longitud y aleatoriedad |

---

## Herramientas aprobadas

| Herramienta | Propósito | Versión |
|-------------|-----------|---------|
| `config` | Carga tipada de configuración | `0.15.22` |
| `dotenvy` | Variables locales de desarrollo | `0.15.7` |
| `secrecy` | Protección de secretos sensibles en memoria | `0.10.3` |
| `shadow-rs` | Metadata del build (git hash, fecha) | `0.35.x` |
| `taplo` | Orden del workspace TOML | `0.10.0` (CLI) |

---

## Comparativa

| Estrategia | Resultado |
|-------------|-----------|
| `std::env::var` disperso | Difícil mantenimiento, secretos expuestos |
| Configuración tipada + `secrecy` | Centralización, seguridad, validación automática |

---

## Consecuencias

### ✅ Positivas

- Errores detectados al arrancar (no en producción a las 3am)
- Configuración documentada automáticamente vía `.env.example`
- Secretos protegidos en memoria (no en logs ni stack traces)
- Menos errores humanos por tipos incorrectos
- Menos fallos en producción por configuración incompleta
- Configuración consistente entre entornos (dev/staging/prod)
- Fácil mantenimiento — un solo struct con todos los campos

---

### ⚠️ Trade-offs

- Cada nueva variable requiere recompilación (cambio en `AppConfig`)
- Mayor disciplina inicial (todos los campos deben estar en el struct)
- Más estricta que configuraciones dinámicas (no se puede cambiar en runtime)
- `.env.example` debe mantenerse sincronizado manualmente

---

## Reglas obligatorias

| Regla | Descripción |
|-------|-------------|
| R1 | `.env` y `.env.local` siempre están en `.gitignore` |
| R2 | `.env.example` siempre está actualizado con todos los campos |
| R3 | Los secretos de producción se inyectan vía Docker/Coolify (no en repo) |
| R4 | Nunca se escriben secretos en logs (usar `secrecy::ExposeSecret` solo cuando es necesario) |
| R5 | Nunca se hardcodean claves en el código fuente |
| R6 | Todos los secretos usan `SecretString` o `SecretBox` en `AppConfig` |
| R7 | Validación de entropía mínima en secretos criptográficos (PASETO, API keys) |
| R8 | URLs de servicios externos validadas al arrancar (formato, protocolo HTTPS en prod) |

---

## Impacto regional

La estrategia fail-fast evita despliegues defectuosos
en oficinas donde el soporte técnico puede ser limitado.

Si falta configuración:

- el sistema falla inmediatamente,
- el error es explícito y legible,
- y el problema se detecta antes de entrar en operación.

Esto reduce:

- tiempo de diagnóstico,
- errores silenciosos,
- mantenimiento remoto innecesario,
- riesgo de exposición de secretos.

---

## Resultado esperado

Un sistema:

- predecible,
- seguro,
- fácil de desplegar,
- fácil de mantener,
- robusto ante errores humanos,
- con secretos protegidos en memoria,
- y compatible con infraestructura limitada.

---

## Notas de actualización de versiones (2026-05-16)

| Componente | Versión/Config | Notas |
|------------|----------------|-------|
| **config** | **0.15.22** | Última estable (mar 2026). Soporta múltiples fuentes, layered overrides, integración serde. |
| **dotenvy** | **0.15.7** | Última estable (mar 2023). Fork mantenido de `dotenv`. MSRV 1.56.1. |
| **secrecy** | **0.10.3** | Última estable (oct 2024). `SecretBox`/`SecretString` con `zeroize` en drop. `serde` feature para Deserialize. |
| **shadow-rs** | **0.35.x** | Última estable. Build-time metadata (git hash, fecha, toolchain). Requiere `build.rs`. |
| **taplo** | **0.10.0** (CLI) | Última estable (may 2025). Toolkit TOML: formatter, linter, schema. |
| **dotenvy-derive** | **0.15.8** (unstable) | Macro `#[derive(LoadEnv)]`. Usa Rust 2024. Aún en desarrollo. |
