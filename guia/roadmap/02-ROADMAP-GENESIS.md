# Roadmap — Génesis (Arranque del Proyecto)

> **Última revisión de versiones:** 2026-05-16  
> Se actualizaron las versiones de herramientas y toolchain tras auditoría contra rust-lang.org, crates.io, nodejs.org y docs.rs.
>
> **Objetivo:** directorio vacío → monorepo funcional con crates declarados,
> herramientas instaladas y `cargo check --workspace` pasando limpio.
>
> **Referencia Principal:** ADR 0020 (Módulo de Monitoreo de Infraestructura Regional)
> **Referencias:** ADR 0001, ADR 0002, ADR 0003, ADR 0012, ADR 0017, ADR 0015, ADR 0016, ADR 0021, ADR 0022
> **Estimado:** 1-2 días de trabajo

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
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
    [ ] rust = "1.95"  ← Última estable verificada (Edition 2024 soportada desde 1.85+)
    [ ] node = "24"    ← LTS estable para producción (Node 26 es Current)
    [ ] pnpm = "10"    ← Última estable: 10.27.0
    [ ] just = "1.51"  ← Última estable: 1.51.0 (may 2026)

rust-toolchain.toml en la raíz
    └─ Ref: https://rust-lang.github.io/rustup/overrides.html
    [ ] channel = "1.95.0"  ← Última stable (abr 2026)
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
    [ ] crates/inventory/     ← Ref: ADR 0020 (Inventario físico de dispositivos)
    [ ] crates/monitoring/    ← Ref: ADR 0014 (Healthchecks), ADR 0020
    [ ] crates/jobs/          ← Ref: ADR 0015 (Apalis), ADR 0020
    [ ] crates/sync/          ← Ref: ADR 0021 (Local-First Sync Offline)
    [ ] crates/snmp/          ← Ref: ADR 0020 (Monitoreo red)
    [ ] crates/topology/      ← Ref: ADR 0020 (Topología red)
    [ ] crates/storage/       ← Ref: ADR 0020 (Almacenamiento S3/assets)

Crear carpetas de apps:
    [ ] apps/api/             ← Ref: ADR 0003 (Axum)
    [ ] apps/web/             ← Ref: ADR 0017 (SvelteKit + Svelte 5)
    [ ] apps/agent/           ← Ref: ADR 0022 (Agente de monitoreo distribuido)

Crear carpetas de infraestructura:
    [ ] infra/docker/         ← Ref: ADR 0013 (Docker Compose)
    [ ] infra/coolify/        ← Ref: ADR 0019 (Coolify)

Crear carpetas de datos:
    [ ] data/migrations/      ← Ref: ADR 0005
    [ ] data/seeds/           ← Ref: ADR 0005
    [ ] data/assets/         ← Ref: ADR 0020 (snmp mibs)

pnpm-workspace.yaml  (packages: apps/web)
    └─ Ref: ADR 0017
    [ ] packages: ["apps/web"]

README.md en la raíz
    └─ Copiar resumen de arquitectura

```

**Inicialización mínima de crates Rust:**
```bash
# Inicializar cada crate como librería (lib.rs vacío mínimo)
cargo init --lib crates/domain
cargo init --lib crates/application
cargo init --lib crates/infrastructure
cargo init --lib crates/database
cargo init --lib crates/auth
cargo init --lib crates/inventory
cargo init --lib crates/monitoring
cargo init --lib crates/jobs
cargo init --lib crates/sync
cargo init --lib crates/snmp
cargo init --lib crates/topology
cargo init --lib crates/storage

# Inicializar apps
cargo init --bin apps/api
cargo init --bin apps/agent

