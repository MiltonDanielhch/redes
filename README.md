# Monitoreo de Infraestructura Regional - Gobernación del Beni

Sistema centralizado de monitoreo de infraestructura de red institucional para la Gobernación del Beni, Bolivia.

---

## 🎯 Objetivo

Monitorear la infraestructura de red de las sedes institucionales del Beni:
- Detectar fallas rápidamente
- Identificar dispositivos no autorizados
- Analizar consumo de ancho de banda
- Visualizar topología de red
- Generar auditorías y reportes técnicos

---

## 🏗️ Arquitectura

```
Monitoreo de Infraestructura Regional
├── Backend:    Rust 1.95.0 + Axum + SQLx 0.8.6 + PostgreSQL 17+
├── Frontend:   SvelteKit 2.x + Svelte 5 + TypeScript
├── Auth:       PASETO v4 Local (pasetors 0.7.8) — nunca JWT
├── Jobs:       Apalis 1.0.0-rc.9 (async processing)
├── Deploy:     Kamal (principal MVP) / Coolify (alternativa multi-proyecto)
├── Monitoreo:  Healthchecks.io (SaaS)
├── API Docs:   OpenAPI 3.1 + Utoipa 5.5.0 + Scalar 0.3.0
├── Local-First: @sqlite.org/sqlite-wasm 3.53.0-build1 + SSE realtime
└── Agentes:    Rust 1.95.0 + reqwest 0.13.2 + snmp2 0.5.0 + surge-ping 0.8.4
```

### Stack Tecnológico

| Componente | Tecnología | Versión | ADR |
|------------|------------|---------|-----|
| Backend | Rust + Axum | 1.95.0 / latest | ADR 0003 |
| Frontend | SvelteKit + Svelte 5 | 2.x / 5.x | ADR 0017 |
| DB | PostgreSQL | 17+ | ADR 0004 |
| Deploy Principal | Kamal + Caddy 2.11.3 | latest / 2.11.3 | ADR 0013 |
| Deploy Alternativa | Coolify + Traefik 3.7.1 | latest / 3.7.1 | ADR 0019 |
| Auth | PASETO v4 Local | pasetors 0.7.8 | ADR 0008 |
| Jobs | Apalis | 1.0.0-rc.9 | ADR 0015 |
| API Docs | OpenAPI + Utoipa + Scalar | 5.5.0 / 0.3.0 | ADR 0016 |
| Monitoreo SaaS | Healthchecks.io | latest | ADR 0014 |
| Realtime | SSE (Server-Sent Events) | nativo Axum | ADR 0020 |
| Local-First SQLite | @sqlite.org/sqlite-wasm | 3.53.0-build1 | ADR 0021 |
| Sync Frontend | UUID v7 | uuid@14.0.0 | ADR 0021 |
| Agente SNMP | snmp2 | 0.5.0 | ADR 0022 |
| Agente ICMP | surge-ping | 0.8.4 | ADR 0022 |
| Agente HTTP | reqwest | 0.13.2 | ADR 0022 |
| Agente Config | toml | 0.8.22 | ADR 0022 |
| Observabilidad | tracing + tracing-subscriber | 0.1.44 / 0.3.23 | ADR 0022 |
| Async Runtime | tokio | 1.52.3 (LTS) | ADR 0003, ADR 0022 |
| CLI Generator | Sintonía (clap 4.6.1 + tera 1.20.1) | internal | ADR 0018 |
| Tooling | mise + just + lefthook 2.1.6 | latest / 2.1.6 | ADR 0012 |
| Linting | cargo-deny 0.19.6 + cargo-audit | 0.19.6 | ADR 0010 |
| Tests | cargo-nextest 0.9.135 | 0.9.135 | ADR 0010 |

---

## 📁 Estructura del Proyecto

```
├── crates/                  # Crates de Rust (arquitectura hexagonal)
│   ├── domain/            # Entidades de negocio (sin deps externas)
│   ├── application/        # Casos de uso
│   ├── database/          # Repositorios SQLx 0.8.6
│   ├── auth/              # PASETO v4 (pasetors 0.7.8) + argon2 0.5.3
│   ├── infrastructure/    # Axum + config + utoipa 5.5.0
│   └── sync/              # Sync engine Rust para agentes (ADR 0022)
│
├── apps/                   # Aplicaciones
│   ├── api/               # API REST Axum (apps/api/)
│   ├── web/               # Dashboard SvelteKit (apps/web/)
│   │   └── src/lib/sync/  # Sync engine TypeScript para frontend (ADR 0021)
│   ├── agent/             # Agente de monitoreo distribuido (ADR 0022)
│   └── cli/               # Sintonía CLI generator (ADR 0018)
│
├── guia/                   # Documentación
│   ├── adr/               # Architecture Decision Records (20+ ADRs)
│   ├── roadmap/           # Roadmaps por fase
│   └── PROMPT_MAESTRO.md  # Contexto global del proyecto
│
└── data/                  # Datos
    └── migrations/        # Migraciones PostgreSQL (sqlx migrate)
```

