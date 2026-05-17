# Roadmap Master — Monitoreo de Infraestructura Regional (Beni)

> El mapa de todos los mapas. Cada bloque tiene su propio documento detallado.
> **Proyecto:** Sistema de Monitoreo de Infraestructura de Red Institucional
> **Referencia Principal:** ADR 0020 (Módulo de Monitoreo de Infraestructura Regional)
> **Fuente de verdad:** este archivo + ADRs en `guia/adr/`

---

## Estado global

| Fase | Documento | Foco | Cuando empezar | Estado |
|------|-----------|------|----------------|--------|
| 0 | `ROADMAP-GENESIS.md` | Workspace + tooling + estructura del proyecto | Día 1 | [ ] Pendiente |
| 1 | `ROADMAP-BACKEND.md` | Dominio + DB + API + Monitoreo | Día 1-3 | [ ] Pendiente |
| 2 | `ROADMAP-FRONTEND.md` | SvelteKit + Dashboard + Mapas | Paralelo con Backend | [ ] Pendiente |
| 3 | `ROADMAP-AUTH-FULLSTACK.md` | Login/Registro back+front | Después de Backend | [ ] Pendiente |
| 5 | `ROADMAP-MONITOREO.md` | Agentes + SNMP + Topología | Después de Backend II | [ ] Pendiente |
| — | **MVP EN PRODUCCIÓN** | Monitoreo de red regional | — | [ ] Pendiente |

---

## Orden de ejecución — el camino crítico

```
DÍA 1 ─── GÉNESIS
│         Workspace + crates (domain, application, infrastructure, database)
│         + tooling (mise v2026.5.10, just, lefthook v2.1.6)
│         + estructura del proyecto según ADR 0020
│         cargo check --workspace ✓
│
DÍA 2 ─── BACKEND I — Fundación 🔥 CRÍTICO
│         entities: Sede, Dispositivo, Métrica, Alerta
│         ports: repositories para inventario, métricas, alertas
│         6 migraciones + PostgreSQL
│
DÍA 3 ─── BACKEND II + FRONTEND I (paralelo)
│         Axum setup + API REST ← Backend
│         SvelteKit setup + componentes ← Frontend
│
DÍA 4 ─── BACKEND III + FRONTEND II (paralelo)
│         Auth + RBAC + Audit ← Backend
│         Tipos + stores + layouts ← Frontend
│
DÍA 5 ─── BACKEND IV + FRONTEND III (paralelo)
│         OpenAPI + DTOs ← Backend
│         Dashboard + Gráficos ← Frontend
│
DÍA 6 ─── MONITOREO I — Inventario y Dispositivos
│         CRUD de dispositivos (switches, routers, APs, firewalls)
│         Integración con agente SNMP
│
DÍA 7 ─── MONITOREO II — Métricas y Alertas
│         Recolección de métricas (bandwidth, latencia, packet loss)
│         Detección de anomalías y alertas
│
DÍA 8 ─── MONITOREO III — Topología
│         Mapa de red por sede
│         Visualización de conexiones
│
DÍA 9 ─── INFRA — Deploy
│         Containerfile + Caddy v2.11.3 + Kamal + pg_dump (no Litestream)
│         just deploy ✓ + kamal rollback ✓
│
─────────── MVP EN PRODUCCIÓN ────────────────────────────
│
DÍA 10+ ─ AGENTES DISTRIBUIDOS
│         Agentes en sedes remotas (Rust 1.95.0, reqwest 0.13.2, snmp2 0.5.0)
│         Sincronización offline (crates/sync/ Rust + sqlx 0.8.6)
│
DÍA 12+ ─ LOCAL-FIRST
│         PostgreSQL en servidor
│         Sync queue para operación offline (@sqlite.org/sqlite-wasm 3.53.0-build1)
```

---

## Qué puedes hacer en paralelo

| Tarea A | Tarea B | Se pueden hacer en paralelo? |
|---------|---------|------------------------------|
| Backend I — Migraciones | Frontend I — Setup SvelteKit | ✅ Sí — no se tocan |
| Backend II — Axum | Frontend I — Setup SvelteKit | ✅ Sí |
| Backend III — Auth | Frontend II — Tipos + store | ✅ Sí |
| Backend IV — OpenAPI | Frontend III — Layouts | ⚠️ Parcial — Requiere contrato |
| Monitoreo I — Dispositivos | Frontend IV — Dashboard | ✅ Sí |
| Backend V — Métricas | Infra — Deploy | ✅ Sí |

---

## Entregables por fase

### Génesis
```
cargo check --workspace  → verde
cargo deny 0.19.6 check  → verde
cargo audit check        → verde
just --list              → muestra todos los comandos
grep "jsonwebtoken" .    → cero resultados
```

