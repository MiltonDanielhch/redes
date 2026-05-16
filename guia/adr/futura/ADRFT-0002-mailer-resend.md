# ADR (futuro) — Email Transaccional: Resend + React Email + LogMailer

| Campo               | Valor                                                                                                                                 |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| **Estado**          | ⏳ Pendiente                                                                                                                             |
| **Fecha**           | 2026                                                                                                                                  |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                                     |
| **Relacionado con** | ADR 0015 (Apalis — Jobs asíncronos), ADR 0002 (Config), ADR 0010 (Testing), ADR 0011 (Ciclo Lab→Puente→Producción), ADR 0020 (Monitoreo Regional) |

---

# Contexto

El sistema necesita enviar correos transaccionales para:

* Recuperación de contraseña
* Notificaciones del sistema
* Alertas críticas de red
* Confirmaciones de registro
* Posibles futuras alertas SNMP/email

El ADR 0015 define `EmailJob` como job asíncrono, pero no define:

* proveedor de envío
* estrategia de templates
* comportamiento en desarrollo
* aislamiento del proveedor externo

Necesitamos una solución que:

* Funcione bien en un VPS pequeño
* Tenga alta deliverability
* Sea fácil de reemplazar
* No complique el desarrollo local
* Permita renderizar emails modernos y mantenibles

---

# Decisión

Usar:

* **Resend** como proveedor de email transaccional
* **React Email** para templates HTML
* **LogMailer** en desarrollo
* **Puerto `Mailer` abstracto** en dominio

La arquitectura sigue el patrón hexagonal:

```text
Application
    ↓
Port: Mailer
    ↓
Adapter:
 ├─ ResendMailer
 └─ LogMailer
```

---

# Objetivos Técnicos

| Objetivo                           | Resultado                           |
| ---------------------------------- | ----------------------------------- |
| Cambiar proveedor fácilmente       | Solo cambia `crates/mailer`         |
| Desarrollo sin credenciales reales | `LogMailer`                         |
| Evitar bloqueo HTTP                | Emails enviados vía jobs asíncronos |
| Templates mantenibles              | React Email                         |
| Reducir complejidad SMTP           | API HTTP simple                     |
| Alta entregabilidad                | SPF/DKIM manejado por Resend        |

---

# Dependencias

## Rust

```toml
# crates/mailer/Cargo.toml

[dependencies]
resend-rs = "0.5"
async-trait = "0.1"
tracing = "0.1"
```

---

## Frontend / templates

```bash
pnpm add @react-email/components react react-dom
```

---

# Puerto de Dominio

El dominio NO conoce Resend.

```rust
// crates/domain/src/ports/mailer.rs

use async_trait::async_trait;

#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(
        &self,
        email: EmailMessage,
    ) -> Result<(), MailerError>;
}

pub struct EmailMessage {
    pub to:       String,
    pub subject:  String,
    pub html:     String,
    pub reply_to: Option<String>,
}

#[derive(Debug)]
pub enum MailerError {
    Provider(String),
    Template(String),
}
```

---

# LogMailer — Desarrollo Local

En desarrollo NO se envían emails reales.

```rust
// crates/mailer/src/log_mailer.rs

pub struct LogMailer;

#[async_trait]
impl Mailer for LogMailer {
    async fn send(
        &self,
        email: EmailMessage,
    ) -> Result<(), MailerError> {

        tracing::info!(
            to      = %email.to,
            subject = %email.subject,
            html    = %email.html,
            "📧 [LogMailer] Email simulado"
        );

        Ok(())
    }
}
```

---

# ResendMailer — Producción

```rust
// crates/mailer/src/resend_mailer.rs

use resend_rs::{
    types::CreateEmailBaseOptions,
    Resend,
};

pub struct ResendMailer {
    client: Resend,
    from: String,
}

impl ResendMailer {
    pub fn new(
        api_key: &str,
        from: &str,
    ) -> Self {
        Self {
            client: Resend::new(api_key),
            from: from.to_string(),
        }
    }
}

#[async_trait]
impl Mailer for ResendMailer {
    async fn send(
        &self,
        email: EmailMessage,
    ) -> Result<(), MailerError> {

        let options = CreateEmailBaseOptions::new(
            &self.from,
            [email.to.as_str()],
            &email.subject,
        )
        .with_html(&email.html);

        self.client
            .emails
            .send(options)
            .await
            .map_err(|e| MailerError::Provider(e.to_string()))?;

        Ok(())
    }
}
```

---

# Selección del Mailer (Composition Root)

```rust
// apps/api/src/setup/build_mailer.rs

pub fn build_mailer(
    config: &AppConfig,
) -> Arc<dyn Mailer> {

    match config.environment {
        AppEnvironment::Development => {
            Arc::new(LogMailer)
        }

        AppEnvironment::Production |
        AppEnvironment::Staging => {
            Arc::new(
                ResendMailer::new(
                    &config.resend_api_key,
                    &config.mail_from,
                )
            )
        }
    }
}
```

---

# Templates con React Email

## Ejemplo

```tsx
// apps/mailer/emails/welcome.tsx

import {
    Html,
    Head,
    Body,
    Container,
    Text,
    Button,
} from '@react-email/components';

interface Props {
    email: string;
}

export default function WelcomeEmail({
    email,
}: Props) {

    return (
        <Html>
            <Head />

            <Body
                style={{
                    fontFamily: 'sans-serif',
                    backgroundColor: '#f5f5f5',
                }}
            >
                <Container>
                    <Text>
                        Bienvenido {email}
                    </Text>

                    <Button href="https://tudominio.com/login">
                        Ingresar al sistema
                    </Button>
                </Container>
            </Body>
        </Html>
    );
}
```