---

## 🚀 Primeros Pasos

### Requisitos

- **Rust** 1.95.0+ (Edition 2024)
- **Node.js** 24+ (para frontend SvelteKit)
- **PostgreSQL** 17+ (base de datos principal)
- **mise** (gestor de herramientas y versiones)
- **just** (command runner)
- **lefthook** 2.1.6 (git hooks)
- **pnpm** (gestor de paquetes Node)

### Instalación

```bash
# 1. Instalar herramientas (mise gestiona versiones)
mise install

# 2. Verificar que compila
cargo check --workspace

# 3. Configurar variables de entorno
cp .env.example .env.local
# Editar DATABASE_URL, PASETO_SECRET, etc.

# 4. Ejecutar migraciones (sqlx migrate run)
just migrate

# 5. Arrancar desarrollo
just dev
```

### Comandos principales

```bash
just dev              # Desarrollo completo (api + web)
just dev-api          # Solo backend Axum
just dev-web          # Solo frontend SvelteKit
just test             # Tests con cargo-nextest 0.9.135
just lint             # Clippy + cargo-deny 0.19.6 check
just fmt              # Formateo (rustfmt + prettier)
just audit            # Security audit (cargo-deny + cargo-audit)
just deploy           # Deploy con Kamal (principal MVP)
just deploy-coolify   # Deploy alternativo con Coolify
```

---

## 📚 Documentación

| Documento | Descripción |
|-----------|-------------|
| `guia/PROMPT_MAESTRO.md` | Contexto global del proyecto |
| `guia/roadmap/ROADMAP-MASTER.md` | Mapa general de fases (v2.1) |
| `guia/roadmap/ROADMAP-GENESIS.md` | Fase de arranque — workspace + tooling |
| `guia/roadmap/ROADMAP-BACKEND.md` | Backend completo con checklist |
| `guia/roadmap/ROADMAP-FRONTEND.md` | Frontend completo con checklist |
| `guia/roadmap/ROADMAP-AUTH-FULLSTACK.md` | Login/Registro back+front |
| `guia/roadmap/ROADMAP-INFRA.md` | Deploy, Caddy 2.11.3, Kamal, pg_dump |
| `guia/roadmap/ROADMAP-MONITOREO.md` | Monitoreo de red (inventario, métricas, topología) |
| `guia/verification_comando.md` | Verificaciones por fase (v2.1) |
| `guia/adr/ADR-0001.md` — `ADR-0022.md` | 22 ADRs activos |

### ADRs Principales

| ADR | Tema | Versión |
|-----|------|---------|
| ADR 0001 | Arquitectura Hexagonal | latest |
| ADR 0003 | Axum + REST | latest |
| ADR 0004 | PostgreSQL | latest |
| ADR 0006 | RBAC + Audit | latest |
| ADR 0008 | PASETO Auth (nunca JWT) | latest |
| ADR 0010 | Testing (cargo-nextest 0.9.135) | latest |
| ADR 0012 | Tooling (mise, just, lefthook 2.1.6) | latest |
| ADR 0013 | Docker + Kamal Deploy | latest |
| ADR 0015 | Jobs (Apalis 1.0.0-rc.9) | latest |
| ADR 0016 | OpenAPI (Utoipa 5.5.0 + Scalar 0.3.0) | v2.1 |
| ADR 0017 | SvelteKit Frontend | latest |
| ADR 0018 | Sintonía CLI (clap 4.6.1 + tera 1.20.1) | v2.1 |
| ADR 0019 | Deploy Coolify Alternativo | v2.1 |
| ADR 0020 | Módulo Monitoreo Regional | v2.1 |
| ADR 0021 | Local-First Sync Offline | v2.1 |
| ADR 0022 | Agentes Monitoreo Distribuidos | v2.1 |

---

## 🔐 Reglas de Arquitectura

1. **domain sin dependencias** — El crate `domain` NO puede importar sqlx, axum, reqwest, etc. Solo `thiserror`, `uuid`, `time`, `serde`.
2. **JWT prohibido** — Solo PASETO v4 Local (tokens empiezan con `v4.local.`). Verificación: `grep -r "jsonwebtoken" . --include="*.toml"` → cero resultados.
3. **Soft Delete** — Nunca DELETE real, solo `UPDATE deleted_at`. Verificación: `grep -rn "DELETE FROM users" . --include="*.rs"` → cero resultados.
4. **Fail-fast config** — Si falta variable de entorno, el proceso no arranca. `AppConfig::load()` panic en startup.
5. **Todo autenticado se audita** — `audit_logs` automático vía middleware.
6. **PostgreSQL es la base de datos principal** — SQLite solo en frontend (Local-First) y agentes (buffering offline).
7. **SSE preferido sobre WebSockets** — Más simple, compatible con HTTP infraestructura existente, mejor para firewalls institucionales.
8. **Rust 1.95.0** — Toolchain mínima. Containerfile usa `rust:1.95-alpine`.
9. **cargo-deny 0.19.6 + cargo-nextest 0.9.135** — En CI obligatorio.
10. **No añadir Fase 2 hasta que el problema exista** — Decisión consciente (ADR 0011).

