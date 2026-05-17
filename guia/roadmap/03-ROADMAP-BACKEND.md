# Roadmap — Backend (Monitoreo de Infraestructura Regional)

> **Última revisión de versiones:** 2026-05-17
> Se actualizaron las versiones de dependencias tras auditoría contra crates.io, GitHub, docs.rs y repositorios oficiales.
>
> **Stack:** Rust 1.95 · Axum 0.8 · SQLx 0.9.0-alpha.1 · PostgreSQL 17.10 · PASETO v4 · Utoipa 5.5
>
> **Proyecto:** Monitoreo de Infraestructura Regional - Gobernación del Beni
> **ADRs clave:** 0020 (Monitoreo Regional) · 0001 (Arquitectura) · 0003 (Axum) · 0004 (PostgreSQL) ·
> 0006 (RBAC) · 0007 (Errores) · 0008 (Auth) · 0009 (Rate Limit) · 0010 (Testing)

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| I | Fundación — Dominio + DB + RBAC | [x] B.1 Entities + Ports |
| | | [x] B.2 Migraciones SQLx |
| II | API — Axum + Middleware + Errores | [x] B.3 DTOs + Handlers |
| III | Seguridad — Auth + RBAC + Audit | [ ] |
| IV | OpenAPI + Scalar | [ ] |
| V | Async — Jobs + Cache + Email | [ ] |
| VI | Observabilidad | [ ] |
| VII | Monitoreo — Inventario + Métricas + Topología | [ ] |
| **Backend Core** | | [x] 30% |

---

## Bloque I — Fundación (Dominio + DB + RBAC) 🔥

> **Completado:** 2026-05-17

### I.1 — Pool PostgreSQL ✅

```
[x] crates/database/src/pool.rs — create_pool()
```

### I.2 — Migraciones SQLx ✅

```
[x] 15 archivos de migración en data/migrations/
```

### I.3 — Entidades de Dominio ✅

```
[x] 11 entidades + 9 puertos en crates/domain/
```

---

## Bloque II — API (Axum + Handlers) 🔥

> **En progreso:** 2026-05-17

### II.1 — DTOs ✅

```
[x] CreateSedeRequest, UpdateSedeRequest, SedeResponse
[x] CreateDeviceRequest, UpdateDeviceRequest, DeviceResponse
[x] ApiErrorResponse con IntoResponse para Axum
```

### II.2 — Handlers ✅

```
[x] SedeHandlers: list, get, create, update
[x] DeviceHandlers: list, get, create, update, delete
[x] Repository pattern con Arc<R>
```

### II.3 — Rutas (Pendiente)

```
[ ] Configurar rutas en Router
[ ] /api/v1/sedes/*
[ ] /api/v1/devices/*
```

### II.4 — Middleware (Pendiente)

```
[ ] Tracing middleware
[ ] Auth middleware (PASETO)
[ ] CORS middleware
```

---

## Commits Realizados

| Commit | Descripción |
|--------|-------------|
| `6e7fd3e` | feat(genesis): workspace completo |
| `7731871` | feat(backend): B.1 entidades + puertos |
| `0895b26` | feat(backend): B.2 migraciones SQLx |
| `a1e254d` | docs: roadmap actualizado |
| `47b4f52` | feat(backend): B.3 DTOs + handlers sede |
| `42dbd93` | feat(backend): B.3 device handler + routes skeleton |

---

## Siguiente Fase

**Bloque II (continuación):** Middleware + Rutas completas

```
1. [ ] Configurar Router con rutas /api/v1/*
2. [ ] Middleware de tracing
3. [ ] Middleware de auth (PASETO) - pendiente por API incompatible
4. [ ] Middleware de CORS
```
