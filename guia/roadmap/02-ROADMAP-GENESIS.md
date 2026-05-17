# Roadmap — Génesis (Arranque del Proyecto)

> **Última revisión de versiones:** 2026-05-17
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
| G.1 | Estructura física del workspace | [x] |
| G.2 | Cargo.toml workspace y crates | [x] |
| G.3 | Configuración de herramientas | [x] |
| G.4 | justfile y comandos | [x] |
| G.5 | Verificaciones iniciales | [x] |
| **Total Génesis** | | [x] **100%** |

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
    [x] rust = "1.95"  ← Última estable verificada (Edition 2024 soportada desde 1.85+)
    [x] node = "24"    ← LTS estable para producción (Node 26 es Current)
    [x] pnpm = "11.1"    ← Última estable: 11.1.x
    [x] just = "1.51"  ← Última estable: 1.51.0 (may 2026)

rust-toolchain.toml en la raíz
    └─ Ref: https://rust-lang.github.io/rustup/overrides.html
    [x] channel = "1.95.0"  ← Última stable (abr 2026)
    [x] components = ["rustfmt", "clippy", "rust-analyzer"]
    [x] targets = ["x86_64-unknown-linux-musl"]
    [x] profile = "minimal"

.gitignore  (Rust, Node, SQLite, .env.local, /data, /target)
    └─ Ref: ADR 0012 (Herramientas), ADR 0002 (Config)

Cargo.toml workspace root con resolver = "2"
    └─ Ref: ADR 0001 (Arquitectura Hexagonal)

Crear carpetas de crates:
    [x] crates/domain/        ← Ref: ADR 0001
    [x] crates/application/   ← Ref: ADR 0001
    [x] crates/infrastructure/← Ref: ADR 0003 (Axum)
    [x] crates/database/      ← Ref: ADR 0004 (PostgreSQL), ADR 0020
    [x] crates/auth/          ← Ref: ADR 0008 (PASETO)
    [x] crates/inventory/     ← Ref: ADR 0020 (Inventario físico de dispositivos)
    [x] crates/monitoring/    ← Ref: ADR 0014 (Healthchecks), ADR 0020
    [x] crates/jobs/          ← Ref: ADR 0015 (Apalis), ADR 0020
    [x] crates/sync/          ← Ref: ADR 0021 (Local-First Sync Offline)
    [x] crates/snmp/          ← Ref: ADR 0020 (Monitoreo red)
    [x] crates/topology/      ← Ref: ADR 0020 (Topología red)
    [x] crates/storage/       ← Ref: ADR 0020 (Almacenamiento S3/assets)

Crear carpetas de apps:
    [x] apps/api/             ← Ref: ADR 0003 (Axum)
    [x] apps/web/             ← Ref: ADR 0017 (SvelteKit + Svelte 5)
    [x] apps/agent/           ← Ref: ADR 0022 (Agente de monitoreo distribuido)

Crear carpetas de infraestructura:
    [x] infra/docker/         ← Ref: ADR 0013 (Docker Compose)
    [x] infra/coolify/        ← Ref: ADR 0019 (Coolify)

Crear carpetas de datos:
    [x] data/migrations/      ← Ref: ADR 0005
    [x] data/seeds/           ← Ref: ADR 0005
    [x] data/assets/           ← Ref: ADR 0020 (snmp mibs)

pnpm-workspace.yaml  (packages: apps/web)
    └─ Ref: ADR 0017
    [x] packages: ["apps/web"]

README.md en la raíz
    └─ Copiar resumen de arquitectura
