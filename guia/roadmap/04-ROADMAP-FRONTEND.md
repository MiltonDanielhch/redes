# Roadmap — Frontend (Monitoreo de Infraestructura Regional)

> **Stack:** Astro 6 SSR · Svelte 5 Runes · Tailwind v4 · shadcn-svelte · TanStack Query · ArkType · Paraglide JS · LayerChart
>
> **Proyecto:** Sistema de Monitoreo de Infraestructura de Red Institucional
> **ADRs clave:** 0035 (Monitoreo Regional) · 0022 (Frontend) · 0023 (i18n) · 0006 (RBAC) · 0008 (PASETO) · 0021 (OpenAPI) · 0024 (Local-First)

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| FE.I | Fundación — setup e infraestructura | [ ] |
| FE.II | Tipos, estado y validación | [ ] |
| FE.III | Layouts y navegación | [ ] |
| FE.IV | Dashboard de Monitoreo | [ ] |
| FE.V | Inventario y Dispositivos | [ ] |
| FE.VI | Métricas y Gráficos | [ ] |
| FE.VII | Topología de Red | [ ] |
| FE.VIII | Alertas e Intrusiones | [ ] |
| FE.IX | i18n y formatters | [ ] |
| **Frontend** | | [ ] |

---

## FE.I — Fundación (setup e infraestructura)

> **Referencia:** ADR 0022, ADR 0035

```
[ ] Astro 6 SSR setup en apps/web/:
    [ ] npm create astro@latest apps/web -- --template minimal
    [ ] output: 'server'
    [ ] adapter: @astrojs/node (mode: 'standalone')
    [ ] @astrojs/svelte
    [ ] @astrojs/tailwind con applyBaseStyles: false
    [ ] @inlang/paraglide-js

[ ] npx shadcn-svelte@latest init

[ ] src/styles/global.css — CSS variables (shadcn-svelte default)

[ ] shadcn-svelte components instalados:
    [ ] Base: button, card, badge, separator, avatar
    [ ] Forms: input, label, form
    [ ] Feedback: alert, dialog, dropdown-menu
    [ ] Navigation: tabs, navigation-menu, sidebar, sheet
    [ ] Extras: tooltip, skeleton, is-mobile hook

[ ] Verificar: pnpm dev arranca sin errores
```

---

## FE.II — Tipos, estado y validación

> **Requiere:** Backend Bloque I completado
> **Referencia:** ADR 0027, ADR 0022

```
[ ] apps/web/src/lib/types/*.ts — tipos definidos manualmente:
    [ ] types/sede.ts
    [ ] types/device.ts (DeviceType, DeviceStatus)
    [ ] types/metric.ts
    [ ] types/alert.ts (AlertType, AlertSeverity)
    [ ] types/intrusion.ts

[ ] API client con fetch + PASETO
    [ ] Authorization: Bearer ${accessToken} en headers

[ ] apps/web/src/lib/stores/auth.svelte.ts
    [ ] user = $state<User | null>(null)
    [ ] accessToken = $state<string | null>(null)
    [ ] get isLoggedIn() { return user !== null }
    [ ] isAdmin() + hasPermission(permission)
    [ ] setAuth(user, token) + clearAuth()

[ ] QueryClient configurado en QueryProvider.svelte:
    [ ] @tanstack/svelte-query QueryClientProvider con staleTime 5min

[ ] apps/web/src/lib/validation/schemas.ts — ArkType:
    [ ] LoginSchema, RegisterSchema
    [ ] DeviceSchema
    [ ] AlertSchema

[ ] apps/web/src/lib/api/ — módulos por dominio:
    [ ] api/auth.ts (login, register, logout, refresh)
    [ ] api/sedes.ts (list, create, get)
    [ ] api/devices.ts (list, get, create, update, delete)
    [ ] api/metrics.ts (getRecent, getByDevice, getAggregated)
    [ ] api/alerts.ts (list, acknowledge)
    [ ] api/topology.ts (getBySede)
    [ ] api/intrusions.ts (list, resolve)
```

---

## FE.III — Layouts y navegación

> **Referencia:** ADR 0022, ADR 0008

