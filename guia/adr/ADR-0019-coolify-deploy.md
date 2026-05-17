# ADR 0019 — Deploy con Coolify + PostgreSQL

| Campo               | Valor                                                                                                               |
| ------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                                                          |
| **Fecha**           | 2026-05-16                                                                                                          |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                    |
| **Versión**         | 2.1 (Corrección 2026-05-16)                                                                                         |
| **Relacionado con** | ADR 0004 (PostgreSQL + Docker), ADR 0013 (Infraestructura Docker Compose), ADR 0015 (Jobs + Apalis), ADR 0020 (Monitoreo Regional) |

---

# Contexto

Coolify es una plataforma de deployment todo-en-uno que se ejecuta en el VPS y permite:

* build local
* push al registry
* deploy vía SSH
* rollback instantáneo
* sin componentes extra en el VPS

Este enfoque es extremadamente eficiente para VPS pequeños.

Sin embargo, algunos equipos prefieren una experiencia tipo PaaS con:

* dashboard web
* deploy automático desde Git
* SSL visual
* manejo centralizado de múltiples aplicaciones
* logs y rollbacks desde interfaz gráfica

Para esto, **Coolify** surge como alternativa self-hosted moderna.

La pregunta clave es:

> ¿Coolify puede ejecutar correctamente este stack basado en PostgreSQL sin romper la arquitectura?

---

# Decisión

Se usa **Coolify** como sistema de deploy oficial alternativo a Kamal.

Coolify:

* NO reemplaza la arquitectura actual
* NO modifica el Containerfile
* NO cambia PostgreSQL
* NO introduce Kubernetes
* NO requiere reescribir el sistema

Solo cambia la capa operativa del deploy.

**Nota:** El deploy principal del MVP sigue siendo **Kamal** (ADR 0013, ROADMAP-MASTER.md Día 9). Coolify es alternativa validada para escenarios multi-proyecto o equipos que prefieren dashboard visual.

---

# Principio arquitectónico

La aplicación debe seguir siendo:

* un contenedor autónomo para la API
* PostgreSQL como base de datos principal (ADR 0004)
* SQLite Wasm solo en cliente para Local-First (ADR 0021)
* respaldos automáticos de PostgreSQL (pgBackRest / pg_dump)
* healthcheck HTTP
* sin dependencias externas obligatorias

Coolify únicamente orquesta el ciclo de vida del contenedor de la API y puede gestionar PostgreSQL como servicio adjunto.

---

# Compatibilidad con PostgreSQL

## Estado

| Componente                 | Compatibilidad                     | Notas |
| -------------------------- | ---------------------------------- | ----- |
| PostgreSQL                 | ✅ Compatible                       | Servicio gestionado por Coolify o contenedor separado |
| Axum                       | ✅ Compatible                       | Sin cambios |
| Healthcheck `/health`      | ✅ Compatible                       | Requerido por Coolify para health checks |
| Apalis Jobs                | ✅ Compatible                       | Requiere PostgreSQL para queue storage |
| pgBackRest / pg_dump       | ✅ Compatible                       | Backups nativos de PostgreSQL |
| Distroless                 | ✅ Compatible                       | Containerfile sin cambios |
| Volume persistence         | ✅ Requiere configuración explícita | Volumen persistente para datos de PostgreSQL |
| Litestream                 | ❌ **No aplica**                    | Proyecto usa PostgreSQL, no SQLite (ADR 0004) |

**Nota crítica:** El proyecto utiliza **PostgreSQL** como base de datos principal (ADR 0004, ADR 0020). Litestream es una herramienta de replicación para **SQLite**, no para PostgreSQL. Su inclusión en este ADR era un error de coherencia con ADR 0004. Los backups de PostgreSQL se realizan con `pgBackRest`, `pg_dump` o el mecanismo nativo de Coolify para snapshots de volúmenes.

---

# Arquitectura con Coolify

