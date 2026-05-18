# Roadmap — Frontend (Monitoreo de Infraestructura Regional)

> **Stack:** SvelteKit SSR · Svelte 5 Runes · TypeScript · Tailwind v4 · shadcn-svelte · TanStack Query · Zod
>
> **Proyecto:** Monitoreo de Infraestructura Regional - Gobernación del Beni
> **ADRs clave:** 0020 (Monitoreo Regional) · 0017 (Frontend SvelteKit) · 0006 (RBAC) · 0008 (PASETO) · 0016 (OpenAPI) · 0021 (Local-First) · 0022 (Agentes Distribuidos)

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| FE.I | Fundación — SvelteKit + Tooling | [x] 100% |
| FE.II | Tipos generados + Estado + Validación | [x] 100% |
| FE.III | Layouts, navegación y SSR | [x] 100% |
| FE.IV | Dashboard de Monitoreo | [x] 100% |
| FE.V | Inventario y Dispositivos | [x] ~80% (TanStack Table pendiente) |
| FE.VI | Métricas, Gráficos y SSE | [x] 100% |
| FE.VII | Topología de Red | [x] 100% |
| FE.VIII | Alertas e Intrusiones | [x] 100% |
| FE.IX | Gestión de Agentes Distribuidos | [x] 100% |
| FE.X | Admin — Usuarios, Roles y Auditoría | [x] 100% |
| FE.XI | Local-First y Operación Offline | [ ] 0% |
| FE.XII | PWA, Service Worker e i18n | [ ] 0% |
| **Frontend** | | **~85%** |

---

## FE.I — Fundación (SvelteKit + Tooling)

> **Referencia:** ADR 0017, ADR 0020, ADR 0021

```
[x] SvelteKit SSR setup en apps/web/
[x] TypeScript habilitado
[x] ESLint y Prettier configurados
[x] adapter-node@5.5.4 (mode: standalone)

[x] Tailwind v4 configurado con @tailwindcss/vite
[x] shadcn-svelte instalado con init
[x] Componentes: button, card, badge, input, label, dialog, dropdown-menu, select, separator, skeleton, avatar, table, toast
[x] @tanstack/svelte-query@6.x
[x] @lucide/svelte (no lucide-svelte)
[x] date-fns, zod instalados

[x] Verificar: pnpm dev arranca sin errores en localhost
```

---

## FE.II — Tipos generados, estado y validación

> **Requiere:** Backend OpenAPI
> **Referencia:** ADR 0016, ADR 0017, ADR 0020

```
[x] src/lib/generated/api-types.ts — tipos desde DTOs del backend:
    [x] User, LoginRequest, LoginResponse, RegisterRequest
    [x] SedeResponse, CreateSedeRequest, UpdateSedeRequest
    [x] DeviceResponse, CreateDeviceRequest, UpdateDeviceRequest
    [x] AlertResponse, CreateAlertRequest, AlertListResponse
    [x] IntrusionEvent, AgentResponse
    [x] MetricReading, AuditLogEntry, CreateUserRequest, etc.
    [ ] CI check pendiente (requiere backend)

[x] src/lib/types/ui.ts — Toast, ModalState, FilterState, ChartConfig, PaginationState, SortState, TableState
[x] src/lib/types/auth.ts — AuthState, Permission

[x] src/lib/api/client.ts — API client:
    [x] baseURL desde PUBLIC_API_URL
    [x] headers: Authorization Bearer, Content-Type, X-Request-Id
    [x] Manejo 401: redirect a /login
    [x] Manejo 403: redirect a /dashboard
    [x] Manejo 500: toast error

[x] src/lib/stores/auth.svelte.ts — Svelte 5 Runes:
    [x] user, accessToken, refreshToken como $state
    [x] isLoggedIn, isAdmin como $derived
    [x] hasPermission(permission)
    [x] setAuth(), clearAuth()
    [x] Persistencia en localStorage

[x] src/routes/+layout.svelte — Root layout:
    [x] QueryClientProvider configurado
    [x] Toaster de svelte-sonner
    [x] Global CSS

[x] src/lib/validation/schemas.ts — Zod:
    [x] LoginSchema, RegisterSchema
    [x] DeviceFormSchema, SedeFormSchema
    [x] AlertFilterSchema, UserFormSchema
```