```
[ ] BaseLayout.astro
    [ ] SEO tags, meta description
    [ ] Tailwind + global.css

[ ] DashboardLayout.astro
    [ ] Verificación SSR de PASETO
    [ ] Si no hay sesión → redirect /login
    [ ] Sidebar + Topbar + slot (canvas central)

[ ] components/layout/Sidebar.svelte
    [ ] Navegación colapsable — $state collapsed
    [ ] Tooltips en modo colapsado
    [ ] Iconos con lucide-svelte
    [ ] Items de MONITOREO (ADR 0035):
        [ ] NavItem: href="/dashboard" → Inicio
        [ ] NavItem: href="/dashboard/sedes" → Sedes
        [ ] NavItem: href="/dashboard/devices" → Dispositivos
        [ ] NavItem: href="/dashboard/metrics" → Métricas
        [ ] NavItem: href="/dashboard/topology" → Topología
        [ ] NavItem: href="/dashboard/alerts" → Alertas
        [ ] NavItem: href="/dashboard/intrusions" → Intrusiones
        [ ] NavItem: href="/dashboard/audit" permission="audit:read" → Auditoría
        [ ] NavItem: href="/dashboard/settings" → Configuración

[ ] components/layout/Topbar.svelte
    [ ] Búsqueda
    [ ] Notificaciones con badge de alertas críticas
    [ ] Avatar de usuario con dropdown

[ ] pages/login.astro + pages/register.astro
    [ ] Formularios con ArkType validation
    [ ] Integración con auth store
```

---

## FE.IV — Dashboard de Monitoreo

> **Referencia:** ADR 0035, ADR 0022

```
[ ] pages/dashboard/index.astro
    [ ] DashboardLayout
    [ ] KPIs de Monitoreo:
        [ ] Total de dispositivos
        [ ] Dispositivos online/offline
        [ ] Alertas activas
        [ ] Ancho de banda total

[ ] components/dashboard/KpiCard.svelte
    [ ] Props: title, value, badge, change con iconos

[ ] components/dashboard/StatsOverview.svelte
    [ ] createQuery para métricas globales
    [ ] 4 KpiCards conectados a datos reales

[ ] components/dashboard/AlertSummary.svelte
    [ ] Últimas alertas críticas
    [ ] Actualización cada 30s

[ ] components/dashboard/DeviceStatusChart.svelte
    [ ] Gráfico de pastel: online, offline, maintenance

[ ] components/dashboard/NetworkHealth.svelte
    [ ] Estado general de la red
    [ ] Latencia promedio
```

---

## FE.V — Inventario y Dispositivos

> **Referencia:** ADR 0035

```
[ ] pages/dashboard/sedes/index.astro
    [ ] DashboardLayout
    [ ] Lista de sedes regionales
    [ ] Cards con: nombre, ubicación, secretaría, dispositivo_count
    [ ] Crear/editar sede

[ ] pages/dashboard/devices/index.astro
    [ ] DashboardLayout
    [ ] DeviceTable con paginación, búsqueda, filtros
    [ ] Filtros por: tipo, estado, sede
    [ ] Columnas: hostname, IP, tipo, sede, estado, última vez visto

[ ] components/devices/DeviceTable.svelte
    [ ] Paginación + búsqueda
    [ ] Filtros por tipo y estado
    [ ] Acciones por fila: editar, eliminar, ver métricas

[ ] components/devices/DeviceForm.svelte
    [ ] Modal crear/editar
    [ ] Campos: hostname, IP, MAC, tipo, sede, vendor, modelo
    [ ] ArkType validation

[ ] components/devices/DeviceDetail.svelte
    [ ] Vista detallada del dispositivo
    [ ] Historial de métricas
    [ ] Estado actual (online/offline)

[ ] PermissionGate para acciones:
    [ ] devices:write → crear, editar, eliminar
    [ ] devices:read → ver lista
```

---

## FE.VI — Métricas y Gráficos

> **Referencia:** ADR 0035, ADR 0022

```
[ ] pages/dashboard/metrics/index.astro
    [ ] DashboardLayout
    [ ] Selector de dispositivo o sede
    [ ] Rango de tiempo: 1h, 6h, 24h, 7d, 30d

[ ] components/metrics/MetricsChart.svelte
    [ ] Gráfico de línea: bandwidth_rx, bandwidth_tx
    [ ] Eje Y: Mbps, Eje X: tiempo
    [ ] LayerChart o recharts

[ ] components/metrics/LatencyChart.svelte
    [ ] Gráfico de línea: latencia_ms
    [ ] Packet loss como área

[ ] components/metrics/MetricsSummary.svelte
    [ ] Stats: promedio, pico, mínimo
    [ ] Comparación con período anterior

[ ] SSE (Server-Sent Events) para realtime:
    [ ] /api/v1/stream/metrics/:device_id
    [ ] Actualización cada 5 segundos
    [ ]ADR 0022 — SSE preferido sobre WebSocket
```

---

## FE.VII — Topología de Red

> **Referencia:** ADR 0035

