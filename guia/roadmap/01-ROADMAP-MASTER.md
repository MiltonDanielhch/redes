# Roadmap Master — Monitoreo de Infraestructura Regional (Beni)

> El mapa de todos los mapas. Cada bloque tiene su propio documento detallado.
> **Proyecto:** Sistema de Monitoreo de Infraestructura de Red Institucional
> **Referencia Principal:** ADR 0035 (Módulo de Monitoreo de Infraestructura Regional)
> **Fuente de verdad:** este archivo + ADRs en `guia/adr/`

---

## Estado global

| Fase | Documento | Foco | Cuando empezar | Estado |
|------|-----------|------|----------------|--------|
| 0 | `ROADMAP-GENESIS.md` | Workspace + tooling + estructura del proyecto | Día 1 | [ ] Pendiente |
| 1 | `ROADMAP-BACKEND.md` | Dominio + DB + API + Monitoreo | Día 1-3 | [ ] Pendiente |
| 2 | `ROADMAP-FRONTEND.md` | SvelteKit + Dashboard + Mapas | Paralelo con Backend | [ ] Pendiente |
| 3 | `ROADMAP-AUTH-FULLSTACK.md` | Login/Registro back+front | Después de Backend | [ ] Pendiente |
| 4 | `ROADMAP-INFRA.md` | Deploy + Caddy + Kamal + Litestream | MVP backend listo | [ ] Pendiente |
| 5 | `ROADMAP-MONITOREO.md` | Agentes + SNMP + Topología | Después de Backend II | [ ] Pendiente |
| — | **MVP EN PRODUCCIÓN** | Monitoreo de red regional | — | [ ] Pendiente |

---

## Orden de ejecución — el camino crítico

```
DÍA 1 ─── GÉNESIS
│         Workspace + crates (domain, application, infrastructure, database)
│         + tooling (mise, just, lefthook)
│         + estructura del proyecto según ADR 0035
│         cargo check --workspace ✓
│
DÍA 2 ─── BACKEND I — Fundación 🔥 CRÍTICO
│         entities: Sede, Dispositivo, Métrica, Alerta
│         ports: repositories para inventario, métricas, alertas
│         6 migraciones + SQLite + Litestream
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
│         Containerfile + Caddy + Kamal + Litestream
│         just deploy ✓ + kamal rollback ✓
│
─────────── MVP EN PRODUCCIÓN ────────────────────────────
│
DÍA 10+ ─ AGENTES DISTRIBUIDOS
│         Agentes en sedes remotas
│         Sincronización offline
│
DÍA 12+ ─ LOCAL-FIRST
│         SQLite Wasm en navegador
│         Sync queue para operación offline
```

---

## Qué puedes hacer en paralelo

| Tarea A | Tarea B | Se pueden hacer en paralelo? |
|---------|---------|------------------------------|
| Backend I — Migraciones | Frontend I — Setup Astro | ✅ Sí — no se tocan |
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
cargo deny check         → verde
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

### Monitoreo (ADR 0035)
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
Imagen Docker < 15MB
litestream snapshots → entradas de hoy
```

---

## Reglas de oro — el contrato del proyecto (ADR 0035)

| # | Regla | Garantizada por | ADR |
|---|-------|----------------|-----|
| 1 | `crates/domain` sin deps externas | Cargo.toml domain | 0001 |
| 2 | SQL solo en `crates/database` | Cargo.toml domain/application | 0001 |
| 3 | JWT prohibido — solo PASETO v4 Local | `jsonwebtoken` fuera del workspace | 0008 |
| 4 | Soft Delete — nunca DELETE real | Trigger deleted_at | 0006 |
| 5 | Toda acción autenticada se audita | audit_middleware automático | 0006 |
| 6 | Tipos TypeScript por buf generate | CI verifica diff en api.ts | 0027 |
| 7 | cargo-deny + cargo-audit en CI | just audit antes de deploy | 0010 |
| 8 | Imagen distroless — ~10MB, sin shell | Containerfile | 0013 |
| 9 | Fail-fast en config al arrancar | AppConfig::load() | 0002 |
| 10 | No añadir Fase 2 hasta que el problema exista | Decisión consciente | 0011 |
| 11 | Local-First para operación offline | SQLite Wasm + sync queue | 0024, 0035 |
| 12 | SSE preferido sobre WebSockets | ADR 0035 | 0035 |
| 13 | Agentes ligeros en sedes remotas | ADR 0035 | 0035 |

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
    [ ] 07-ROADMAP-INFRA.md completado
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
| `ROADMAP-INFRA.md` | Deploy, Caddy, Kamal, Litestream |
| `ROADMAP-MONITOREO.md` | Monitoreo de red (inventario, métricas, topología) |
| `guia/adr/` | 35 ADRs activos (incluye ADR 0035) |

---

## Stack Tecnológico del Proyecto

| Componente | Tecnología | ADR |
|------------|------------|-----|
| Backend | Rust + Axum | ADR 0003 |
| Frontend | SvelteKit + Svelte 5 | ADR 0022 |
| DB | SQLite + Litestream | ADR 0004, ADR 0035 |
| Auth | PASETO | ADR 0008 |
| Jobs | Apalis | ADR 0018 |
| Mail | Resend + React Email | ADR 0016 |
| Storage | Tigris (S3) | ADR 0020 |
| Monitoreo | Healthchecks.io | ADR 0015 |
| Realtime | SSE | ADR 0022, ADR 0035 |
| Offline | Local-First | ADR 0024, ADR 0035 |
| API Docs | OpenAPI + Utoipa | ADR 0021 |
| gRPC | ConnectRPC | ADR 0027 |
| SNMP | snmp crate | ADR 0035 |
| Topología | LayerChart | ADR 0035 |

---

## Componentes del Módulo de Monitoreo (ADR 0035)

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

**Nota:** Este roadmap está basado en el ADR 0035 que define el proyecto de Monitoreo de Infraestructura Regional para la Gobernación del Beni.