---

## 📋 Estado del Proyecto

| Fase | Estado | Documento |
|------|--------|-------------|
| Génesis | Pendiente | `ROADMAP-GENESIS.md` |
| Backend I — Fundación | Pendiente | `ROADMAP-BACKEND.md` |
| Backend II — API | Pendiente | `ROADMAP-BACKEND.md` |
| Backend III — Auth | Pendiente | `ROADMAP-AUTH-FULLSTACK.md` |
| Backend IV — OpenAPI | Pendiente | `ROADMAP-BACKEND.md` |
| Frontend I — Setup | Pendiente | `ROADMAP-FRONTEND.md` |
| Frontend II — Tipos + Stores | Pendiente | `ROADMAP-FRONTEND.md` |
| Frontend III — Dashboard | Pendiente | `ROADMAP-FRONTEND.md` |
| Monitoreo I — Inventario | Pendiente | `ROADMAP-MONITOREO.md` |
| Monitoreo II — Métricas + Alertas | Pendiente | `ROADMAP-MONITOREO.md` |
| Monitoreo III — Topología | Pendiente | `ROADMAP-MONITOREO.md` |
| Infra — Deploy MVP | Pendiente | `ROADMAP-INFRA.md` |
| **MVP EN PRODUCCIÓN** | **Pendiente** | — |
| Agentes Distribuidos | Pendiente | `ROADMAP-MONITOREO.md` |
| Local-First Full | Pendiente | `ROADMAP-FRONTEND.md` |

---

## 🛡️ Seguridad

- **PASETO v4 Local** — Tokens con cifrado simétrico (no firmados como JWT). Rotación manual cada 90 días.
- **Argon2id** — Hashing de passwords vía `argon2` 0.5.3 (RustCrypto, pure Rust).
- **Rate Limiting** — En todos los endpoints de auth (ADR 0009).
- **RBAC** — Roles: Admin, Operator, Viewer, Agent (ADR 0006).
- **Audit Logs** — Toda acción autenticada se registra (user_id, action, resource, IP, timestamp).
- **Agent Security** — Token dedicado con scope `agent`, expiración 365 días, revocación inmediata vía dashboard (ADR 0022).
- **TLS 1.3** — Obligatorio para comunicación agente↔API.
- **Certificate Pinning** — Opcional Fase 2 para agentes (hash embedido en binario).
- **Systemd Hardening** — `NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome=true` (ADR 0022).

---

## 📊 Métricas y Observabilidad

- **tracing 0.1.44** + **tracing-subscriber 0.3.23** — Logs estructurados en todo el workspace.
- **sentry 0.47.0** — Monitoreo de errores en producción (opcional).
- **Healthchecks.io** — Pings de jobs y backups (ADR 0014).
- **Prometheus metrics** — Exposición opcional vía `/metrics` (ADR 0020).
- **Agent heartbeat** — Cada 60s con métricas de sistema (CPU, RAM, disco) (ADR 0022).

---

## 🌐 Local-First y Offline

- **@sqlite.org/sqlite-wasm 3.53.0-build1** — SQLite en browser con OPFS (Origin Private File System) para persistencia real.
- **Sync Engine** — TypeScript en `apps/web/src/lib/sync/` (frontend) y Rust en `crates/sync/` (agente).
- **UUID v7** — Identificadores monotónicos para operaciones offline (uuid@14.0.0).
- **Conflict Resolution** — Last-Write-Wins por defecto, configurable por entidad (ADR 0021).
- **Grace Period** — 24h para tokens expirados en modo offline (solo lectura + sync queue).

---

## 🤖 Agentes de Monitoreo

- **Rust 1.95.0** — Binario único estático, < 50MB RAM, < 20MB disco.
- **SNMP v1/v2/v3** — vía `snmp2` 0.5.0 (community configurable por sede).
- **ICMP** — Latencia y packet loss vía `surge-ping` 0.8.4.
- **ARP Scan** — Detección de dispositivos no autorizados.
- **SQLite local** — Buffering offline vía `sqlx` 0.8.6.
- **Sync automático** — Batch de hasta 100 métricas, retry con backoff exponencial.
- **Comandos remotos** — Reconfigure, Restart, UpdateWhitelist, ForceSync, Shutdown, UpdateVersion (ADR 0022).

---

## 📄 Licencia

MIT

---

## 👤 Autores

Milton Hipamo / Laboratorio 3030

---

> **Nota:** Este proyecto está basado en los ADRs en `guia/adr/` como fuente de verdad. La versión actual del README refleja el estado al 2026-05-16 con todas las dependencias verificadas y versionadas.