```
[ ] pages/dashboard/topology/index.astro
    [ ] DashboardLayout
    [ ] Selector de sede
    [ ] Mapa visual de red

[ ] components/topology/NetworkGraph.svelte
    [ ] Visualización de nodos y enlaces
    [ ] Nodos: dispositivos (color por estado)
    [ ] Enlaces: conexiones entre dispositivos
    [ ] Zoom y pan
    [ ] Click en nodo → details

[ ] components/topology/Legend.svelte
    [ ] Estados visuales:
        [ ] Verde: Online
        [ ] Amarillo: Advertencia
        [ ] Rojo: Offline
        [ ] Azul: Mantenimiento

[ ] components/topology/TopologyControls.svelte
    [ ] Zoom in/out
    [ ] Fit to screen
    [ ] Filter by type
```

---

## FE.VIII — Alertas e Intrusiones

> **Referencia:** ADR 0035

```
[ ] pages/dashboard/alerts/index.astro
    [ ] DashboardLayout
    [ ] Filtros: severidad, tipo, fecha, device
    [ ] Tabla de alertas

[ ] components/alerts/AlertTable.svelte
    [ ] Columnas: fecha, tipo, severidad, dispositivo, mensaje, acciones
    [ ] Acciones: acknowledge, details
    [ ] Badge por severidad (Critical=rojo, High=naranja, Medium=amarillo, Low=azul)

[ ] components/alerts/AlertDetail.svelte
    [ ] Detalle completo de la alerta
    [ ] Historial de la alerta

[ ] components/alerts/AlertForm.svelte
    [ ] Crear alerta manual (para testing)

[ ] pages/dashboard/intrusions/index.astro
    [ ] DashboardLayout
    [ ] Lista de dispositivos no autorizados detectados

[ ] components/intrusions/IntrusionList.svelte
    [ ] MAC, IP detectada, fecha, estado
    [ ] Acciones: resolve, mark_false_positive
```

---

## FE.IX — i18n y formatters (ADR 0023)

> **Referencia:** ADR 0023, ADR 0035

```
[ ] Paraglide JS configurado en astro.config.mjs
    [ ] project.inlang actualizado con baseLocale: es, locales: [es, en]
    [ ] i18n: { defaultLocale: 'es', locales: ['es', 'en'] }

[ ] apps/web/messages/es.json — mensajes en español
    [ ] dashboard, devices, metrics, topology, alerts, intrusions
    [ ] Términos técnicos: switch, router, AP, firewall, etc.

[ ] apps/web/messages/en.json — mensajes en inglés

[ ] apps/web/src/lib/i18n/formatters.ts:
    [ ] formatCurrency (BOB para Bolivia)
    [ ] formatDate (DD/MM/YYYY, timezone America/La_Paz)
    [ ] formatNumber (separadores bolivianos)
    [ ] formatNetworkSpeed (Mbps, Kbps)
    [ ] formatLatency (ms)
```

---

## ADRs de referencia por bloque

| Bloque | ADR |
|--------|-----|
| FE.I — Setup | 0035, 0022 |
| FE.II — Tipos | 0035, 0027, 0022 |
| FE.III — Layouts | 0035, 0008, 0022 |
| FE.IV — Dashboard | 0035, 0022 |
| FE.V — Dispositivos | 0035, 0006 |
| FE.VI — Métricas | 0035, 0022, 0024 |
| FE.VII — Topología | 0035 |
| FE.VIII — Alertas | 0035, 0016 |
| FE.IX — i18n | 0035, 0023 |

---

## Diagrama de Flujo de Bloques