---

## FE.III — Layouts, navegación y SSR

> **Referencia:** ADR 0017, ADR 0008, ADR 0006

```
[x] src/routes/+layout.svelte — Root layout
[x] src/routes/+page.svelte — Redirect a /login o /dashboard
[x] src/routes/(auth)/+layout.svelte — Auth layout centrado con logo
[x] src/routes/(auth)/login/+page.svelte
[x] src/routes/(auth)/register/+page.svelte
[x] src/routes/(dashboard)/+layout.svelte — Dashboard layout con sidebar
[x] src/routes/(dashboard)/dashboard/+page.svelte — Dashboard page

[x] components/layout/Sidebar.svelte:
    [x] Navegación colapsable (collapsed state)
    [x] Iconos con lucide-svelte
    [x] NavItems: Inicio, Sedes, Dispositivos, Métricas, Topología, Alertas, Intrusiones, Agentes, Auditoría, Admin, Configuración
    [x] Badge de alertas en sidebar
    [x] PermissionGate para ocultar items sin permiso

[x] components/layout/Topbar.svelte:
    [x] Búsqueda global
    [x] Notificaciones con badge
    [x] Avatar con dropdown (logout)

[x] src/lib/routeGuard.ts:
    [x] checkAuth() con redirect
[x] src/routes/(dashboard)/+layout.ts — requireLogin
[x] src/routes/(auth)/+layout.ts — requireLogout
[x] src/lib/components/auth/PermissionGate.svelte
[x] src/lib/components/auth/AuthForm.svelte
```

---

## FE.IV — Dashboard de Monitoreo

> **Referencia:** ADR 0020, ADR 0017

```
[x] src/routes/(dashboard)/dashboard/+page.svelte
[x] components/dashboard/StatsOverview.svelte
[x] components/dashboard/KpiCard.svelte
[x] components/dashboard/AlertSummary.svelte
[x] components/dashboard/DeviceStatusChart.svelte
[x] components/dashboard/NetworkHealth.svelte
[x] components/dashboard/AgentStatusWidget.svelte

[x] KPIs implementados:
    [x] Total de dispositivos
    [x] Dispositivos online / offline
    [x] Alertas activas
    [x] Ancho de banda
    [x] Agentes conectados

[x] createQuery con staleTime 1min
[x] Refetch cada 30s para alertas
[x] Skeleton loading en componentes
```

---

## FE.V — Inventario y Dispositivos

> **Referencia:** ADR 0020, ADR 0006

```
[x] src/routes/(dashboard)/sedes/+page.svelte
[x] src/routes/(dashboard)/devices/+page.svelte
[x] src/routes/(dashboard)/devices/[id]/+page.svelte

[x] components/devices/DeviceTable.svelte
[x] components/devices/DeviceForm.svelte

[ ] TanStack Table v9 — sorting/filtering avanzado pendiente
[ ] Paginación server-side pendiente
[ ] DeviceDetail component pendiente (usando page directo)
```

---

## FE.VI — Métricas, Gráficos y SSE

> **Referencia:** ADR 0020, ADR 0017, ADR 0021

```
[x] src/routes/(dashboard)/metrics/+page.svelte
[x] components/metrics/MetricsChart.svelte (SVG area chart)
[x] components/metrics/LatencyChart.svelte (SVG line chart)
[x] components/metrics/MetricsSummary.svelte
[x] components/metrics/RealtimeConnector.svelte

[x] Selector de rango: 1h, 6h, 24h, 7d, 30d
[x] Toggle realtime/polling mode
[x] EventSource con reconexión automática
[x] Exponential backoff para reconexión

[ ] LayerChart integration (usando SVG puro por ahora)
[ ] Brush/zoom para rangos personalizados
[ ] Offline banner cuando navigator.onLine = false
```