```text
Git Push
   ↓
Coolify Webhook
   ↓
Build Docker Image (API Rust distroless)
   ↓
Deploy Container API
   ↓
PostgreSQL Service (gestionado por Coolify o contenedor Docker)
   ↓
Volume persistente /var/lib/postgresql/data
   ↓
Axum inicia → Pool PostgreSQL
   ↓
/health OK
   ↓
Traffic swap (Traefik / Nginx)
```

**Nota:** No hay Litestream en el flujo. El backup de PostgreSQL es responsabilidad del operador (pgBackRest, snapshots de volumen, o backup automático de Coolify si soporta). Coolify v4+ gestiona Traefik 3.7.x como reverse proxy interno.

---

# Persistencia de PostgreSQL

## Regla obligatoria

PostgreSQL debe vivir en un volumen persistente Docker.

En Coolify:

```text
Persistent Storage
Destination Path: /var/lib/postgresql/data
```

Entonces:

```env
DATABASE_URL=postgres://user:password@postgres:5432/redes
```

**Nota:** `DATABASE_URL` apunta a PostgreSQL, no a SQLite. El proyecto no usa SQLite en el servidor (ADR 0004). SQLite solo se usa en el cliente (frontend Wasm) para Local-First (ADR 0021).

---

# Backups de PostgreSQL

## Estrategia

El proyecto no usa Litestream. Las estrategias de backup son:

| Herramienta | Propósito | Frecuencia | Versión referencia |
|-------------|-----------|------------|-------------------|
| `pg_dump` | Backup lógico completo | Diario (cron job o Apalis) | PostgreSQL 17+ |
| `pgBackRest` | Backup físico con WAL archiving | Continuo (si se configura) | **2.58.0** (ene 2026) |
| Coolify Snapshots | Snapshot de volumen | Según configuración de Coolify | Coolify native |
| S3 / MinIO | Destino de backups | Automático vía rclone o AWS CLI | Latest |

### Backup con pg_dump (recomendado para MVP)

```bash
# En un job de Apalis o cron dentro del VPS
pg_dump -h postgres -U redes -d redes -F c -f /backup/redes-$(date +%Y%m%d_%H%M%S).dump
```

### Restauración

```bash
pg_restore -h postgres -U redes -d redes --clean /backup/redes-20260516_120000.dump
```

**Advertencia pgBackRest (2026-05-16):** pgBackRest 2.58.0 (enero 2026) es la última release publicada. Existen reportes de que el proyecto original enfrenta desafíos de sostenibilidad a largo plazo. Para nuevas infraestructuras, evaluar alternativas como **Barman** (EnterpriseDB) o **WAL-G** (cloud-native) si se requiere soporte enterprise continuado. pgBackRest sigue funcionando correctamente, pero conviene monitorear su roadmap.

---

# Healthcheck

```text
Path: /health
Port: 8080
Interval: 10s
Timeout: 5s
Retries: 3
```

Coolify no redirige tráfico hasta que el healthcheck pasa.

El endpoint `/health` debe verificar:
- Conectividad al pool PostgreSQL
- Estado del servidor Axum
- (Opcional) Estado de la cola de jobs Apalis

---

# Containerfile

El Containerfile del ADR 0013 funciona sin modificaciones, actualizando la versión base de Rust.

```dockerfile
# Build stage
FROM rust:1.95-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev postgresql-dev
WORKDIR /app
COPY . .
RUN cargo build --release --bin api

# Runtime stage — distroless
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/api /api
EXPOSE 8080
ENTRYPOINT ["/api"]
```

**Nota:** Rust 1.95.0 es la última estable al 16 de abril de 2026. La imagen `rust:1.86-alpine` del ADR 0013 original quedó obsoleta (1.86 = abril 2025). Se actualiza a `rust:1.95-alpine` para aprovechar mejoras de compilación, trait upcasting y soporte de plataformas. La imagen `gcr.io/distroless/cc-debian12` sigue siendo la referencia estándar para runtimes distroless con libc/libgcc (~32MB).