### Backend completo
```
GET /api/v1/sedes → lista de sedes
GET /api/v1/devices → inventario de dispositivos
POST /api/v1/devices → crear dispositivo
GET /api/v1/metrics → métricas de red
POST /auth/register → 201
POST /auth/login → 200 + "v4.local.xxx" (PASETO)
curl http://localhost:8080/health → {"status":"ok"}
```

### Frontend completo
```
pnpm dev → arranca sin errores
/dashboard → KPIs de red + mapa de topología
/devices → tabla de dispositivos con filtros
/alerts → listado de alertas y anomalías
RBAC → botones ocultos sin permiso
```

### Monitoreo (ADR 0020)
```
Inventario de dispositivos → CRUD completo
Topología de red → mapa visual por sede
Métricas realtime → gráficos de bandwidth
Detección de intrusos → alertas por dispositivos desconocidos
Alertas → notificaciones por canales múltiples
```

### Infra (MVP en producción)
```
just deploy → funciona desde la laptop
https://tudominio.com/health → {"status":"ok"}
kamal rollback → < 10 segundos
Imagen Docker < 15MB (distroless)
pg_dump backups → entradas de hoy (no Litestream)
```

---

## Reglas de oro — el contrato del proyecto (ADR 0020)

| # | Regla | Garantizada por | ADR |
|---|-------|----------------|-----|
| 1 | `crates/domain` sin deps externas | Cargo.toml domain | 0001 |
| 2 | SQL solo en `crates/database` | Cargo.toml domain/application | 0001 |
| 3 | JWT prohibido — solo PASETO v4 Local | `jsonwebtoken` fuera del workspace | 0008 |
| 4 | Soft Delete — nunca DELETE real | Trigger deleted_at | 0006 |
| 5 | Toda acción autenticada se audita | audit_middleware automático | 0006 |
| 6 | Tipos TypeScript por openapi-typescript 7.13.0 | CI verifica diff en api-types.ts | 0016 |
| 7 | cargo-deny 0.19.6 + cargo-audit en CI | just audit antes de deploy | 0010 |
| 8 | Imagen distroless — ~10MB, sin shell | Containerfile (Rust 1.95.0) | 0013 |
| 9 | Fail-fast en config al arrancar | AppConfig::load() (toml 0.8.22) | 0002 |
| 10 | No añadir Fase 2 hasta que el problema exista | Decisión consciente | 0011 |
| 11 | Local-First para operación offline | @sqlite.org/sqlite-wasm 3.53.0-build1 + sync queue | 0021 |
| 12 | SSE preferido sobre WebSockets | ADR 0020 | 0020 |
| 13 | Agentes ligeros en sedes remotas | ADR 0020 (Rust 1.95.0, reqwest 0.13.2, snmp2 0.5.0) | 0020 |

---

## Cuándo pasar de Fase — Checklist de Transición

### De Fase 1 (MVP) → Fase 2 (Agentes Distribuidos)

**Criterio de activación:** Sedes remotas con conectividad inestable.

```
[ ] API funciona completamente
[ ] Dashboard muestra datos correctamente
[ ] Módulo de monitoreo operativos
[ ] Checklist previo:
    [ ] 02-ROADMAP-GENESIS.md completado
    [ ] 03-ROADMAP-BACKEND.md completado
    [ ] 04-ROADMAP-FRONTEND.md completado
    [ ] 05-ROADMAP-AUTH-FULLSTACK.md completado
```

---

## Documentos de referencia

| Documento | Propósito |
|-----------|-----------|
| `00-ROADMAP-TEMPLATE.md` | Template para crear nuevos roadmaps de módulos |
| `ROADMAP-GENESIS.md` | Arranque del proyecto — estructura + tooling |
| `ROADMAP-BACKEND.md` | Backend completo con checklist integrado |
| `ROADMAP-FRONTEND.md` | Frontend completo con checklist integrado |
| `ROADMAP-AUTH-FULLSTACK.md` | Login/Registro back+front coordinados |
| `ROADMAP-INFRA.md` | Deploy, Caddy v2.11.3, Kamal, pg_dump (no Litestream) |
| `ROADMAP-MONITOREO.md` | Monitoreo de red (inventario, métricas, topología) |
| `guia/adr/` | 20+ ADRs activos (incluye ADR 0020 v2.1) |

---

## Stack Tecnológico del Proyecto