---

## FE.VII — Topología de Red

> **Referencia:** ADR 0020

```
[x] src/routes/(dashboard)/topology/+page.svelte
[x] components/topology/TopologyMap.svelte (SVG interactivo)
[x] components/topology/TopologySidebar.svelte

[x] Nodos con colores por estado (online/offline/warning)
[x] Edges con flechas y bandwidth
[x] Click handler para seleccionar nodo
[x] Jerarquía sede > dispositivos en sidebar
[x] Selector de sede

[ ] LayerChart Graph integration (usando SVG puro)
[ ] Zoom/pan controls
[ ] Legend component separado
```

---

## FE.VIII — Alertas e Intrusiones

> **Referencia:** ADR 0020, ADR 0006

```
[x] src/routes/(dashboard)/alerts/+page.svelte
[x] components/alerts/AlertTable.svelte
[x] components/alerts/AlertFilters.svelte

[x] src/routes/(dashboard)/intrusions/+page.svelte

[x] Filtros por severidad y estado
[x] Badges de severidad con colores
[x] Reconocer y resolver alertas
[x] Estados: active, acknowledged, resolved
[x] Marcar intrusiones como resolved/false-positive
```

---

## FE.IX — Gestión de Agentes Distribuidos

> **Referencia:** ADR 0022, ADR 0020

```
[x] src/routes/(dashboard)/agents/+page.svelte
[x] components/agents/AgentCard.svelte
[x] components/agents/AgentTable.svelte

[x] Stats: total, online, offline
[x] Acciones: restart, pause, resume
[x] Refresh manual de lista
```

---

## FE.X — Admin — Usuarios, Roles y Auditoría

> **Referencia:** ADR 0006, ADR 0008

```
[x] src/routes/(dashboard)/admin/users/+page.svelte
[x] src/routes/(dashboard)/admin/audit/+page.svelte
[x] src/lib/components/admin/UserTable.svelte

[x] CRUD de usuarios con modal
[x] Roles badges (admin, operator, viewer)
[x] Exportar auditoría a CSV
[x] Filtros por acción en auditoría
```

---

## FE.XI — Local-First y Operación Offline

> **Referencia:** ADR 0021

```
[ ] PendingQueue para acciones offline
[ ] TanStack Query persist con localStorage
[ ] Offline banner UI
[ ] Sync cuando vuelve la conexión
[ ] Optimistic updates
```

---

## FE.XII — PWA, Service Worker e i18n

```
[ ] manifest.json y service worker
[ ] PWA installable
[ ] i18n con svelte-i18n (español/inglés)
[ ] Theme toggle dark/light
```

---

## Commits Realizados

| Commit | Descripción |
|--------|-------------|
| `25d87b1` | feat(web): frontend foundation (FE.I, FE.II, FE.III) |
| `4901f29` | feat(web): FE.IV Dashboard de Monitoreo |
| `7a50d0c` | feat(web): FE.V Inventario y Dispositivos (parcial) |
| `3af1ec7` | feat(web): Auth frontend parcial (A.3, A.4, A.6, A.8) |
| `0b28194` | feat(web): FE.VI Métricas, Gráficos y SSE |
| `5165452` | feat(web): FE.VII Topología de Red |
| `925f083` | feat(web): FE.VIII Alertas e Intrusiones |
| `3ac47ee` | feat(web): FE.X Admin (Usuarios, Roles y Auditoría) |
| `d8324e3` | fix(web): errores de tipos y self-closing tags |
| `3090c8d` | feat(web): FE.IX Gestión de Agentes Distribuidos |

---

**Última actualización:** Frontend ~85% completo
**Pendiente:** FE.XI (Local-First), FE.XII (PWA/i18n)