---

# Build de Templates

```bash
pnpm --filter mailer build
```

Genera:

```text
apps/mailer/dist/
 ├─ welcome.html
 ├─ password_reset.html
 ├─ alert_notification.html
 └─ network_alert.html
```

---

# Integración con Apalis (ADR 0015)

```rust
// apps/api/src/jobs/email_job.rs

pub async fn handle_email_job(
    job: EmailJob,
    ctx: JobContext,
) -> Result<(), JobError> {

    let html = render_template(&job.template)?;

    ctx.data::<Arc<dyn Mailer>>()
        .unwrap()
        .send(EmailMessage {
            to:       job.to,
            subject:  job.subject,
            html,
            reply_to: None,
        })
        .await
        .map_err(JobError::from)
}
```

---

# Renderizado de Templates

```rust
fn render_template(
    template: &EmailTemplate,
) -> Result<String, JobError> {

    let html = match template {

        EmailTemplate::Welcome => {
            include_str!(
                "../../../apps/mailer/dist/welcome.html"
            )
        }

        EmailTemplate::PasswordReset => {
            include_str!(
                "../../../apps/mailer/dist/password_reset.html"
            )
        }

        EmailTemplate::NetworkAlert => {
            include_str!(
                "../../../apps/mailer/dist/network_alert.html"
            )
        }

        EmailTemplate::Notification => {
            include_str!(
                "../../../apps/mailer/dist/notification.html"
            )
        }
    };

    Ok(html.to_string())
}
```

---

# Variables de Entorno

```env
# .env.example

RESEND_API_KEY=re_xxxxxxxxx
MAIL_FROM="Gobernación Beni <noreply@beni.gob.bo>"
```

---

# Flujo Completo

```text
Usuario / Evento SNMP
        ↓
Use Case
        ↓
EmailJob
        ↓
Apalis Worker
        ↓
Mailer Port
        ↓
ResendMailer
        ↓
API Resend
        ↓
Email entregado
```

---

# Testing

## Test unitario del puerto

```rust
#[tokio::test]
async fn log_mailer_no_falla() {
    let mailer = LogMailer;

    let result = mailer.send(EmailMessage {
        to: "test@test.com".into(),
        subject: "hola".into(),
        html: "<h1>hola</h1>".into(),
        reply_to: None,
    }).await;

    assert!(result.is_ok());
}
```

---

## Test E2E

```rust
#[tokio::test]
async fn registro_dispara_email_job() {
    // Verifica que el evento genera EmailJob
}
```

---

# Seguridad

## Reglas

* Nunca enviar emails directamente desde handlers HTTP
* Todo email pasa por jobs asíncronos
* `RESEND_API_KEY` solo existe en producción/staging
* `LogMailer` evita envío accidental durante desarrollo
* No guardar HTML generado en DB
* Los templates se incluyen en el binario usando `include_str!`

---

# Observabilidad

Integración recomendada:

| Herramienta     | Uso                   |
| --------------- | --------------------- |
| `tracing`       | Logs estructurados    |
| `Sentry`        | Captura de errores    |
| Healthchecks.io | Verificar worker vivo |
| `request_id`    | Trazabilidad completa |

Ejemplo:

```rust
tracing::info!(
    request_id = %request_id,
    to = %email.to,
    "email enviado"
);
```

---

# Alternativas consideradas

| Opción      | Motivo de descarte                         |
| ----------- | ------------------------------------------ |
| SMTP propio | Complejidad innecesaria                    |
| SendGrid    | API más pesada                             |
| AWS SES     | IAM compleja                               |
| Postmark    | Excelente pero menos generoso en free tier |
| SMTP Gmail  | Mala deliverability y límites bajos        |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta         | Propósito                       |
| ------------------- | ------------------------------- |
| **`lettre`**        | Migración futura a SMTP         |
| **`preview-email`** | Vista previa local de templates |
| **`tracing`**       | Logs estructurados              |
| **`sentry`**        | Captura de errores de entrega   |
| **`react-email`**   | Templates modernos compatibles  |
| **`resend-rs`**     | SDK oficial Rust                |

---

# Consecuencias

## ✅ Positivas

* Desarrollo local sin emails reales
* Alta deliverability desde el inicio
* Arquitectura desacoplada del proveedor
* Templates modernos y reutilizables
* Emails enviados fuera del request HTTP
* Compatible con arquitectura hexagonal

---

## ⚠️ Negativas / Trade-offs

### Dependencia externa

Resend es SaaS.

→ Mitigado con el puerto `Mailer`

→ Cambiar proveedor requiere modificar solo un adapter

---

### Paso adicional de build

Los templates deben compilarse antes del build Rust.

→ `just build` ejecuta:

```bash
pnpm --filter mailer build
cargo build --release
```

---

### Mayor complejidad inicial

Separar mailer, templates y jobs agrega estructura.

→ Beneficio: desacoplamiento total y mantenibilidad a largo plazo.

---

# Decisiones derivadas

* `just build` ejecuta el build del mailer primero
* `LogMailer` es obligatorio en development
* Todos los emails pasan por Apalis
* `Mailer` vive en `crates/domain`
* Los templates viven en `apps/mailer/emails`
* `apps/mailer/dist` se copia durante el build del contenedor
* Nunca enviar emails síncronos desde handlers HTTP
* El worker de emails debe tener monitoreo en Healthchecks.io (ADR 0015)
