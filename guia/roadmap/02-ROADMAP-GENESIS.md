# Roadmap — Génesis (Arranque del Proyecto)

> **Objetivo:** directorio vacío → monorepo funcional con crates declarados,
> herramientas instaladas y `cargo check --workspace` pasando limpio.
>
> **Referencia Principal:** ADR 0020 (Módulo de Monitoreo de Infraestructura Regional)
> **Referencias:** ADR 0001, ADR 0002, ADR 0003, ADR 0012, ADR 0017, ADR 0015, ADR 0016
> **Estimado:** 1-2 días de trabajo

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
🟡 Fase 2      🔴 Fase 3

Leyenda de Fases:
• Fase 1 (MVP)   → Funcionalidad core — implementar ahora
• 🟡 Fase 2      → Diferida — implementar SOLO cuando el problema exista
• 🔴 Fase 3      → Escalamiento futuro — no implementar sin criterio medido
```

---

## Progreso

| Fase | Nombre | Progreso |
|------|--------|----------|
| G.1 | Estructura física del workspace | [ ] |
| G.2 | Cargo.toml workspace y crates | [ ] |
| G.3 | Configuración de herramientas | [ ] |
| G.4 | justfile y comandos | [ ] |
| G.5 | Verificaciones iniciales | [ ] |
| **Total Génesis** | | [ ] |

---

## G.1 — Estructura física del workspace

> **Referencia:** ADR 0020 (Monitoreo Regional), ADR 0001 (Arquitectura Hexagonal), ADR 0003 (Axum), ADR 0017 (SvelteKit)

El proyecto es un **Monitoreo de Infraestructura Regional** para la Gobernación del Beni:

* monitorear infraestructura de red institucional
* visualizar sedes regionales
* detectar fallas rápidamente
* identificar dispositivos no autorizados
* analizar consumo de ancho de banda
* generar auditorías y reportes técnicos

Debe operar con **Local-First** tolerando:
* latencia alta
* cortes de internet
* sincronización diferida
* operación offline parcial

```
mise.toml en la raíz (toolchain management)
    └─ Ref: ADR 0012, https://mise.jdx.dev
    [ ] rust = "1.95"  ← Versión 2026
    [ ] node = "24"
    [ ] pnpm = "10"
    [ ] just = "1.40"

rust-toolchain.toml en la raíz
    └─ Ref: https://rust-lang.github.io/rustup/overrides.html
    [ ] channel = "1.95.0"
    [ ] components = ["rustfmt", "clippy", "rust-analyzer"]
    [ ] targets = ["x86_64-unknown-linux-musl"]
    [ ] profile = "minimal"

.gitignore  (Rust, Node, SQLite, .env.local, /data, /target)
    └─ Ref: ADR 0012 (Herramientas), ADR 0002 (Config)

Cargo.toml workspace root con resolver = "2"
    └─ Ref: ADR 0001 (Arquitectura Hexagonal)

Crear carpetas de crates:
    [ ] crates/domain/        ← Ref: ADR 0001
    [ ] crates/application/   ← Ref: ADR 0001
    [ ] crates/infrastructure/← Ref: ADR 0003 (Axum)
    [ ] crates/database/      ← Ref: ADR 0004 (PostgreSQL), ADR 0020
    [ ] crates/auth/          ← Ref: ADR 0008 (PASETO)
    [ ] crates/mailer/        ← Ref: ADR 0016 (Resend)
    [ ] crates/storage/       ← Ref: ADR 0020 (Tigris)
    [ ] crates/monitoring/    ← Ref: ADR 0015 (Healthchecks), ADR 0020
    [ ] crates/jobs/          ← Ref: ADR 0015 (Apalis), ADR 0020
    [ ] crates/sync/          ← Ref: ADR 0020
    [ ] crates/snmp/          ← Ref: ADR 0020 (Monitoreo red)
    [ ] crates/topology/      ← Ref: ADR 0020 (Topología red)
    [ ] crates/events/        🟡 Fase 2 — Ref: ADR 0015 (Jobs)

Crear carpetas de apps:
    [ ] apps/api/             ← Ref: ADR 0003 (Axum)
    [ ] apps/web/             ← Ref: ADR 0017 (SvelteKit + Svelte 5)
    [ ] apps/mailer/          ← Ref: ADR 0016 (Resend)
    [ ] apps/agent/           ← Ref: ADR 0020 (Agente de monitoreo)
    [ ] apps/cli/             🟡 Fase 2 — Ref: ADR 0018