---

# Configuración recomendada en Coolify

## Variables de entorno (API)

```env
SERVER_PORT=8080
ENVIRONMENT=production
RUST_LOG=info

# PostgreSQL
DATABASE_URL=postgres://redes:${DB_PASSWORD}@postgres:5432/redes
DB_PASSWORD=<generado-por-coolify-o-secrets>

# Auth
PASETO_SECRET=<32-bytes-hex>

# Email
RESEND_API_KEY=...
MAIL_FROM=noreply@tudominio.gob.bo

# Storage S3 (opcional para assets)
AWS_ENDPOINT_URL_S3=...
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
STORAGE_BUCKET=redes-assets

# Monitoreo tareas
HC_API_KEY=...
HC_PING_URL=...

# Sentry (opcional)
SENTRY_DSN=...
```

## Variables de entorno (PostgreSQL)

```env
POSTGRES_USER=redes
POSTGRES_PASSWORD=<mismo-que-DB_PASSWORD>
POSTGRES_DB=redes
PGDATA=/var/lib/postgresql/data
```

---

# Deploy Flow

## Kamal (principal MVP)

```text
Laptop → SSH → Build → Push Registry → Deploy VPS
         ↓
    just deploy ✓
    kamal rollback ✓
```

## Coolify (alternativa)

```text
Git Push → Webhook → Auto Build → Deploy Container
              ↓
         Coolify Dashboard
         (logs, rollback, SSL, env vars)
```

---

# Comparativa — Kamal vs Coolify

| Aspecto             | Kamal                          | Coolify                        |
| ------------------- | ------------------------------ | ------------------------------ |
| Filosofía           | Minimalista, CLI-first         | PaaS visual, dashboard         |
| Interfaz            | CLI (`just deploy`)            | Dashboard web + CLI            |
| RAM extra           | ~0MB (solo contenedores app)   | ~500MB-1GB (Coolify + Traefik + workers) |
| SSL                 | Caddy 2.11.3 manual (ADR 0013) | Automático (Traefik 3.7.1)     |
| Rollback            | CLI (`kamal rollback`)         | Visual + CLI                   |
| Multi-app           | Manual (múltiples configs)     | Integrado nativo               |
| PostgreSQL          | ✅ Contenedor Docker manual     | ✅ Servicio gestionado o contenedor |
| Backup DB           | Manual (pg_dump + cron)        | Snapshots de volumen + manual  |
| VPS ideal           | $5 (1GB RAM)                   | $10+ (2GB RAM)                 |
| Complejidad         | Baja                           | Media                          |
| Dependencias        | Docker + SSH                   | Docker + Coolify               |
| Stack Rust/Axum     | ✅ Compatible                   | ✅ Compatible                   |
| Reverse Proxy       | Caddy 2.11.3                   | Traefik 3.7.1 (gestionado)    |

---

# Cuándo usar Coolify

## Elegir Coolify si

* se manejan múltiples proyectos en el mismo VPS
* se quiere dashboard visual para deploys y logs
* el equipo no domina SSH/CLI
* se desea auto-deploy desde Git
* el VPS tiene ≥2GB RAM
* se necesita SSL automático sin configurar Caddy manualmente

---

# Cuándo mantener Kamal

## Mantener Kamal si

* el VPS es de 1GB RAM (mínimo para MVP)
* se prioriza simplicidad extrema
* se quiere mínimo consumo posible
* el equipo ya domina terminal y SSH
* existe solo un proyecto (Monitoreo Regional)
* se prefiere control total del Containerfile y Caddy 2.11.3

---

# Requisitos del VPS

| Escenario | Recomendación     | Notas |
| --------- | ----------------- | ----- |
| Kamal     | VPS $5 — 1GB RAM  | Stack mínimo: API + PostgreSQL contenedor |
| Coolify   | VPS $10 — 2GB RAM | Coolify + Traefik 3.7.1 + API + PostgreSQL |

Coolify ejecuta múltiples contenedores internos:

