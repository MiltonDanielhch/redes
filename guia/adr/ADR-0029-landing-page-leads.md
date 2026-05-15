# ADR 0029 — Landing Page: Captura de Leads + SEO + Conversión

| Campo               | Valor                                                                                                              |
| ------------------- | ------------------------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                                         |
| **Fecha**           | 2026                                                                                                               |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                   |
| **Relacionado con** | ADR 0022 (SvelteKit + Svelte 5), ADR 0018 (Apalis Jobs), ADR 0016 (Resend + React Email), ADR 0009 (Rate Limiting) |

---

# Contexto

El ecosistema del Laboratorio 3030 necesita una landing page pública que funcione como:

* página institucional
* sistema de captura de leads
* canal SEO principal
* entrada de futuros usuarios
* validación temprana del producto

La landing debe:

* cargar instantáneamente incluso en conexiones lentas
* funcionar sin JavaScript para SEO
* capturar leads sin fricción
* resistir spam y bots
* mantener excelente Core Web Vitals
* integrarse con el backend Rust sin servicios externos

Además:

* los leads NO son usuarios autenticados
* no deben contaminar el sistema de auth
* el proceso debe ser simple y mantenible

---

# Decisión

Usar:

* **Astro SSR** para renderizado HTML ultra rápido
* **Svelte 5** únicamente para componentes interactivos
* endpoint backend `/api/v1/leads`
* `LeadWelcomeJob` vía Apalis
* anti-spam mediante:

  * honeypot
  * rate limiting
  * deduplicación silenciosa
* SEO completo server-side

---

# Arquitectura de la landing

```text id="09nqoq"
apps/web/
├── src/
│   ├── pages/
│   │   └── index.astro
│   │
│   ├── layouts/
│   │   └── LandingLayout.astro
│   │
│   ├── components/
│   │   └── landing/
│   │       ├── Hero.svelte
│   │       ├── Features.svelte
│   │       ├── SocialProof.svelte
│   │       ├── CTA.svelte
│   │       └── LeadForm.svelte
│   │
│   └── lib/
│       ├── seo/
│       ├── analytics/
│       └── validation/
│
└── static/
```

---

# Entidad de dominio `Lead`

```rust id="1g6bsm"
#[derive(Debug, Clone)]
pub struct Lead {
    pub id:         LeadId,
    pub email:      Email,
    pub name:       Option<String>,
    pub source:     LeadSource,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub enum LeadSource {
    Landing,
    Referral,
    Social,
    Campaign,
}
```

---

# Filosofía del sistema de leads

Un lead:

* no tiene password
* no tiene sesión
* no puede iniciar sesión
* no pertenece al sistema RBAC

Es únicamente:

* una intención de interés
* un contacto potencial
* un punto de conversión

---

# Caso de uso `CaptureLead`

```rust id="om3p8x"
pub async fn execute(
    &self,
    input: CaptureLeadInput,
) -> Result<(), AppError> {
    let email = Email::new(&input.email)?;

    // Deduplicación silenciosa
    // Nunca revelar si existe o no
    if self.leads.find_by_email(&email).await?.is_some() {
        return Ok(());
    }

    let lead = Lead::new(
        email,
        input.name,
        LeadSource::Landing,
    );

    self.leads.save(&lead).await?;

    // Async — no bloquear HTTP
    self.jobs.push(EmailJob {
        to:       lead.email.to_string(),
        subject:  "Gracias por tu interés".into(),
        template: EmailTemplate::LeadWelcome,
        context: serde_json::json!({
            "email": lead.email
        }),
    }).await?;

    Ok(())
}
```

---

# Endpoint HTTP

```rust id="8wuzk6"
#[utoipa::path(
    post,
    path = "/api/v1/leads",
    request_body = CaptureLeadRequest,
    responses(
        (status = 200, description = "Lead registrado"),
        (status = 429, description = "Rate limit excedido"),
    ),
    tag = "leads",
)]
pub async fn capture_lead(
    State(state): State<AppState>,
    Json(body): Json<CaptureLeadRequest>,
) -> Result<StatusCode, AppError> {
    CaptureLead::new(&state)
        .execute(body.into())
        .await?;

    Ok(StatusCode::OK)
}
```

---

# Rate limiting estricto

```rust id="mtjlwm"
Router::new()
    .route("/api/v1/leads", post(capture_lead))
    .layer(leads_rate_limit())
```

## Política

| Regla    | Valor    |
| -------- | -------- |
| Requests | 3        |
| Ventana  | 1 minuto |
| Scope    | por IP   |
| Exceso   | HTTP 429 |

---

# Frontend — formulario Svelte 5

```svelte id="9n9n0x"
<script lang="ts">
    import { type } from 'arktype';

    const LeadSchema = type({
        email: 'string.email',
        name: 'string?',
        honeypot: 'string?',
    });

    let email = $state('');
    let name = $state('');
    let honeypot = $state('');

    let status = $state<
        'idle' |
        'loading' |
        'success' |
        'error'
    >('idle');

    async function handleSubmit() {
        if (honeypot) return;

        const result = LeadSchema({
            email,
            name,
            honeypot,
        });

        if (result instanceof type.errors) return;

        status = 'loading';

        try {
            await fetch('/api/v1/leads', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    email,
                    name,
                }),
            });

            status = 'success';
        } catch {
            status = 'error';
        }
    }
</script>
```