Crear carpetas de infraestructura:
    [ ] infra/docker/         ← Ref: ADR 0013, ADR 0014
    [ ] infra/caddy/          ← Ref: ADR 0014
    [ ] infra/coolify/       ← Ref: ADR 0019 (Coolify)
    [ ] infra/kamal/          ← Ref: ADR 0014
    [ ] infra/scripts/        ← Ref: ADR 0020 (scripts monitoreo)

Crear carpetas de datos:
    [ ] data/migrations/      ← Ref: ADR 0005
    [ ] data/seeds/           ← Ref: ADR 0005
    [ ] data/assets/         ← Ref: ADR 0020 (snmp mibs)

pnpm-workspace.yaml  (packages: apps/web, apps/mailer)
    └─ Ref: ADR 0017

README.md en la raíz
    └─ Copiar resumen de arquitectura

proto/buf.yaml + proto/buf.gen.yaml + proto/v1/
    🟡 Fase 2 — Ref: ADR 0015 (Jobs)
```

**Verificación G.1:** `ls -la` muestra la estructura completa.

---

## G.2 — Cargo.toml por crate

> **Referencia:** ADR 0001, ADR 0002, ADR 0003, ADR 0020

Cada crate declara SOLO sus dependencias directas. El compilador hace cumplir las fronteras.

```
Workspace root Cargo.toml — [workspace.dependencies] centralizado
    [ ] Todas las dependencias en [workspace.dependencies]
    
[profile.release] en workspace root:
    [ ] opt-level = "z"
    [ ] lto = true
    [ ] codegen-units = 1
    [ ] panic = "abort"
    [ ] strip = true
    [ ] incremental = false

crates/domain/Cargo.toml
    └─ Ref: ADR 0001 — domain SIN dependencias externas
    [ ] edition = "2024"
    [ ] thiserror, uuid, time, serde
    [ ] VERIFICAR: cargo grep "sqlx" crates/domain/ → cero resultados

crates/application/Cargo.toml
    └─ Ref: ADR 0001
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }

crates/database/Cargo.toml
    └─ Ref: ADR 0004, ADR 0020 (métricas)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] sqlx, moka

crates/auth/Cargo.toml
    └─ Ref: ADR 0008 (PASETO - JWT prohibido)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] argon2, pasetors, secrecy
    [ ] VERIFICAR: cargo grep "jsonwebtoken" → cero resultados

crates/mailer/Cargo.toml
    └─ Ref: ADR 0016
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] resend-rs

crates/storage/Cargo.toml
    └─ Ref: ADR 0020
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] aws-config, aws-sdk-s3

crates/monitoring/Cargo.toml
    └─ Ref: ADR 0015 (Healthchecks), ADR 0020
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] reqwest, tracing

crates/jobs/Cargo.toml
    └─ Ref: ADR 0015 (Apalis), ADR 0020
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] apalis, async-trait

crates/sync/Cargo.toml
    └─ Ref: ADR 0020
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] tokio, serde

crates/snmp/Cargo.toml
    └─ Ref: ADR 0020 (Monitoreo red)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] snmp, tokio

crates/topology/Cargo.toml
    └─ Ref: ADR 0020 (Topología)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }

crates/infrastructure/Cargo.toml
    └─ Ref: ADR 0003 (Axum)
    [ ] edition = "2024"
    [ ] application, database, auth, mailer, storage
    [ ] monitoring, jobs, sync, snmp, topology
    [ ] axum, utoipa, tower, tower-http

apps/api/Cargo.toml
    └─ Ref: ADR 0003, ADR 0020
    [ ] edition = "2024"
    [ ] infrastructure, database, auth, mailer, storage
    [ ] domain, application

apps/agent/Cargo.toml
    └─ Ref: ADR 0020 (Agente de monitoreo)
    [ ] edition = "2024"
    [ ] snmp, sync, monitoring, tokio, reqwest

apps/cli/Cargo.toml
    🟡 Fase 2
    [ ] domain, application, clap, tera
```

**Verificación G.2:** `cargo check --workspace` → cero errores.

---

## Diagrama de Dependencias entre Crates

```
┌─────────────────────────────────────────────────────────────┐
│  crates/domain                                              │
│  (thiserror, uuid, time, serde)                            │
│  └─ Ref: ADR 0001                                          │
└──────────────┬──────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│  crates/application                                         │
│  (solo domain)                                            │
└──────────────┬──────────────────────────────────────────────┘
               │
       ┌───────┴───────┐
       ▼               ▼