```

**Verificación G.1:** `ls -la` muestra la estructura completa y `cargo check --workspace` no falla por crates vacíos.

---

## G.2 — Cargo.toml por crate

> **Referencia:** ADR 0001, ADR 0002, ADR 0003, ADR 0020

Cada crate declara SOLO sus dependencias directas. El compilador hace cumplir las fronteras.

```
Workspace root Cargo.toml — [workspace.dependencies] centralizado
    [x] Todas las dependencias en [workspace.dependencies]
    [x] members list completo: crates/*, apps/*
    [x] resolver = "2"

[profile.release] en workspace root:
    [x] opt-level = 3           ← Rendimiento para backend de monitoreo en tiempo real
    [x] lto = true
    [x] codegen-units = 1
    [x] panic = "abort"
    [x] strip = true
    [x] incremental = false

[profile.release-size]      ← Perfil opcional para apps/agent si se necesita bin ligero
    [x] inherits = "release"
    [x] opt-level = "z"
    [x] lto = true
    [x] codegen-units = 1
    [x] panic = "abort"
    [x] strip = true

crates/domain/Cargo.toml
    └─ Ref: ADR 0001 — domain SIN dependencias externas
    [x] edition = "2024"
    [x] thiserror, uuid, time, serde
    [x] VERIFICAR: grep -r "sqlx" crates/domain/ --include="*.toml" → cero resultados
    [x] VERIFICAR: grep -r "axum" crates/domain/ --include="*.toml" → cero resultados

crates/application/Cargo.toml
    └─ Ref: ADR 0001
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] uuid = { workspace = true }

crates/database/Cargo.toml
    └─ Ref: ADR 0004, ADR 0020 (métricas)
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] sqlx 0.9.0-alpha.1, moka 0.12.15

crates/auth/Cargo.toml
    └─ Ref: ADR 0008 (PASETO - JWT prohibido)
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] base64, rand_core
    [x] ⚠️ PASETO real pendiente - API incompatible con 0.7.8
    [x] ⚠️ Argon2id simplificado - API inestable en 0.6.0-rc.8
    [x] VERIFICAR: grep -r "jsonwebtoken" . --include="*.toml" → cero resultados

crates/inventory/Cargo.toml
    └─ Ref: ADR 0020 (Inventario físico de dispositivos)
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] uuid, time, serde

crates/storage/Cargo.toml
    └─ Ref: ADR 0020
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] aws-config, aws-sdk-s3

crates/monitoring/Cargo.toml
    └─ Ref: ADR 0014 (Healthchecks), ADR 0020
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] reqwest 0.13, tracing

crates/jobs/Cargo.toml
    └─ Ref: ADR 0015 (Apalis), ADR 0020
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] ⚠️ Apalis removido - 1.0.0-rc.9 sin features postgres
    [x] tokio, tracing, uuid, serde

crates/sync/Cargo.toml
    └─ Ref: ADR 0021 (Local-First Sync Offline)
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] tokio 1.52, serde, uuid, time

crates/snmp/Cargo.toml
    └─ Ref: ADR 0020 (Monitoreo red)
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] async-snmp 0.12.0, tokio 1.52

crates/topology/Cargo.toml
    └─ Ref: ADR 0020 (Topología)
    [x] edition = "2024"
    [x] domain = { path = "../domain" }
    [x] uuid, serde

crates/infrastructure/Cargo.toml
    └─ Ref: ADR 0003 (Axum)
    [x] edition = "2024"
    [x] application, database, auth, storage
    [x] inventory, monitoring, jobs, sync, snmp, topology
    [x] axum 0.8, utoipa 5.5, tower 0.5, tower-http 0.6

apps/api/Cargo.toml
    └─ Ref: ADR 0003, ADR 0020
    [x] edition = "2024"
    [x] infrastructure, database, auth, storage
    [x] domain, application

apps/agent/Cargo.toml
    └─ Ref: ADR 0022 (Agente de monitoreo distribuido)
    [x] edition = "2024"
    [x] snmp, sync, monitoring
    [x] tokio, reqwest, tracing, anyhow
```

**Verificación G.2:** `cargo check --workspace` → cero errores (solo warnings esperados).

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
│  (domain + uuid)                                           │
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
                   │  (application + axum + config)       │
                   │  └─ Ref: ADR 0003                    │
                   └──────────────┬───────────────────────┘
                                   │
                                   ▼
                   ┌──────────────────────────────────────┐
                   │  apps/api                              │
                   │  (ensambla todo)                      │
                   │  └─ Ref: ADR 0003, ADR 0020           │
                   └──────────────────────────────────────┘
```

**Regla:** Flechas suben. Ningún crate importa uno que esté por encima.

---

## G.3 — Tooling

> **Referencia:** ADR 0012 (Herramientas), ADR 0010 (Testing), ADR 0011 (Calidad)

```
Instalar herramientas (versiones actualizadas 2026):
    [x] mise.toml define rust = "1.95", node = "24", pnpm = "11.1", just = "1.51"
    [x] lefthook = "2.1.6"
    [x] cargo-deny = "0.19.6"

Verificar:
    [x] mise doctor → toolchain completo (Windows)
    [x] just --version → confirmar instalación
```

---

## G.4 — justfile y comandos

> **Referencia:** ADR 0012

```
justfile en la raíz con todos los comandos:
    [x] doctor      (verifica toolchain: mise doctor + cargo --version + pnpm --version)
    [x] setup       (instala todo + lefthook install + cp .env.example .env.local)
    [x] dev         (desarrollo completo: api + web en paralelo)
    [x] dev-api     (solo backend: cargo watch -x check -x run --bin api)
    [x] build       (cargo build --release)
    [x] build-agent (cargo build --release --bin agent --profile release-size)
    [x] test        (cargo nextest run)
    [x] lint        (cargo clippy -D warnings)
    [x] fmt         (cargo fmt --all)
    [x] check       (cargo check --workspace)
    [x] audit       (cargo deny check + cargo audit)
    [x] migrate     (sqlx migrate run)
    [x] migrate-reset
    [x] db-status
    [x] prepare     (cargo sqlx prepare --workspace)
```

---

## G.5 — Verificaciones iniciales

> **Referencia:** ADR 0012

```
Verificar:
    [x] cargo check --workspace → verde (solo warnings)
    [x] just --list → muestra todos los comandos
    [x] grep -r "jsonwebtoken" . --include="*.toml" → cero resultados
    [x] grep -r "sqlx" crates/domain/ --include="*.toml" → cero resultados
    [x] grep -r "axum" crates/domain/ --include="*.toml" → cero resultados
```

---

## Notas de Implementación

### Versiones Ajustadas (2026-05-17)

| Crate | Original | Ajustado | Razón |
|-------|----------|----------|-------|
| sqlx | 0.8.6 | 0.9.0-alpha.1 | Solo versión disponible en crates.io |
| pnpm | 11.1 | 11.1 | ✅ Sin cambios |
| svelte | 5.55.0 | 5.55.0 | ✅ Sin cambios |
| @sveltejs/kit | 2.57.0 | 2.57.0 | ✅ Sin cambios |
| async-snmp | snmp | async-snmp 0.12.0 | Crate correcto según ADR |
| apalis | 1.0.0-rc.7 | (removido) | Sin features postgres |
| argon2 | 0.5.3 | 0.6.0-rc.8 (simplified) | API inestable |
| pasetors | 0.7.8 | (simplified) | API incompatible con docs |

### Pendientes para Fases Avanzadas

1. **PASETO real**: `pasetors 0.7.8` API no compatible. Requiere investigar.
2. **Argon2id real**: `argon2 0.6.0-rc.8` API inestable. Pendiente versión estable.
3. **Apalis Jobs**: Sin features disponibles actualmente.
4. **Migraciones SQLx**: No creadas aún.
5. **SvelteKit frontend**: Skeleton creado, sin implementación real.

---

## Commit

```
feat(genesis): workspace completo con 12 crates y 3 apps

- Estructura hexagonal: domain → application → infrastructure → apps
- 12 crates: domain, application, database, auth, infrastructure, inventory, monitoring, jobs, sync, snmp, topology, storage
- 3 apps: api (Axum 0.8), web (SvelteKit 5), agent (monitoring distribuido)
- Configuración: mise.toml, rust-toolchain.toml, justfile, lefthook.yml, deny.toml
- JWT prohibido: deny.toml bloquea jsonwebtoken
- Domain limpio: sin sqlx ni axum
- Soft Delete preparado en entidades
- Versiones verificadas contra crates.io (2026-05-17)
```

---

## Siguiente Fase

**Roadmap Backend:** `03-ROADMAP-BACKEND.md`

```
B.1 — Entidades de dominio (Sede, Device, MetricReading, Alert, User, AuditLog)
B.2 — Migraciones SQLx para PostgreSQL
B.3 — Handlers HTTP (CRUD sedes, dispositivos, métricas)
B.4 — OpenAPI con Utoipa + Scalar
B.5 — Middleware auth (PASETO)
B.6 — Background jobs (monitoreo periódico)
B.7 — Healthchecks + Readiness probes
```