---

# Anti-spam

## Estrategia MVP

| Defensa            | Objetivo           |
| ------------------ | ------------------ |
| Honeypot           | Bots simples       |
| Rate limiting      | Flood              |
| Deduplicación      | Spam repetido      |
| Respuesta uniforme | Evitar enumeración |

---

# Honeypot

```html id="8e6oyx"
<input
    type="text"
    style="display:none"
    tabindex="-1"
    autocomplete="off"
/>
```

Los usuarios reales nunca lo ven.

Bots automatizados suelen rellenarlo.

---

# SEO SSR

La landing se renderiza completamente desde el servidor.

Google puede indexar:

* contenido
* headings
* metadata
* OpenGraph
* sitemap

sin necesidad de JavaScript.

---

# Landing principal

```astro id="bpx5vw"
<LandingLayout
    title="Laboratorio 3030"
    description="Rust · Axum · Svelte 5 · ConnectRPC"
>
    <Hero />
    <Features />
    <SocialProof />
    <CTA>
        <LeadForm client:load />
    </CTA>
</LandingLayout>
```

---

# Layout SEO

```astro id="z80w13"
<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8" />

    <meta
        name="viewport"
        content="width=device-width, initial-scale=1"
    />

    <title>{Astro.props.title}</title>

    <meta
        name="description"
        content={Astro.props.description}
    />

    <meta
        property="og:title"
        content={Astro.props.title}
    />

    <meta
        property="og:description"
        content={Astro.props.description}
    />
</head>
<body>
    <slot />
</body>
</html>
```

---

# Tabla SQL

```sql id="rwq6p1"
CREATE TABLE IF NOT EXISTS leads (
    id         TEXT PRIMARY KEY NOT NULL,
    email      TEXT NOT NULL UNIQUE,
    name       TEXT,
    source     TEXT NOT NULL DEFAULT 'landing',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_leads_email
ON leads(email);

CREATE INDEX IF NOT EXISTS idx_leads_source
ON leads(source);
```

---

# Core Web Vitals

| Métrica | Objetivo | Estrategia                |
| ------- | -------- | ------------------------- |
| LCP     | <2.5s    | Astro SSR                 |
| CLS     | <0.1     | imágenes con dimensiones  |
| INP     | <100ms   | Svelte sin runtime pesado |
| TTFB    | <200ms   | Axum + SQLite             |

---

# Estrategia visual

## Stack UI

| Herramienta  | Uso              |
| ------------ | ---------------- |
| TailwindCSS  | Layout           |
| Astro Image  | Optimización     |
| Svelte 5     | Interactividad   |
| Lucide Icons | Íconos           |
| Motion One   | Microanimaciones |

---

# Analytics

## Eventos mínimos

| Evento           | Objetivo    |
| ---------------- | ----------- |
| `lead_submitted` | conversión  |
| `cta_clicked`    | interés     |
| `scroll_depth`   | engagement  |
| `hero_viewed`    | visibilidad |

---

# Emails automáticos

Después de capturar el lead:

```text id="m6gskn"
Lead creado
→ Apalis enqueue
→ LeadWelcomeJob
→ ResendMailer
→ Email enviado
```

Todo async.

La respuesta HTTP nunca espera el email.

---

# Estrategia de conversión

## Objetivos UX

* formulario mínimo
* cero fricción
* no pedir password
* no pedir teléfono
* feedback inmediato
* mobile-first

---

# Alternativas consideradas

| Opción           | Motivo de descarte       |
| ---------------- | ------------------------ |
| Webflow          | stack separado           |
| Framer           | dependencia externa      |
| Google Forms     | datos fuera del sistema  |
| reCAPTCHA        | UX degradada             |
| Leads como users | mezcla responsabilidades |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta            | Propósito                   |
| ---------------------- | --------------------------- |
| `@astrojs/sitemap`     | sitemap automático          |
| `astro-compress`       | compresión HTML/CSS         |
| `Astro Image`          | imágenes AVIF/WebP          |
| `Cloudflare Turnstile` | anti-spam avanzado          |
| `PostHog`              | analytics y funnels         |
| `Partytown`            | scripts terceros en workers |
| `arktype`              | validación runtime tipada   |

---

# Consecuencias

## ✅ Positivas

* Landing extremadamente rápida
* SEO completo sin JS
* Leads centralizados en la misma DB
* Sistema async y desacoplado
* Infraestructura simple
* No depende de SaaS externos
* Conversión optimizada para móviles

---

## ⚠️ Negativas / Trade-offs

### Los leads no son usuarios

→ requiere migración manual si se convierten

→ mitigado porque el flujo está controlado

---

### Honeypot no detiene bots sofisticados

→ mitigado con:

* rate limiting
* deduplicación
* futura integración con Turnstile

---

### Analytics añade scripts externos

→ mitigado usando Partytown para no bloquear el hilo principal

---

# Decisiones derivadas

* `/api/v1/leads` tiene rate limiting obligatorio
* Los leads usan entidad separada de `User`
* El email de bienvenida es async mediante Apalis
* `LandingLayout` y `DashboardLayout` son independientes
* `LeadWelcomeJob` usa ADR 0016
* Analytics nunca deben bloquear render SSR
* El formulario debe funcionar correctamente en móviles desde 320px de ancho