┌──────────────┐  ┌──────────────────────────────────────────┐
│crates/database│  │crates/auth  crates/mailer  crates/storage│
│(domain + sqlx)│  │crates/monitoring  crates/jobs            │
│└─ ADR 0004   │  │crates/sync  crates/snmp  crates/topology │
└──────────────┘  └──────────────────┬───────────────────────┘
                                    │
                                    ▼
                   ┌──────────────────────────────────────┐
                   │  crates/infrastructure               │
                   │  (application + axum + config)      │
                   │  └─ Ref: ADR 0003                    │
                   └──────────────┬───────────────────────┘
                                   │
                                   ▼
                   ┌──────────────────────────────────────┐
                   │  apps/api                              │
                   │  (ensambla todo)                       │
                   │  └─ Ref: ADR 0003, ADR 0020          │
                   └──────────────────────────────────────┘
```

**Regla:** Flechas suben. Ningún crate importa uno que esté por encima.

---

## G.3 — Tooling

> **Referencia:** ADR 0012 (Herramientas), ADR 0010 (Testing), ADR 0011 (Calidad)

```
Instalar herramientas (versiones actualizadas 2026):
    [ ] cargo install cargo-watch
    [ ] cargo install cargo-nextest
    [ ] cargo install cargo-deny
    [ ] cargo install cargo-audit
    [ ] cargo install sqlx-cli --features sqlite
    [ ] cargo install lefthook
    [ ] cargo install just

    [ ] npm install -g pnpm

Verificar:
    [ ] mise doctor → toolchain completo
```

---

## G.4 — justfile y comandos

> **Referencia:** ADR 0012

```
justfile en la raíz con todos los comandos:
    [ ] doctor      (verifica toolchain)
    [ ] setup       (instala todo + lefthook install)
    [ ] dev         (desarrollo completo)
    [ ] dev-api     (solo backend)
    [ ] build       (cargo build --release)
    [ ] test        (cargo nextest run)
    [ ] lint        (cargo clippy -D warnings)
    [ ] fmt         (cargo fmt --all)
    [ ] check       (cargo check --workspace)
    [ ] audit       (cargo deny check + cargo audit)
    [ ] migrate     (sqlx migrate run)
    [ ] migrate-reset
    [ ] db-status
    [ ] prepare
    [ ] types       🟡 Fase 2 (buf generate)
    [ ] deploy      🟡 Fase 2

lefthook.yml:
    [ ] pre-commit: cargo fmt --all --check
    [ ] pre-push: cargo clippy + cargo nextest run + cargo deny

deny.toml:
    [ ] license-check: permitir MIT, Apache-2.0, BSD, ISC
    [ ] deny: GPL, AGPL, jsonwebtoken ← Ref: ADR 0008
    [ ] vulnerability-check: deny

.env.example con TODAS las variables:
    [ ] SERVER_PORT, ENVIRONMENT, RUST_LOG
    [ ] DATABASE_URL
    [ ] PASETO_SECRET
    [ ] RESEND_API_KEY, MAIL_FROM
    [ ] AWS_ENDPOINT_URL_S3, AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY
    [ ] STORAGE_BUCKET
    [ ] DATABASE_URL (PostgreSQL)
    [ ] HC_* (Healthchecks.io)
    [ ] SENTRY_DSN (opcional)
```

---

## G.5 — Verificaciones de primera sintonía

> **Referencia:** ADR 0011 (Estándares), ADR 0001, ADR 0008

```
[ ] cargo check --workspace → cero errores
[ ] cargo deny check → sin violations
[ ] buf lint → sin errores
[ ] grep -r "jsonwebtoken" . --include="*.toml" → cero resultados
[ ] grep -r "sqlx" crates/domain/ --include="*.toml" → cero resultados
[ ] grep -r "axum" crates/domain/ --include="*.toml" → cero resultados
[ ] just --list → comandos visibles
[ ] cargo nextest run --workspace → tests ejecutan
[ ] lefthook install → hooks activos
```

---

## Arquitectura del Proyecto (ADR 0020)

```
┌─────────────────────────────────────────────────────────────┐
│           Monitoreo de Infraestructura Regional             │
│                    Gobernación del Beni                     │
│                        ADR 0020                             │
└─────────────────────────────────────────────────────────────┘

Sedes Regionales (múltiples)
    ↓
Agentes/Sensores Locales (apps/agent)
    ↓
API Axum (apps/api) ← Ref: ADR 0003
    ↓
Jobs de procesamiento (crates/jobs - Apalis) ← Ref: ADR 0018
    ↓
PostgreSQL + métricas históricas (crates/database) ← Ref: ADR 0004
    ↓