```
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.I — Fundación                                                      │
│  ├─ Astro SSR (@astrojs/node standalone)                               │
│  ├─ Svelte 5 Runes + Tailwind v4                                      │
│  ├─ shadcn-svelte + bits-ui                                           │
│  ├─ TanStack Query + ArkType                                          │
│  └─ Paraglide JS (i18n)                                               │
│     └─ Ref: ADR 0035, 0022                                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.II — Tipos, Estado y Validación                                   │
│  ├─ types/*.ts (Sede, Device, Metric, Alert, Intrusion)              │
│  ├─ auth.svelte.ts — estado global con Runes                          │
│  ├─ ArkType schemas                                                   │
│  ├─ API client con Bearer PASETO                                      │
│  └─ Domain modules (auth, sedes, devices, metrics, alerts)           │
│     └─ Ref: ADR 0035, 0027, 0022, 0008                              │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.III — Layouts y Navegación                                        │
│  ├─ BaseLayout (SEO + QueryClientProvider)                              │
│  ├─ DashboardLayout (verifica PASETO SSR)                             │
│  ├─ Sidebar (sedés, devices, metrics, topology, alerts, intrusions)  │
│  ├─ Topbar + CommandPalette                                            │
│  └─ Pages: login, register, dashboard                                 │
│     └─ Ref: ADR 0035, 0008, 0006                                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.IV — Dashboard de Monitoreo                                        │
│  ├─ KpiCards (dispositivos, online/offline, alertas)                 │
│  ├─ AlertSummary (últimas alertas críticas)                           │
│  ├─ DeviceStatusChart (gráfico de pastel)                             │
│  ├─ NetworkHealth (estado general de la red)                         │
│  └─ TanStack Query para métricas globales                             │
│     └─ Ref: ADR 0035, 0022                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.V — Inventario y Dispositivos                                     │
│  ├─ Sedes (lista, crear, editar)                                      │
│  ├─ Devices (CRUD, filtros, búsqueda)                                 │
│  ├─ DeviceTable, DeviceForm, DeviceDetail                            │
│  ├─ PermissionGate para RBAC                                          │
│  └─ Tipos: switch, router, AP, firewall, server, UPS, cámara         │
│     └─ Ref: ADR 0035, 0006                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.VI — Métricas y Gráficos                                           │
│  ├─ Selector de dispositivo/sede y tiempo                             │
│  ├─ MetricsChart (bandwidth RX/TX)                                     │
│  ├─ LatencyChart (latencia + packet loss)                             │
│  ├─ MetricsSummary (promedio, pico, mínimo)                          │
│  └─ SSE para realtime (5s)                                            │
│     └─ Ref: ADR 0035, 0022, 0024                                     │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.VII — Topología de Red                                             │
│  ├─ Selector de sede                                                   │
│  ├─ NetworkGraph (nodos + enlaces)                                    │
│  ├─ Colores por estado: verde/amarillo/rojo/azul                      │
│  ├─ Zoom, pan, click para details                                     │
│  └─ Legend + Controls                                                 │
│     └─ Ref: ADR 0035                                                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.VIII — Alertas e Intrusiones                                       │
│  ├─ AlertTable (filtros por severidad, tipo, fecha)                  │
│  ├─ AlertDetail, AlertForm                                           │
│  ├─ IntrusionList (dispositivos no autorizados)                       │
│  ├─ Badges por severidad                                               │
│  └─ Acciones: acknowledge, resolve, false_positive                    │
│     └─ Ref: ADR 0035, 0016                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.IX — i18n y Formatters                                             │
│  ├─ Paraglide JS (es, en)                                             │
│  ├─ messages/es.json, messages/en.json                                │
│  ├─ formatDate(timezone America/La_Paz)                              │
│  ├─ formatNetworkSpeed (Mbps, Kbps) + formatLatency (ms)            │
│  └─ Términos técnicos del dominio (switch, router, AP, etc.)         │
│     └─ Ref: ADR 0035, 0023                                           │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta/Librería | URL | Útil para |
|----------------------|-----|-----------|
| **Astro 6** | https://docs.astro.build | SSR, layouts, routing |
| **Svelte 5 Runes** | https://svelte.dev/docs/svelte/what-are-runes | $state, $derived, $effect |
| **Tailwind CSS v4** | https://tailwindcss.com/docs | Utility classes |
| **shadcn-svelte** | https://shadcn-svelte.com | Componentes UI |
| **TanStack Query** | https://tanstack.com/query/latest | Caching, mutations |
| **ArkType** | https://arktype.io | Validación runtime |
| **Paraglide JS** | https://inlang.com/m/gerre34r/library-inlang-paraglideJs | i18n type-safe |
| **LayerChart** | https://layerchart.com | Gráficos realtime |
| **SSE** | https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events | Realtime |

---

## Troubleshooting — Frontend por Bloque

### FE.IV — Dashboard

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| KPIs no cargan | Endpoint no implementado | Verificar backend |
| Gráficos vacíos | Sin métricas en DB | Generar datos de prueba |

### FE.V — Dispositivos

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Tabla vacía | Sin dispositivos en DB | Crear dispositivos de prueba |
| Filtros no funcionan | Query params mal | Verificar API |

### FE.VI — Métricas

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Gráfico no renderiza | Sin datos o error de parse | Verificar datos |
| SSE no conecta | Backend no implementado | Verificar endpoint |

### FE.VII — Topología

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Nodos vacíos | device_links no creados | Crear enlaces |
| Gráfico lento | Demasiados nodos | Implementar pagination |

---

**Nota:** Este roadmap está basado en el ADR 0035 (Módulo de Monitoreo de Infraestructura Regional) para la Gobernación del Beni.