# Inicializar frontend
pnpm create svelte@latest apps/web  # o scaffolding según ADR 0017
```

**Verificación G.1:** `ls -la` muestra la estructura completa y `cargo check --workspace` no falla por crates vacíos.

---

## G.2 — Cargo.toml por crate

> **Referencia:** ADR 0001, ADR 0002, ADR 0003, ADR 0020

Cada crate declara SOLO sus dependencias directas. El compilador hace cumplir las fronteras.

```
Workspace root Cargo.toml — [workspace.dependencies] centralizado
    [ ] Todas las dependencias en [workspace.dependencies]
    [ ] members list completo: crates/*, apps/*
    [ ] resolver = "2"

[profile.release] en workspace root:
    [ ] opt-level = 3           ← Rendimiento para backend de monitoreo en tiempo real
    [ ] lto = true
    [ ] codegen-units = 1
    [ ] panic = "abort"
    [ ] strip = true
    [ ] incremental = false

[profile.release-size]      ← Perfil opcional para apps/agent si se necesita bin ligero
    [ ] inherits = "release"
    [ ] opt-level = "z"
    [ ] lto = true
    [ ] codegen-units = 1
    [ ] panic = "abort"
    [ ] strip = true

crates/domain/Cargo.toml
    └─ Ref: ADR 0001 — domain SIN dependencias externas
    [ ] edition = "2024"
    [ ] thiserror, uuid, time, serde
    [ ] VERIFICAR: grep -r "sqlx" crates/domain/ --include="*.toml" → cero resultados
    [ ] VERIFICAR: grep -r "axum" crates/domain/ --include="*.toml" → cero resultados

crates/application/Cargo.toml
    └─ Ref: ADR 0001
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }

crates/database/Cargo.toml
    └─ Ref: ADR 0004, ADR 0020 (métricas)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] sqlx 0.8.6, moka 0.12.15

crates/auth/Cargo.toml
    └─ Ref: ADR 0008 (PASETO - JWT prohibido)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] argon2 0.5.3, pasetors 0.7.8, secrecy 0.10.3
    [ ] VERIFICAR: grep -r "jsonwebtoken" . --include="*.toml" → cero resultados

crates/inventory/Cargo.toml
    └─ Ref: ADR 0020 (Inventario físico de dispositivos)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }

crates/storage/Cargo.toml
    └─ Ref: ADR 0020
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] aws-config, aws-sdk-s3

crates/monitoring/Cargo.toml
    └─ Ref: ADR 0014 (Healthchecks), ADR 0020
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] reqwest 0.13, tracing

crates/jobs/Cargo.toml
    └─ Ref: ADR 0015 (Apalis), ADR 0020
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] apalis 1.0.0-rc.7, async-trait

crates/sync/Cargo.toml
    └─ Ref: ADR 0021 (Local-First Sync Offline)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] tokio 1.52, serde

crates/snmp/Cargo.toml
    └─ Ref: ADR 0020 (Monitoreo red)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }
    [ ] snmp, tokio 1.52

crates/topology/Cargo.toml
    └─ Ref: ADR 0020 (Topología)
    [ ] edition = "2024"
    [ ] domain = { path = "../domain" }

crates/infrastructure/Cargo.toml
    └─ Ref: ADR 0003 (Axum)
    [ ] edition = "2024"
    [ ] application, database, auth, storage
    [ ] inventory, monitoring, jobs, sync, snmp, topology
    [ ] axum 0.8, utoipa 5.4, tower 0.5.3, tower-http 0.6.10

apps/api/Cargo.toml
    └─ Ref: ADR 0003, ADR 0020
    [ ] edition = "2024"
    [ ] infrastructure, database, auth, storage
    [ ] domain, application

apps/agent/Cargo.toml
    └─ Ref: ADR 0022 (Agente de monitoreo distribuido)
    [ ] edition = "2024"
    [ ] snmp, sync, monitoring, tokio 1.52, reqwest 0.13

apps/cli/Cargo.toml
    🟡 Fase 2 — Definir en roadmap futuro (CLI/Sintonía)
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
│crates/database│  │crates/auth  crates/storage            │
│(domain + sqlx)│  │crates/inventory  crates/monitoring     │
│└─ ADR 0004   │  │crates/jobs  crates/sync               │
└──────────────┘  │crates/snmp  crates/topology            │
                  └──────────────────┬───────────────────────┘
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
    [ ] cargo install sqlx-cli --features postgres,sqlite  ← PostgreSQL principal + SQLite para Local-First
    [ ] cargo install lefthook
    # just ya está gestionado por mise.toml (verificar con mise doctor)

    [ ] npm install -g pnpm

Verificar:
    [ ] mise doctor → toolchain completo
    [ ] just --version → confirmar instalación
```

---

## G.4 — justfile y comandos

> **Referencia:** ADR 0012

```
justfile en la raíz con todos los comandos:
    [ ] doctor      (verifica toolchain: mise doctor + cargo --version + pnpm --version)
    [ ] setup       (instala todo + lefthook install + cp .env.example .env.local)
    [ ] dev         (desarrollo completo: api + web en paralelo)
    [ ] dev-api     (solo backend: cargo watch -x check -x run --bin api)
    [ ] build       (cargo build --release)
    [ ] build-agent (cargo build --release --bin agent --profile release-size)
    [ ] test        (cargo nextest run)
    [ ] lint        (cargo clippy -D warnings)
    [ ] fmt         (cargo fmt --all)
    [ ] check       (cargo check --workspace)
    [ ] audit       (cargo deny check + cargo audit)
    [ ] migrate     (sqlx migrate run)
    [ ] migrate-reset
    [ ] db-status
    [ ] prepare     (cargo sqlx prepare --workspace)
    [ ] types       🟡 Fase 2 (buf generate / OpenAPI types)
    [ ] deploy      🟡 Fase 2 (coolify deploy)

lefthook.yml:
    [ ] pre-commit: cargo fmt --all --check
    [ ] pre-push: cargo clippy + cargo nextest run + cargo deny check

deny.toml:
    [ ] license-check: permitir MIT, Apache-2.0, BSD, ISC
    [ ] deny: GPL, AGPL, jsonwebtoken ← Ref: ADR 0008
    [ ] vulnerability-check: deny

.env.example con TODAS las variables:
    [ ] SERVER_PORT, ENVIRONMENT, RUST_LOG
    [ ] DATABASE_URL=postgres://user:pass@localhost:5432/redes  ← PostgreSQL principal
    [ ] SQLITE_URL=file:./data/local.db  ← SQLite para Local-First (ADR 0021)
    [ ] PASETO_SECRET
    [ ] RESEND_API_KEY, MAIL_FROM
    [ ] AWS_ENDPOINT_URL_S3, AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY
    [ ] STORAGE_BUCKET
    [ ] HC_API_KEY, HC_PING_URL  ← Healthchecks.io (ADR 0014)
    [ ] SENTRY_DSN (opcional)
```

---

## G.5 — Verificaciones de primera sintonía

> **Referencia:** ADR 0011 (Estándares), ADR 0001, ADR 0008

```
[ ] cargo check --workspace → cero errores
[ ] cargo deny check → sin violations
[ ] grep -r "jsonwebtoken" . --include="*.toml" → cero resultados
[ ] grep -r "sqlx" crates/domain/ --include="*.toml" → cero resultados
[ ] grep -r "axum" crates/domain/ --include="*.toml" → cero resultados
[ ] just --list → comandos visibles
[ ] lefthook install → hooks activos
```

**Nota:** `cargo nextest run --workspace` y `buf lint` se verifican en fases posteriores
(Backend I / Protocolos) cuando existan tests y archivos `.proto`.

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
Agentes/Sensores Locales (apps/agent) ← Ref: ADR 0022
    ↓
API Axum (apps/api) ← Ref: ADR 0003
    ↓
Jobs de procesamiento (crates/jobs - Apalis) ← Ref: ADR 0015
    ↓
PostgreSQL + métricas históricas (crates/database) ← Ref: ADR 0004
    ↓
Dashboard SvelteKit realtime (apps/web) ← Ref: ADR 0017
```

### Componentes del Módulo (ADR 0020)

| Componente  | Responsabilidad                   | Crate |
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

## Stack Tecnológico (basado en ADRs actualizados)

| Componente | Tecnología | ADR |
|------------|------------|-----|
| Backend | Rust 1.95 + Axum 0.8 | ADR 0003 |
| Frontend | SvelteKit 2.57 + Svelte 5.55 | ADR 0017 |
| DB | PostgreSQL 17.10 + SQLite (Local-First) | ADR 0004, ADR 0021 |
| Deploy | Coolify | ADR 0019 |
| Jobs | Apalis 1.0.0-rc.7 | ADR 0015 |
| Mail | Resend | ADR 0016 |
| Monitoreo tareas | Healthchecks.io | ADR 0014 |
| Realtime | SSE | ADR 0017 |
| API Docs | OpenAPI + Utoipa 5.4 | ADR 0016 |
| Auth | PASETO v4 + argon2id | ADR 0008 |
| Agentes distribuidos | Rust ligero + SNMP | ADR 0022 |
| Sync offline | SQLite Wasm + sync queue | ADR 0021 |
| Toolchain | mise 2026.5, just 1.51.0, pnpm 10.27 | ADR 0012 |
| Testing | cargo-nextest 0.9.135, Vitest 4.1, Playwright 1.59 | ADR 0010, ADR 0017 |

---

## Documentación de Referencia

| ADR | Tema |
|-----|------|
| ADR 0001 | Arquitectura Hexagonal |
| ADR 0002 | Configuración Tipeada |
| ADR 0003 | Stack Backend (Rust 1.95 + Axum 0.8) |
| ADR 0004 | Persistencia PostgreSQL 17.10 |
| ADR 0005 | Migraciones y Seeding |
| ADR 0006 | RBAC, Sessions, Audit |
| ADR 0007 | Manejo de Errores |
| ADR 0008 | Seguridad Auth PASETO |
| ADR 0009 | Rate Limiting |
| ADR 0010 | Testing y Calidad |
| ADR 0011 | Estándares de Desarrollo |
| ADR 0012 | Herramientas de Desarrollo |
| ADR 0013 | Infraestructura Docker Compose |
| ADR 0014 | Monitoreo Tareas Críticas (Healthchecks) |
| ADR 0015 | Jobs Asíncronos Apalis |
| ADR 0016 | Documentación OpenAPI + Mailer Resend |
| ADR 0017 | Frontend SvelteKit 2.57 + Svelte 5.55 |
| ADR 0018 | Sintonía CLI |
| ADR 0019 | Coolify Deploy |
| ADR 0020 | Monitoreo de Infraestructura Regional |
| ADR 0021 | Local-First Sync Offline |
| ADR 0022 | Agentes Monitoreo Distribuidos |

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
| `just: command not found` | just no instalado | `mise install` o `cargo install just` |
| `.env.local` no existe | No copiado desde ejemplo | `cp .env.example .env.local` |
| `rustup: target not found 1.95.0` | Versión inexistente | Actualizar rustup: `rustup self update` |
| `crate inventory not found` | Carpeta no creada | Crear `crates/inventory/` y `cargo init --lib` |
| `sqlx-cli: no postgres feature` | Instalado sin feature correcta | Reinstalar: `cargo install sqlx-cli --features postgres,sqlite` |
| `pasetors compile error` | MSRV insuficiente | Verificar `rustc --version` ≥ 1.88 (pasetors 0.7.8 requiere 1.88+) |
| `reqwest 0.13` no compila | Feature TLS incorrecta | Usar `rustls` (no `rustls-tls`) en reqwest 0.13 |

---

## Registro de cambios de versiones

| Fecha | Componente | Anterior | Actual | Notas |
|-------|------------|----------|--------|-------|
| 2026-05-16 | Rust toolchain | 1.86.0 | **1.95.0** | Última stable (abr 2026). Edition 2024 soportada. |
| 2026-05-16 | mise | 2026.x | **2026.5** | v2026.5.9 (15 may 2026) |
| 2026-05-16 | just | 1.40 | **1.51.0** | Última estable (may 2026) |
| 2026-05-16 | pnpm | 10 | **10.27** | Última estable (dic 2025) |
| 2026-05-16 | Node.js | 24 | **24** (LTS) | Node 24 LTS estable. Node 26 Current para dev. |
| 2026-05-16 | sqlx | 0.8.5 | **0.8.6** | Patch release |
| 2026-05-16 | moka | 0.12 | **0.12.15** | Patch release |
| 2026-05-16 | argon2 | 0.5 | **0.5.3** | Última estable |
| 2026-05-16 | pasetors | 0.7 | **0.7.8** | Última estable (feb 2026). MSRV 1.88. |
| 2026-05-16 | secrecy | 0.10 | **0.10.3** | Última estable |
| 2026-05-16 | reqwest | 0.12 | **0.13** | Breaking change: `rustls-tls` → `rustls` |
| 2026-05-16 | tokio | 1.45 | **1.52** | Runtime actualizado |
| 2026-05-16 | tower | 0.5.2 | **0.5.3** | Última estable |
| 2026-05-16 | tower-http | 0.6.2 | **0.6.10** | Patches de seguridad |
| 2026-05-16 | utoipa | 5 | **5.4** | Nuevas features |
| 2026-05-16 | apalis | 1.0.0-rc.9 | **1.0.0-rc.7** | rc.9 no existe. Última real: rc.7 |
| 2026-05-16 | Svelte | 5.45 | **5.55.0** | Última estable |
| 2026-05-16 | SvelteKit | 2.x | **2.57.0** | Última estable |
| 2026-05-16 | Vitest | 3.x | **4.1** | Última estable |
| 2026-05-16 | Playwright | 1.x | **1.59** | Última estable |

---

**Nota:** Este roadmap está basado en el ADR 0020 que define el proyecto completo de Monitoreo de Infraestructura Regional para la Gobernación del Beni.