| Componente | Tecnología | Versión | ADR |
|------------|------------|---------|-----|
| Backend | Rust + Axum | 1.95.0 / latest | ADR 0003 |
| Frontend | SvelteKit + Svelte 5 | 2.x / 5.x | ADR 0017 |
| DB | PostgreSQL | 17+ | ADR 0004 |
| Deploy | Coolify (alternativa) / Kamal (principal MVP) | latest / latest | ADR 0019 |
| Auth | PASETO | pasetors 0.7.8 | ADR 0008 |
| Jobs | Apalis | 1.0.0-rc.9 | ADR 0015 |
| Monitoreo | Healthchecks.io | SaaS (latest) | ADR 0014 |
| Realtime | SSE | nativo Axum | ADR 0017, ADR 0020 |
| API Docs | OpenAPI + Utoipa | 5.5.0 | ADR 0016 |
| SNMP | snmp2 | 0.5.0 | ADR 0020 |
| Topología | LayerChart | latest | ADR 0020 |
| Local-First SQLite | @sqlite.org/sqlite-wasm | 3.53.0-build1 | ADR 0021 |
| Sync Frontend | uuid v7 | uuid@14.0.0 | ADR 0021 |
| HTTP Client Agente | reqwest | 0.13.2 | ADR 0022 |
| ICMP Agente | surge-ping | 0.8.4 | ADR 0022 |
| Async Runtime | tokio | 1.52.3 (LTS) | ADR 0003, ADR 0022 |
| Config Agente | toml | 0.8.22 | ADR 0022 |
| Observabilidad | tracing + tracing-subscriber | 0.1.44 / 0.3.23 | ADR 0022 |
| Reverse Proxy Kamal | Caddy | 2.11.3 | ADR 0019 |
| Reverse Proxy Coolify | Traefik | 3.7.1 | ADR 0019 |
| Tooling | mise | v2026.5.10 | Workspace |
| Git Hooks | lefthook | 2.1.6 | Workspace |
| Linting Rust | cargo-deny | 0.19.6 | CI |
| Audit Rust | cargo-audit | latest | CI |

---

## Componentes del Módulo de Monitoreo (ADR 0020)

| Componente  | Responsabilidad                   |
| ----------- | --------------------------------- |
| `inventory` | Inventario físico de dispositivos |
| `topology`  | Mapeo visual de conexiones        |
| `metrics`   | Métricas de red                   |
| `alerts`    | Detección de anomalías            |
| `audit`     | Bitácora y reportes               |
| `agents`    | Recolección distribuida           |
| `sync`      | Sincronización offline            |

---

## Notas de corrección (v2.0 → v2.1)

**Cambios aplicados al 2026-05-16:**

1. **Tooling:** Se especifican versiones exactas: mise `v2026.5.10`, lefthook `2.1.6`, cargo-deny `0.19.6`
2. **Deploy:** Se elimina Litestream (es para SQLite, el proyecto usa PostgreSQL). Se reemplaza por `pg_dump` + cron/Apalis job
3. **Containerfile:** Rust actualizado de `1.86` a `1.95.0` (latest estable al 16 abr 2026)
4. **Caddy:** Fijado a `v2.11.3` (latest estable al 11 may 2026)
5. **Traefik:** Fijado a `v3.7.1` (latest estable al 11 may 2026) para escenario Coolify
6. **Tipos TypeScript:** Se corrige referencia de `buf generate` (gRPC/Protobuf) a `openapi-typescript@7.13.0` (OpenAPI/REST). El proyecto usa REST + OpenAPI, no gRPC
7. **Agentes:** Se especifican versiones: reqwest `0.13.2`, snmp2 `0.5.0`, tokio `1.52.3`, toml `0.8.22`
8. **Local-First:** Se especifica `@sqlite.org/sqlite-wasm` `3.53.0-build1` y `uuid@14.0.0` para UUID v7
9. **Observabilidad:** tracing `0.1.44`, tracing-subscriber `0.3.23`
10. **Auth:** pasetors `0.7.8`
11. **API Docs:** utoipa `5.5.0`
12. **Jobs:** Apalis `1.0.0-rc.9`
13. **Config Agente:** toml `0.8.22`

---

## Historial de cambios

| Versión | Fecha       | Cambios realizados |
| ------- | ----------- | ------------------ |
| 1.0     | 2026 (orig) | Versión inicial con Litestream, buf generate, Rust 1.86, versiones genéricas |
| 2.0     | 2026-05-16  | Elimina Litestream (reemplaza por pg_dump), corrige buf generate a openapi-typescript, actualiza Rust a 1.95.0, agrega versiones de Caddy y Traefik |
| 2.1     | 2026-05-16  | Fija versiones exactas de todas las dependencias y herramientas: mise v2026.5.10, lefthook 2.1.6, cargo-deny 0.19.6, reqwest 0.13.2, snmp2 0.5.0, tokio 1.52.3, toml 0.8.22, tracing 0.1.44, tracing-subscriber 0.3.23, pasetors 0.7.8, utoipa 5.5.0, Apalis 1.0.0-rc.9, @sqlite.org/sqlite-wasm 3.53.0-build1, uuid 14.0.0; actualiza stack tecnológico completo con tabla de versiones |
