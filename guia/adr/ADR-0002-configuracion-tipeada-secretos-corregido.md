# ADR 0002 — Configuración Tipeada: Fail-Fast y Secretos

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (postgresql), ADR 0008 (Seguridad), ADR 0012 (CLI Setup), ADR 0020 (Monitoreo Regional) |

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

- falta una variable,
- un tipo es inválido,
- un secreto es incorrecto,
- la configuración es inconsistente.

---

## Estructura de configuración

```rust
// crates/infrastructure/src/config/app_config.rs

use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    // Servidor
    pub server_port: u16,
    pub environment: AppEnvironment,
    pub rust_log: String,

    // Base de datos
    pub database_url: String,

    // Seguridad
    pub paseto_secret: String,

    // Observabilidad
    pub sentry_dsn: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppEnvironment {
    Development,
    Production,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(
                Environment::default()
                    .try_parsing(true),
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
        .expect("Configuración inválida");

    assert!(
        config.paseto_secret.len() == 32,
        "PASETO_SECRET debe tener 32 bytes"
    );

    if config.environment == AppEnvironment::Production {
        assert!(
            config.sentry_dsn.is_some(),
            "SENTRY_DSN es obligatorio en producción"
        );
    }

    tracing::info!(
        port = config.server_port,
        "configuración válida"
    );
}
```

---

## Jerarquía de configuración

```txt
1. Variables del sistema       ← Producción
2. .env.local                  ← Desarrollo
3. .env.example                ← Plantilla
```

---

## `.env.example`

```bash
# Servidor
SERVER_PORT=8080
ENVIRONMENT=development
RUST_LOG=info

# PostgreSQL
DATABASE_URL=postgres://postgres:postgres@localhost:5432/redes

# Seguridad
PASETO_SECRET=CAMBIAR_POR_32_BYTES_SEGUROS

# Observabilidad
# SENTRY_DSN=https://xxx@sentry.io/xxx
```

---

## Principios adoptados

| Principio | Descripción |
|-----------|-------------|
| Fail-fast | El sistema falla antes de aceptar tráfico |
| Configuración explícita | Todo parámetro existe en `AppConfig` |
| Tipado fuerte | Nada importante se maneja como string ambiguo |
| Secretos fuera de git | Nunca se suben `.env` reales |
| Inmutabilidad | La configuración no cambia en runtime |

---

## Simplificaciones aplicadas

La configuración se limita únicamente
a lo necesario para el MVP del sistema regional.

No se incluyen todavía:

- Tigris
- S3 avanzado
- OTLP
- Healthchecks.io
- Multicloud
- Feature flags distribuidos
- Configuración dinámica

Estas capacidades solo se incorporarán
si existe necesidad operacional real.

---

## Herramientas aprobadas

| Herramienta | Propósito |
|-------------|-----------|
| `config` | Carga tipada de configuración |
| `dotenvy` | Variables locales de desarrollo |
| `secrecy` | Protección de secretos sensibles |
| `shadow-rs` | Metadata del build |
| `taplo` | Orden del workspace |

---

## Comparativa

| Estrategia | Resultado |
|-------------|-----------|
| `std::env::var` disperso | Difícil mantenimiento |
| Configuración tipada | Centralización y seguridad |

---

## Consecuencias

### ✅ Positivas

- Errores detectados al arrancar
- Configuración documentada automáticamente
- Menos errores humanos
- Menos fallos en producción
- Configuración consistente entre entornos
- Fácil mantenimiento

---

### ⚠️ Trade-offs

- Cada nueva variable requiere recompilación
- Mayor disciplina inicial
- Más estricta que configuraciones dinámicas

---

## Reglas obligatorias

- `.env` y `.env.local` siempre están en `.gitignore`
- `.env.example` siempre está actualizado
- Los secretos de producción se inyectan vía Docker/Kamal
- Nunca se escriben secretos en logs
- Nunca se hardcodean claves en el código fuente

---

## Impacto regional

La estrategia fail-fast evita despliegues defectuosos
en oficinas donde el soporte técnico puede ser limitado.

Si falta configuración:

- el sistema falla inmediatamente,
- el error es explícito,
- y el problema se detecta antes de entrar en operación.

Esto reduce:

- tiempo de diagnóstico,
- errores silenciosos,
- y mantenimiento remoto innecesario.

---

## Resultado esperado

Un sistema:

- predecible,
- seguro,
- fácil de desplegar,
- fácil de mantener,
- robusto ante errores humanos,
- y compatible con infraestructura limitada.