* Traefik 3.7.1 (reverse proxy)
* PostgreSQL (si se gestiona internamente)
* Redis (opcional, para cache)
* workers (Coolify internals)
* scheduler (Coolify internals)

Esto consume memoria adicional.

---

# Seguridad

## Reglas obligatorias

* Nunca exponer PostgreSQL fuera del contenedor (puerto 5432 solo interno)
* `/var/lib/postgresql/data` solo accesible internamente
* Secrets únicamente en variables cifradas (Coolify Secrets o .env cifrado)
* HTTPS obligatorio (Traefik/Caddy auto-SSL)
* Healthcheck obligatorio (`/health`)
* Backups automáticos activados (pg_dump cron o Apalis job)
* Fail2ban 1.1.0 en el VPS host para protección SSH

---

# Integración con el stack existente

Coolify es compatible con:

| ADR                            | Compatibilidad | Notas |
| ------------------------------ | -------------- | ----- |
| ADR 0001 — Arquitectura Hexagonal | ✅              | Sin cambios |
| ADR 0003 — Axum                | ✅              | Sin cambios |
| ADR 0004 — PostgreSQL          | ✅              | Servicio gestionado o contenedor |
| ADR 0008 — PASETO Auth         | ✅              | Sin cambientos |
| ADR 0013 — Docker Compose      | ✅              | Coolify orquesta contenedores Docker |
| ADR 0015 — Apalis Jobs         | ✅              | Requiere PostgreSQL para queue storage |
| ADR 0016 — OpenAPI             | ✅              | Sin cambios |
| ADR 0017 — SvelteKit Frontend  | ✅              | Deploy separado o mismo Coolify instance |
| ADR 0020 — Monitoreo Regional  | ✅              | Sin cambios |

---

# Alternativas consideradas

| Opción     | Motivo de descarte                |
| ---------- | --------------------------------- |
| Dokploy    | Ecosistema menos maduro           |
| CapRover   | Arquitectura más antigua          |
| Heroku     | Vendor lock-in, costo escalable   |
| Railway    | No self-hosted                    |
| Fly.io     | Excelente pero no self-hosted     |
| Kubernetes | Overkill absoluto para este stack |

---

# Herramientas y Librerías para Optimizar (Edición 2026-05-16)

| Herramienta       | Versión / Referencia | Propósito                                   |
| ----------------- | ------------------- | ------------------------------------------- |
| `Coolify`         | v4+ (latest stable) | Orquestación visual self-hosted             |
| `Traefik`         | 3.7.1               | SSL automático y reverse proxy (Coolify)    |
| `Caddy`           | 2.11.3              | SSL automático y reverse proxy (Kamal)      |
| `pgBackRest`      | 2.58.0              | Backups físicos avanzados de PostgreSQL     |
| `Healthchecks.io` | SaaS (latest)       | Monitoreo de jobs y backups (ADR 0014)      |
| `Dozzle`          | Docker latest       | Visualización ligera de logs Docker         |
| `ctop`            | Docker latest       | Monitoreo de recursos en tiempo real        |
| `glances`         | Docker latest       | Observabilidad del VPS                      |
| `fail2ban`        | 1.1.0               | Protección SSH y reverse proxy              |
| `Rust`            | 1.95.0              | Toolchain de compilación (Containerfile)    |
| `distroless`      | cc-debian12         | Imagen runtime mínima (~32MB)               |

**Nota:** Se elimina `Litestream` y `Watchtower` de la lista. Litestream es para SQLite (no usado en servidor). Watchtower no es necesario con Coolify (gestiona updates nativamente) ni con Kamal (build explícito). Se actualiza Rust de 1.86 a 1.95.0 (latest estable al 16 abr 2026). Se fijan versiones de Caddy 2.11.3, Traefik 3.7.1 y fail2ban 1.1.0. Se agrega advertencia sobre sostenibilidad de pgBackRest.

---

# Consecuencias

## ✅ Positivas