Dashboard SvelteKit realtime (apps/web) ← Ref: ADR 0017
```

### Componentes del Módulo (ADR 0020)

| Componente  | Responsabilidad                   | Crate/App |
| ----------- | --------------------------------- | --------- |
| `inventory` | Inventario físico de dispositivos | crates/inventory |
| `topology`  | Mapeo visual de conexiones        | crates/topology |
| `metrics`   | Métricas de red                   | crates/database |
| `alerts`    | Detección de anomalías            | crates/jobs |
| `audit`     | Bitácora y reportes               | crates/database |
| `agents`    | Recolección distribuida           | apps/agent |
| `sync`      | Sincronización offline            | crates/sync |

### Funcionalidades Principales (ADR 0020)

1. **Inventario físico de dispositivos** - switches, APs, routers, firewalls, servidores, enlaces WAN, UPS, cámaras IP
2. **Topología de red** - mapa por sede, dependencias jerárquicas, estados visuales
3. **Monitoreo de ancho de banda** - tráfico RX/TX, saturación WAN, latencia, packet loss
4. **Detección de dispositivos intrusos** - ARP scan, SNMP, ICMP discovery, whitelist
5. **Alertas y anomalías** - dispositivos offline, saturación, intrusiones

---

## Stack Tecnológico (basado en ADRs)

| Componente | Tecnología | ADR |
|------------|------------|-----|
| Backend | Rust + Axum | ADR 0003 |
| Frontend | SvelteKit + Svelte 5 | ADR 0017 |
| DB | PostgreSQL | ADR 0004 |
| Deploy | Coolify | ADR 0019 |
| Jobs | Apalis | ADR 0015 |
| Mail | Resend + React Email | ADR (futuro) |
| Monitoreo | Healthchecks.io | ADR 0014 |
| Realtime | SSE | ADR 0017 |
| API Docs | OpenAPI + Utoipa | ADR 0016 |
| gRPC | ConnectRPC | ADR 0015 |

---

## Documentación de Referencia

| ADR | Tema |
|-----|------|
| ADR 0020 | Módulo de Monitoreo de Infraestructura Regional |
| ADR 0001 | Arquitectura Hexagonal |
| ADR 0002 | Configuración Tipeada |
| ADR 0003 | Stack Backend (Rust + Axum) |
| ADR 0004 | Persistencia PostgreSQL |
| ADR 0005 | Migraciones y Seeding |
| ADR 0006 | RBAC, Sessions, Audit |
| ADR 0007 | Manejo de Errores |
| ADR 0008 | Seguridad Auth PASETO |
| ADR 0009 | Rate Limiting |
| ADR 0010 | Testing y Calidad |
| ADR 0011 | Estándares de Desarrollo |
| ADR 0012 | Herramientas de Desarrollo |
| ADR 0013 | Build Externo de Binarios |
| ADR 0014 | Infraestructura Docker Compose |
| ADR 0015 | Monitoreo Healthchecks |
| ADR 0016 | Mailer Resend |
| ADR 0017 | (placeholder) |
| ADR 0018 | Jobs Asíncronos Apalis |
| ADR 0019 | Coolify Deploy |
| ADR 0016 | Documentación OpenAPI |
| ADR 0017 | Frontend SvelteKit + Svelte 5 |
| ADR 0018 | Sintonía CLI |
| ADR 0019 | Coolify Deploy |
| ADR 0019 | Coolify Deploy |
| ADR 0020 | Monitoreo de Infraestructura Regional |

---

## ✅ Entregable de Génesis

Cuando todos los checks pasan, el proyecto está listo para el **Bloque I — Fundación**.

```bash
cargo check --workspace   # verde
cargo deny check          # verde
just --list               # muestra todos los comandos
```

**Referencia siguiente fase:** → `ROADMAP-BACKEND.md` — Bloque I (Fundación)

---

## Troubleshooting — Génesis

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `cargo check` error | Cargo.toml malformado | Validar con `cargo verify-project` |
| `unresolved import` en domain | Violación ADR 0001 | Domain no debe tener sqlx/axum |
| `jsonwebtoken` encontrado | Violación ADR 0008 | Usar `pasetors` |
| `duplicate workspace member` | Crate listado 2 veces | Buscar duplicado en `[workspace.members]` |
| `just: command not found` | just no instalado | `cargo install just` |
| `.env.local` no existe | No copiado desde ejemplo | `cp .env.example .env.local` |

---

**Nota:** Este roadmap está basado en el ADR 0020 que define el proyecto completo de Monitoreo de Infraestructura Regional para la Gobernación del Beni.