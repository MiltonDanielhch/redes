# Roadmap — Backend (Monitoreo de Infraestructura Regional)

> **Última revisión:** 2026-05-17
>
> **Stack:** Rust 1.95 · Axum 0.8 · SQLx 0.9.0-alpha.1 · PostgreSQL 17.10 · PASETO v4 · Utoipa 5.5
>
> **Proyecto:** Monitoreo de Infraestructura Regional - Gobernación del Beni
> **ADRs clave:** 0020 · 0001 · 0003 · 0004 · 0006 · 0007 · 0008

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| I | Fundación — Dominio + DB + RBAC | [x] 100% |
| II | API — Axum + Middleware + Errores | [x] 100% |
| III | Seguridad — Auth + RBAC + Audit | [ ] |
| IV | OpenAPI + Scalar | [ ] |
| V | Async — Jobs + Cache + Email | [ ] |
| VI | Observabilidad | [ ] |
| VII | Monitoreo — Inventario + Métricas | [ ] |
| **Backend Core** | | [x] **40%** |

---

## Bloque I — Fundación ✅

| Sub-bloque | Estado | Commit |
|-------------|--------|--------|
| B.1 Entidades + Puertos | ✅ | `7731871` |
| B.2 Migraciones SQLx | ✅ | `0895b26` |

### Entidades (11): User, Role, Session, AuditLog, Token, Alert, IntrusionEvent, Device, MetricReading, DeviceLink, Sede

### Puertos (9): UserRepository, SessionRepository, AuditRepository, TokenRepository, DeviceRepository, SedeRepository, MetricsRepository, AlertRepository, IntrusionRepository

---

## Bloque II — API (Axum + Middleware) ✅

| Sub-bloque | Estado | Commit |
|-------------|--------|--------|
| B.3 DTOs | ✅ | `47b4f52`, `42dbd93` |
| B.3 Handlers | ✅ | `42dbd93` |
| B.3 Rutas + AppState | ✅ | `2a13c1a` |
| B.3 Middleware tracing | ✅ | `517a36b` |

### Rutas configuradas:
```
GET    /health
GET    /ready
GET    /api/v1/sedes
POST   /api/v1/sedes
GET    /api/v1/sedes/:id
PUT    /api/v1/sedes/:id
GET    /api/v1/devices
POST   /api/v1/devices
GET    /api/v1/devices/:id
PUT    /api/v1/devices/:id
DELETE /api/v1/devices/:id
```

### Middleware:
- ✅ Tracing con `tower_http::TraceLayer`
- ⏳ Auth (PASETO) - pendiente por API incompatible

---

## Bloque III — Seguridad (Pendiente)

| Sub-bloque | Estado |
|-------------|--------|
| B.4 Auth (PASETO) | ⏳ Pendiente - API incompatible |
| B.5 RBAC middleware | ⏳ Pendiente |
| B.6 Audit logging | ⏳ Pendiente |
| B.7 Rate limiting | ⏳ Pendiente |

### Pendiente:
- Implementar PASETO real cuando API sea compatible
- Middleware de autenticación
- Middleware de autorización RBAC
- Logging de auditoría en handlers

---

## Commits Realizados

| Commit | Descripción |
|--------|-------------|
| `6e7fd3e` | feat(genesis): workspace completo con 12 crates y 3 apps |
| `7731871` | feat(backend): B.1 entidades de dominio + puertos/repositorios |
| `0895b26` | feat(backend): B.2 migraciones SQLx |
| `a1e254d` | docs: roadmap actualizado |
| `47b4f52` | feat(backend): B.3 DTOs + handlers sede |
| `42dbd93` | feat(backend): B.3 device handler |
| `1b46a99` | docs: roadmap actualizado |
| `2a13c1a` | feat(backend): B.3 rutas API + AppState + mock repositories |
| `517a36b` | feat(backend): middleware tracing con tower-http TraceLayer |

---

## Problemas Conocidos

| Problema | Severidad | Solución |
|----------|-----------|----------|
| PASETO API incompatible | 🔴 Alta | Pendiente - requiere investigación |
| Argon2id API inestable | 🟡 Media | Versión simplificada placeholder |
| Apalis sin features | 🟡 Media | Pendiente versión estable |

---

## Siguiente Fase

**Bloque III:** Seguridad - Auth + RBAC + Audit

```
1. [ ] Implementar PASETO cuando API sea compatible
2. [ ] Middleware de autenticación
3. [ ] Middleware RBAC
4. [ ] Audit logging en handlers
```

---

## Herramientas Verificadas (2026-05-17)

| Herramienta | Versión | Estado |
|-------------|---------|--------|
| Rust | 1.95.0 | ✅ |
| cargo | 1.95.0 | ✅ |
| axum | 0.8.9 | ✅ |
| tower-http | 0.6.10 | ✅ |
| tokio | 1.52.3 | ✅ |
| sqlx | 0.9.0-alpha.1 | ✅ |
| uuid | 1.23.1 | ✅ |
| time | 0.3.47 | ✅ |
| serde | 1.0.228 | ✅ |