* Dashboard visual para deploys, logs y métricas
* SSL automático sin configurar Caddy manualmente
* Rollbacks visuales con un click
* Multi-proyecto en un solo VPS
* Deploy automático vía Git webhook
* PostgreSQL funciona sin cambios arquitectónicos
* No se rompe la arquitectura hexagonal ni el stack Rust
* Compatible con Apalis jobs (PostgreSQL queue)

---

## ⚠️ Negativas / Trade-offs

### Mayor consumo de RAM

Coolify agrega múltiples contenedores internos (Traefik 3.7.1, workers, scheduler).

→ Mitigación:

* usar VPS de 2GB RAM mínimo
* swap activo según necesidad
* monitorear con `glances` o `ctop`

---

### Mayor complejidad operativa

Más componentes gestionados:

* Traefik 3.7.1 (reverse proxy)
* PostgreSQL (si gestionado por Coolify)
* Redis (opcional)
* workers Coolify

→ Mitigación:

* mantener arquitectura del app extremadamente simple
* un solo contenedor de API Rust distroless
* PostgreSQL como servicio dedicado o contenedor separado
* documentar procedimientos de backup/restore

---

### Menos minimalista que Kamal

Kamal sigue siendo más eficiente para:

* un solo proyecto
* VPS pequeños ($5, 1GB RAM)
* equipos técnicos que dominan CLI

Coolify brilla cuando existen múltiples aplicaciones o el equipo prefiere UI.

---

# Decisiones derivadas

* `DATABASE_URL` apunta siempre a **PostgreSQL**, nunca a SQLite
* **No se usa Litestream** en el servidor — es herramienta de SQLite, no PostgreSQL
* El Containerfile distroless no incluye Litestream y usa **Rust 1.95.0** como base
* **Kamal sigue siendo la opción principal del MVP** (ROADMAP-MASTER.md Día 9)
* Coolify es **alternativa oficialmente soportada** para escenarios multi-proyecto o equipos no técnicos
* VPS de 1GB RAM → **Kamal recomendado**
* VPS de 2GB+ RAM → **Coolify válido**
* El deploy nunca depende de Kubernetes
* PostgreSQL es la base oficial del proyecto (ADR 0004)
* Los backups se realizan con `pg_dump` + cron o jobs Apalis; pgBackRest 2.58.0 como opción avanzada con monitoreo de su roadmap
* SQLite Wasm solo en frontend para Local-First (ADR 0021)
* Coolify gestiona SSL vía Traefik 3.7.1; Kamal gestiona SSL vía Caddy 2.11.3
* Healthcheck `/health` obligatorio en ambos escenarios de deploy
* Fail2ban 1.1.0 protege el VPS host en ambos escenarios

---

# Historial de cambios

| Versión | Fecha       | Cambios realizados |
| ------- | ----------- | ------------------ |
| 1.0     | 2026 (orig) | Versión inicial con SQLite (`boilerplate.db`), Litestream, `DATABASE_URL=sqlite:/data/boilerplate.db`, backup vía Litestream/S3 |
| 2.0     | 2026-05-16  | Corrige base de datos a PostgreSQL (ADR 0004); elimina Litestream del Containerfile y arquitectura; reemplaza backup Litestream/S3 por `pg_dump`/`pgBackRest`; actualiza `DATABASE_URL` a formato PostgreSQL; actualiza comparativa Kamal vs Coolify; actualiza herramientas (elimina Litestream/Watchtower); agrega notas sobre SQLite Wasm solo en frontend (ADR 0021); clarifica Kamal como deploy principal MVP y Coolify como alternativa |
| 2.1     | 2026-05-16  | Actualiza Rust de 1.86 a **1.95.0** en Containerfile; fija versiones: Caddy **2.11.3**, Traefik **3.7.1**, fail2ban **1.1.0**, pgBackRest **2.58.0**; agrega advertencia sobre sostenibilidad de pgBackRest; actualiza tabla de herramientas con versiones exactas |
