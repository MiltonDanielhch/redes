# Roadmap — Frontend (Monitoreo de Infraestructura Regional)

> **Stack:** SvelteKit SSR · Svelte 5 Runes · TypeScript · Tailwind v4 · shadcn-svelte · TanStack Query · ArkType · LayerChart
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
| FE.I | Fundación — SvelteKit + Tooling | [x] |
| FE.II | Tipos generados + Estado + Validación | [x] |
| FE.III | Layouts, navegación y SSR | [x] |
| FE.IV | Dashboard de Monitoreo | [ ] |
| FE.V | Inventario y Dispositivos | [ ] |
| FE.VI | Métricas, Gráficos y SSE | [ ] |
| FE.VII | Topología de Red | [ ] |
| FE.VIII | Alertas e Intrusiones | [ ] |
| FE.IX | Gestión de Agentes Distribuidos | [ ] |
| FE.X | Admin — Usuarios, Roles y Auditoría | [ ] |
| FE.XI | Local-First y Operación Offline | [ ] |
| FE.XII | PWA, Service Worker e i18n | [ ] |
| **Frontend** | | [ ] |

---

## FE.I — Fundación (SvelteKit + Tooling)

> **Referencia:** ADR 0017, ADR 0020, ADR 0021

```
[x] SvelteKit SSR setup en apps/web/:
    [x] pnpm create svelte@latest apps/web
        [x] template: Skeleton project
        [x] TypeScript: Yes
        [x] Add ESLint, Prettier, Playwright: Yes
    [x] adapter: @sveltejs/adapter-node@5.5.4 (mode: standalone) ← SSR para SEO/seguridad
    [x] output: SSR (no static, no SPA)

[x] Configurar Tailwind v4:
    [x] pnpm install -D tailwindcss @tailwindcss/vite
    [x] vite.config.ts: import tailwindcss from "@tailwindcss/vite"
    [x] src/app.css: @import "tailwindcss";
    [x] applyBaseStyles: false (shadcn-svelte maneja sus tokens)

[x] shadcn-svelte init:
    [x] pnpm dlx shadcn-svelte@latest init
    [x] baseColor: slate
    [x] aliases: $lib/components/ui, $lib/utils, etc.

[x] shadcn-svelte components instalados:
    [x] Base: button, card, badge, separator, avatar, table
    [x] Forms: input, label, form, select, textarea, checkbox
    [x] Feedback: alert, dialog, dropdown-menu, toast, sonner
    [x] Navigation: tabs, navigation-menu, sidebar, sheet, breadcrumb
    [x] Data: data-table, pagination, command
    [x] Extras: tooltip, skeleton, progress, calendar

[x] Dependencias adicionales:
    [x] @tanstack/svelte-query@6.1.28
    [x] @tanstack/svelte-table@9.0.0-alpha.47 ← v9 alpha requerida para Svelte 5
    [x] arktype@2.2.0
    [x] layerchart@2.0.0-next.63
    [x] @lucide/svelte ← paquete oficial para Svelte 5 (no lucide-svelte)
    [x] date-fns (timezone America/La_Paz)

[x] Verificar: pnpm dev arranca sin errores en localhost:5173
```

---

## FE.II — Tipos generados, estado y validación

> **Requiere:** Backend Bloque IV (OpenAPI) completado
> **Referencia:** ADR 0016, ADR 0017, ADR 0020

```
[x] Generación de tipos desde OpenAPI:
    [x] Script: scripts/generate-types.ts
    [x] Tipos desde DTOs del backend: src/lib/generated/api-types.ts
    [x] Tipos: User, Sede, Device, MetricReading, Alert, IntrusionEvent, etc.
    [ ] CI check: falla build si types desincronizados con backend (pendiente)
    [ ] NUNCA editar api-types.ts manualmente (requiere CI)

[x] Tipos manuales solo para UI/estado (no para dominio):
    [x] src/lib/types/ui.ts — Toast, ModalState, FilterState, ChartConfig
    [x] src/lib/types/auth.ts — AuthState (local, no del backend)

[x] API client con fetch + PASETO:
    [x] src/lib/api/client.ts
        [x] baseURL desde env: PUBLIC_API_URL
        [x] headers: Authorization: Bearer ${accessToken}
        [x] Content-Type: application/json
        [x] X-Request-Id generado por cliente
    [x] Manejo de 401: intentar refresh → re-login si falla
    [x] Manejo de 403: redirigir a /dashboard (sin permiso)
    [x] Manejo de 500: toast de error genérico + log

[x] src/lib/stores/auth.svelte.ts — estado global con Svelte 5 Runes:
    [x] user = $state<User | null>(null)
    [x] accessToken = $state<string | null>(null)
    [x] refreshToken = $state<string | null>(null)  ← para rotación PASETO
    [x] isLoggedIn = $derived(user !== null && accessToken !== null)
    [x] isAdmin = $derived(user?.roles.includes("admin") ?? false)
    [x] hasPermission(permission: string) = $derived(...)
    [x] setAuth(user, accessToken, refreshToken)
    [x] clearAuth() → limpiar localStorage + redirect /login
    [x] Persistencia: localStorage para tokens (encriptado con subtle crypto)

[x] QueryClient configurado en src/routes/+layout.svelte:
    [x] QueryClientProvider con staleTime 5min
    [x] retry: 1 (no reintentar errores 4xx)
    [x] refetchOnWindowFocus: false (sedes tienen ventanas múltiples)

[x] src/lib/validation/schemas.ts — ArkType:
    [x] LoginSchema, RegisterSchema
    [x] DeviceFormSchema (hostname, ip, mac, tipo, sede)
    [ ] AlertFilterSchema (status, severity, dateRange)
    [ ] SedeFormSchema

[ ] src/lib/api/ — módulos por dominio (usan tipos generados):
    [ ] api/auth.ts (login, register, logout, refresh)
    [ ] api/sedes.ts (list, create, get, update)
    [ ] api/devices.ts (list, get, create, update, archive)
    [ ] api/metrics.ts (getRecent, getByDevice, getAggregated)
    [ ] api/alerts.ts (list, acknowledge, resolve)
    [ ] api/topology.ts (getBySede)
    [ ] api/intrusions.ts (list, resolve, falsePositive)
    [ ] api/agents.ts (list, getStatus, updateConfig) ← ADR 0022
    [ ] api/audit.ts (list, getByUser) ← ADR 0006
    [ ] api/users.ts (list, create, update, softDelete) ← Admin
```

---

## FE.III — Layouts, navegación y SSR

> **Referencia:** ADR 0017, ADR 0008, ADR 0006

```
[ ] src/routes/+layout.svelte — Root layout:
    [ ] QueryClientProvider
    [ ] Toaster (sonner)
    [ ] ThemeProvider (dark/light según preferencia)
    [ ] Global CSS variables (Tailwind + shadcn tokens)

[ ] src/routes/+layout.server.ts — SSR data:
    [ ] Si ruta protegida → verificar PASETO token en cookie
    [ ] Si token inválido → set redirect a /login
    [ ] Inyectar user básico en locals para SSR

[ ] src/routes/(auth)/+layout.svelte — Auth layout (login/register):
    [ ] Sin sidebar, centrado, fondo institucional
    [ ] Logo Gobernación del Beni

[ ] src/routes/(dashboard)/+layout.svelte — Dashboard layout:
    [ ] Sidebar + Topbar + slot (canvas central)
    [ ] Verificación cliente: si !isLoggedIn → goto /login
    [ ] Hydration-safe: $effect para redirect (no durante SSR)

[ ] components/layout/Sidebar.svelte
    [ ] Navegación colapsable — $state collapsed
    [ ] Tooltips en modo colapsado
    [ ] Iconos con @lucide/svelte ← Svelte 5
    [ ] Items de MONITOREO (ADR 0020):
        [ ] NavItem: href="/dashboard" → Inicio
        [ ] NavItem: href="/dashboard/sedes" → Sedes
        [ ] NavItem: href="/dashboard/devices" → Dispositivos
        [ ] NavItem: href="/dashboard/metrics" → Métricas
        [ ] NavItem: href="/dashboard/topology" → Topología
        [ ] NavItem: href="/dashboard/alerts" → Alertas
        [ ] NavItem: href="/dashboard/intrusions" → Intrusiones
        [ ] NavItem: href="/dashboard/agents" → Agentes ← ADR 0022
        [ ] NavItem: href="/dashboard/audit" permission="audit:read" → Auditoría ← ADR 0006
        [ ] NavItem: href="/dashboard/admin/users" permission="user:read" → Admin ← ADR 0006
        [ ] NavItem: href="/dashboard/settings" → Configuración
    [ ] Badge de alertas críticas en sidebar (conteo vía SSE o polling)
    [ ] PermissionGate: ocultar items si no tiene permiso

[ ] components/layout/Topbar.svelte
    [ ] Breadcrumb dinámico
    [ ] Búsqueda global (CommandPalette)
    [ ] Notificaciones con badge de alertas críticas
    [ ] Indicador de conectividad (online/offline) ← ADR 0021
    [ ] Avatar de usuario con dropdown (perfil, logout)

[ ] src/routes/(auth)/login/+page.svelte + /register/+page.svelte
    [ ] Formularios con ArkType validation (cliente + servidor)
    [ ] Integración con auth store
    [ ] Redirect a /dashboard después de login exitoso
    [ ] Mostrar errores del backend (400, 401, 409)
```

---

## FE.IV — Dashboard de Monitoreo

> **Referencia:** ADR 0020, ADR 0017

```
[ ] src/routes/(dashboard)/+page.svelte
    [ ] DashboardLayout
    [ ] KPIs de Monitoreo en grid 4 cols:
        [ ] Total de dispositivos (con link a /devices)
        [ ] Dispositivos online / offline / maintenance (badges)
        [ ] Alertas activas (con link a /alerts)
        [ ] Ancho de banda total RX/TX última hora
        [ ] Agentes conectados / desconectados ← ADR 0022

[ ] components/dashboard/KpiCard.svelte
    [ ] Props: title, value, icon, trend (up/down), href
    [ ] Skeleton loading mientras carga

[ ] components/dashboard/StatsOverview.svelte
    [ ] createQuery para métricas globales (staleTime 1min)
    [ ] Revalidación cada 30s si online
    [ ] Si offline: mostrar últimos datos cacheados ← ADR 0021

[ ] components/dashboard/AlertSummary.svelte
    [ ] Últimas 5 alertas críticas/high
    [ ] Actualización vía SSE cada 10s o polling cada 30s
    [ ] Click → redirect a /alerts con filtro pre-aplicado

[ ] components/dashboard/DeviceStatusChart.svelte
    [ ] Gráfico donut: online, offline, maintenance
    [ ] LayerChart (PieChart o DonutChart)
    [ ] Colores: verde (#22c55e), rojo (#ef4444), azul (#3b82f6)

[ ] components/dashboard/NetworkHealth.svelte
    [ ] Estado general de la red (salud ponderada)
    [ ] Latencia promedio de sedes
    [ ] Packet loss promedio
    [ ] Tendencia últimas 24h (sparkline)

[ ] components/dashboard/AgentStatusWidget.svelte ← ADR 0022
    [ ] Lista compacta de agentes recientes
    [ ] Estado: online (verde), offline (rojo), syncing (amarillo)
```

---

## FE.V — Inventario y Dispositivos

> **Referencia:** ADR 0020, ADR 0006

```
[ ] src/routes/(dashboard)/sedes/+page.svelte
    [ ] DashboardLayout
    [ ] Lista de sedes regionales (cards o table)
    [ ] Datos: nombre, ubicación, secretaría, dispositivo_count, estado de red
    [ ] Crear/editar sede (modal con formulario)
    [ ] PermissionGate: crear/editar solo con "sedes:write"

[ ] src/routes/(dashboard)/devices/+page.svelte
    [ ] DashboardLayout
    [ ] DeviceTable con:
        [ ] Paginación server-side
        [ ] Búsqueda por hostname, IP, MAC
        [ ] Filtros por: tipo (multi-select), estado, sede
        [ ] Sort por: hostname, last_seen_at, status
    [ ] Columnas: hostname, IP, tipo, sede, estado, última vez visto, acciones

[ ] components/devices/DeviceTable.svelte
    [ ] TanStack Table v9 alpha (svelte-table) para sorting/filtering ← Svelte 5
    [ ] Acciones por fila:
        [ ] Ver métricas → /devices/:id/metrics
        [ ] Editar → modal DeviceForm
        [ ] Archivar → PUT /devices/:id/archive (soft delete) ← ADR 0006
    [ ] Badge de estado con colores:
        [ ] active → verde
        [ ] offline → rojo
        [ ] maintenance → azul

[ ] components/devices/DeviceForm.svelte
    [ ] Modal crear/editar
    [ ] Campos: hostname, IP (validación IPv4), MAC (validación IEEE 802), tipo, sede, notas
    [ ] ArkType validation en tiempo real
    [ ] Submit: POST/PUT a API

[ ] components/devices/DeviceDetail.svelte
    [ ] Vista detallada del dispositivo
    [ ] Info básica + historial de estados
    [ ] Métricas en tiempo real (SSE o polling)
    [ ] Alertas relacionadas
    [ ] Botón "Ver en topología" → /topology?sede_id=X&device_id=Y

[ ] src/routes/(dashboard)/devices/[id]/+page.svelte
    [ ] DeviceDetail + métricas históricas
    [ ] Tabs: Info | Métricas | Alertas | Topología
```

---

## FE.VI — Métricas, Gráficos y SSE

> **Referencia:** ADR 0020, ADR 0017, ADR 0021

```
[ ] src/routes/(dashboard)/metrics/+page.svelte
    [ ] DashboardLayout
    [ ] Selector de sede o dispositivo (dropdown con búsqueda)
    [ ] Selector de rango: 1h, 6h, 24h, 7d, 30d
    [ ] Toggle: "Modo realtime" (SSE) vs "Histórico" (REST)

[ ] components/metrics/MetricsChart.svelte
    [ ] Gráfico de área/linea: bandwidth_rx, bandwidth_tx
    [ ] Eje Y: Mbps (auto-escala), Eje X: tiempo
    [ ] LayerChart (AreaChart o LineChart con tooltip)
    [ ] Leyenda interactiva (toggle series)
    [ ] Brush/zoom para rangos personalizados

[ ] components/metrics/LatencyChart.svelte
    [ ] Gráfico de línea: latency_ms
    [ ] Packet loss como área secundaria (eje Y derecho)
    [ ] Threshold visual: línea roja en 100ms

[ ] components/metrics/MetricsSummary.svelte
    [ ] Stats: promedio, pico (95th percentile), mínimo
    [ ] Comparación con período anterior (delta %)
    [ ] Anomaly badge si metric.anomaly_detected = true

[ ] SSE (Server-Sent Events) para realtime:
    [ ] Endpoint backend: GET /api/v1/stream/metrics?device_id=X&sedes=Y
        [ ] ADR 0017 — SSE preferido sobre WebSocket
        [ ] Eventos: metric_update, alert_new, device_status_change
    [ ] components/metrics/RealtimeConnector.svelte
        [ ] EventSource con reconexión automática (exponential backoff)
        [ ] Si conexión perdida > 30s: mostrar "Realtime pausado" + fallback a polling
        [ ] Si offline total: mostrar últimos datos cacheados ← ADR 0021
        [ ] $effect para cleanup al desmontar

[ ] Offline fallback:
    [ ] Si navigator.onLine === false:
        [ ] Mostrar banner "Modo offline — datos pueden estar desactualizados"
        [ ] Deshabilitar SSE, usar cache TanStack Query
        [ ] Cola de acciones pendientes (acknowledge alert, etc.) ← ADR 0021
```

---

## FE.VII — Topología de Red

> **Referencia:** ADR 0020

```
[ ] src/routes/(dashboard)/topology/+page.svelte
    [ ] DashboardLayout
    [ ] Selector de sede (obligatorio)
    [ ] Canvas de topología

[ ] components/topology/NetworkGraph.svelte
    [ ] Visualización de nodos y enlaces
    [ ] Librería: LayerChart (Graph o Force-directed) o D3.js via Svelte action
    [ ] Nodos: dispositivos
        [ ] Tamaño proporcional a importancia (router > switch > AP)
        [ ] Color por estado: online (verde), offline (rojo), maintenance (azul), warning (amarillo)
        [ ] Icono por tipo (@lucide/svelte)
    [ ] Enlaces: device_links
        [ ] Grosor proporcional a bandwidth_mbps
        [ ] Color por tipo: ethernet (gris), fiber (azul claro), wireless (naranja)
        [ ] Animación de tráfico (dasharray animado)
    [ ] Interacciones:
        [ ] Zoom (wheel) y pan (drag)
        [ ] Click en nodo → panel lateral con DeviceDetail
        [ ] Doble click → /devices/[id]
        [ ] Hover en enlace → tooltip con bandwidth y tipo

[ ] components/topology/Legend.svelte
    [ ] Estados visuales: verde (online), amarillo (warning), rojo (offline), azul (maintenance)
    [ ] Tipos de enlace: ethernet, fiber, wireless

[ ] components/topology/TopologyControls.svelte
    [ ] Zoom in/out, fit to screen, reset
    [ ] Filter by device type (toggle switches, routers, etc.)
    [ ] Filter by status
    [ ] Toggle animation (tráfico)

[ ] components/topology/TopologyStats.svelte
    [ ] Resumen de la sede seleccionada:
        [ ] Total nodos, enlaces
        [ ] Single points of failure (detectados por crates/topology/)
        [ ] Path crítico más largo
```

---

## FE.VIII — Alertas e Intrusiones

> **Referencia:** ADR 0020, ADR 0006

```
[ ] src/routes/(dashboard)/alerts/+page.svelte
    [ ] DashboardLayout
    [ ] Filtros: severidad (multi), tipo (multi), estado, fecha, device, sede
    [ ] Tabla de alertas con paginación

[ ] components/alerts/AlertTable.svelte
    [ ] Columnas: fecha, tipo, severidad, dispositivo, sede, mensaje, estado, acciones
    [ ] Badge por severidad:
        [ ] critical → rojo (#dc2626) + icono alert-triangle
        [ ] high → naranja (#ea580c)
        [ ] medium → amarillo (#ca8a04)
        [ ] low → azul (#2563eb)
    [ ] Estado: active (pulsing dot), acknowledged (check), resolved (check-círculo)
    [ ] Acciones:
        [ ] Acknowledge → POST /alerts/:id/acknowledge (permission: alert:write)
        [ ] Resolve → POST /alerts/:id/resolve
        [ ] Details → modal con historial completo
    [ ] Bulk actions: acknowledge múltiples

[ ] components/alerts/AlertDetail.svelte
    [ ] Detalle completo: tipo, severidad, device, métricas relacionadas
    [ ] Timeline: created → acknowledged → resolved
    [ ] Notas internas (editable por admin)
    [ ] Link a métricas del device en el momento de la alerta

[ ] components/alerts/AlertFilters.svelte
    [ ] Presets: "Críticas sin ack", "Últimas 24h", "Por sede"
    [ ] Guardar filtros en URL query params (shareable)

[ ] src/routes/(dashboard)/intrusions/+page.svelte
    [ ] DashboardLayout
    [ ] Lista de dispositivos no autorizados detectados
    [ ] Filtros: status, fecha, sede

[ ] components/intrusions/IntrusionList.svelte
    [ ] Columnas: MAC, IP detectada, sede, fecha, estado, acciones
    [ ] Estado:
        [ ] detected → rojo + icono shield-alert
        [ ] investigating → amarillo
        [ ] resolved → verde
        [ ] false_positive → gris tachado
    [ ] Acciones:
        [ ] Investigate → cambia estado
        [ ] Resolve → POST /intrusions/:id/resolve (permission: intrusion:write)
        [ ] False Positive → POST /intrusions/:id/resolve con status false_positive
    [ ] Bulk: add to whitelist (crea device_whitelist entry)

[ ] components/intrusions/IntrusionDetail.svelte
    [ ] Historial de detección (cuántas veces apareció)
    [ ] Dispositivos cercanos en la misma sede (para correlación)
    [ ] Botón "Crear dispositivo" (si es legítimo y faltaba en inventario)
```

---

## FE.IX — Gestión de Agentes Distribuidos

> **Referencia:** ADR 0022, ADR 0020

```
[ ] src/routes/(dashboard)/agents/+page.svelte
    [ ] DashboardLayout
    [ ] Mapa/Lista de agentes por sede

[ ] components/agents/AgentList.svelte
    [ ] Columnas: sede, hostname, IP, versión, estado, último heartbeat, métricas enviadas
    [ ] Estado:
        [ ] online → verde (heartbeat < 2min)
        [ ] warning → amarillo (heartbeat 2-5min)
        [ ] offline → rojo (heartbeat > 5min o sin heartbeat)
        [ ] updating → azul (en proceso de actualización)
    [ ] Acciones:
        [ ] Ver logs → modal con logs recientes
        [ ] Restart → POST /agents/:id/restart (permission: agent:write)
        [ ] Update config → modal con JSON editor
        [ ] Unregister → soft delete (archivar agente)

[ ] components/agents/AgentConfigModal.svelte
    [ ] JSON editor para config del agente (intervalos de scan, rangos IP, SNMP community)
    [ ] Validación con ArkType
    [ ] Preview de cambios antes de enviar
    [ ] Si agente offline: encolar cambio para próxima conexión ← ADR 0021

[ ] components/agents/AgentMetrics.svelte
    [ ] Métricas del agente (no de la red):
        [ ] CPU/memory usage del agente
        [ ] Paquetes enviados/recibidos
        [ ] Latencia al API
        [ ] Queue size (métricas pendientes de sync)
```

---

## FE.X — Admin — Usuarios, Roles y Auditoría

> **Referencia:** ADR 0006, ADR 0008

```
[ ] src/routes/(dashboard)/admin/+layout.svelte
    [ ] Sub-layout con navegación lateral de admin
    [ ] PermissionGate: redirigir si no tiene "admin:access"

[ ] src/routes/(dashboard)/admin/users/+page.svelte
    [ ] Lista de usuarios (paginada, búsqueda)
    [ ] Columnas: nombre, email, roles, estado, último login, acciones
    [ ] Acciones: editar roles, desactivar (soft delete), ver sesiones
    [ ] Crear usuario (modal)

[ ] components/admin/UserForm.svelte
    [ ] Campos: name, email, roles (multi-select), is_active
    [ ] No editar password desde aquí (solo reset via email)

[ ] src/routes/(dashboard)/admin/roles/+page.svelte
    [ ] Lista de roles
    [ ] Permisos asignados (matrix: recurso × acción)
    [ ] Crear/editar roles

[ ] components/admin/PermissionMatrix.svelte
    [ ] Grid: recursos (users, devices, sedes, alerts, intrusions, agents, audit) × acciones (read, write, delete)
    [ ] Checkboxes para asignar permisos
    [ ] Presets: Admin (all), Operator (read+ack alerts), Viewer (read only)

[ ] src/routes/(dashboard)/admin/audit/+page.svelte
    [ ] Tabla de audit_logs
    [ ] Filtros: usuario, acción, recurso, fecha
    [ ] Columnas: timestamp, usuario, acción, recurso, IP, UA
    [ ] Exportar a CSV (permission: audit:export)

[ ] components/admin/AuditLogTable.svelte
    [ ] Paginación server-side (audit_logs puede ser grande)
    [ ] Formato de fecha: DD/MM/YYYY HH:mm:ss (America/La_Paz)
    [ ] IP geolocation opcional (mostrar ciudad si disponible)
```

---

## FE.XI — Local-First y Operación Offline

> **Referencia:** ADR 0021, ADR 0020
> **CRÍTICO:** Muchas sedes del Beni tienen conectividad inestable

```
[ ] src/lib/sync/offline-store.svelte.ts:
    [ ] isOnline = $state(navigator.onLine)
    [ ] syncStatus = $state<'idle' | 'syncing' | 'error'>('idle')
    [ ] pendingActions = $state<PendingAction[]>([])
    [ ] Escuchar eventos: online, offline
    [ ] Persistir pendingActions en IndexedDB

[ ] src/lib/sync/sync-queue.ts:
    [ ] Interfaz PendingAction: { id, type, endpoint, method, body, timestamp, retries }
    [ ] Encolar acción cuando offline o cuando fetch falla
    [ ] Tipos soportados:
        [ ] acknowledge_alert
        [ ] resolve_intrusion
        [ ] update_device_status
        [ ] create_alert_note
    [ ] NO encolar: create_device (necesita validación server-side), delete_user

[ ] src/lib/sync/sync-engine.ts:
    [ ] Procesar cola cuando vuelve online
    [ ] Orden FIFO (primero en entrar, primero en salir)
    [ ] Si acción falla (4xx): marcar error, notificar usuario, no reintentar
    [ ] Si acción falla (5xx / timeout): reintentar con backoff (max 3)
    [ ] Si token expirado durante sync: pausar, refresh, reanudar
    [ ] Conflict resolution: last-write-wins con timestamp del cliente

[ ] src/lib/sync/db.ts — SQLite Wasm (opcional, fase avanzada):
    [ ] @sqlite.org/sqlite-wasm@3.53.0-build1 ← paquete oficial ES Module
    [ ] Tablas mirror: cached_devices, cached_metrics, cached_alerts
    [ ] Sync desde API → SQLite cuando online
    [ ] Lectura desde SQLite cuando offline
    [ ] Background sync periódico (cada 5min si online)

[ ] components/sync/OfflineBanner.svelte
    [ ] Banner fijo arriba cuando isOnline === false
    [ ] Texto: "Modo offline — acciones se sincronizarán cuando vuelva la conexión"
    [ ] Contador de acciones pendientes
    [ ] Botón "Sincronizar ahora" (cuando vuelve online)

[ ] components/sync/SyncStatusIndicator.svelte
    [ ] Icono en topbar: check (synced), spinner (syncing), cloud-off (offline)
    [ ] Tooltip con detalle: "Última sync: hace 2min, 3 acciones pendientes"
```

---

## FE.XII — PWA, Service Worker e i18n

> **Referencia:** ADR 0017, ADR 0021

```
[ ] PWA manifest:
    [ ] src/app.html: <link rel="manifest" href="/manifest.json">
    [ ] manifest.json: name "Redes Beni", short_name "Redes", theme_color #0f172a
    [ ] Icons: 192x192, 512x512 (logo Gobernación del Beni)
    [ ] start_url: /dashboard
    [ ] display: standalone

[ ] Service Worker (src/service-worker.ts):
    [ ] Cache static assets (JS, CSS, icons) para carga instantánea
    [ ] Network-first para API calls (con fallback a cache si offline)
    [ ] NO cachear datos sensibles (auth tokens, audit logs)
    [ ] Skip waiting para updates inmediatos
    [ ] Notificaciones push para alertas críticas (opcional)

[ ] i18n — Internacionalización (opcional, español primario):
    [ ] src/lib/i18n/ — solo si se requiere multi-idioma en futuro
    [ ] Locale por defecto: es (español boliviano)
    [ ] date-fns locale es para formateo
    [ ] formatters:
        [ ] formatDate: DD/MM/YYYY HH:mm:ss (24h, timezone America/La_Paz)
        [ ] formatNetworkSpeed: Mbps, Kbps, Gbps (auto-escala)
        [ ] formatLatency: ms con 2 decimales
        [ ] formatBytes: KB, MB, GB (base 1024)
        [ ] formatPercent: 0-100% con 1 decimal
    [ ] Términos técnicos del dominio en español:
        [ ] switch → "Switch", router → "Router", access_point → "Punto de Acceso"
        [ ] bandwidth_saturation → "Saturación de Ancho de Banda"
        [ ] packet_loss → "Pérdida de Paquetes"
```

---

## ADRs de referencia por bloque

| Bloque | ADRs |
|--------|------|
| FE.I — Setup | 0017, 0012 |
| FE.II — Tipos | 0016, 0017, 0008 |
| FE.III — Layouts | 0017, 0008, 0006 |
| FE.IV — Dashboard | 0020, 0017 |
| FE.V — Dispositivos | 0020, 0006 |
| FE.VI — Métricas | 0020, 0017, 0021 |
| FE.VII — Topología | 0020 |
| FE.VIII — Alertas | 0020, 0006 |
| FE.IX — Agentes | 0022, 0020 |
| FE.X — Admin | 0006, 0008 |
| FE.XI — Local-First | 0021, 0020 |
| FE.XII — PWA/i18n | 0017, 0021 |

---

## Diagrama de Flujo de Bloques

```
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.I — Fundación                                                      │
│  ├─ SvelteKit SSR (@sveltejs/adapter-node@5.5.4)                       │
│  ├─ Svelte 5 Runes + Tailwind v4 + shadcn-svelte                      │
│  ├─ TanStack Query + ArkType + LayerChart                             │
│  └─ @lucide/svelte + date-fns                                         │
│     └─ Ref: ADR 0017, ADR 0012                                        │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.II — Tipos, Estado y Validación                                   │
│  ├─ Tipos generados desde OpenAPI (ADR 0016)                          │
│  ├─ auth.svelte.ts — estado global con Runes + refresh token        │
│  ├─ ArkType schemas (cliente + servidor)                              │
│  ├─ API client con PASETO Bearer + manejo 401/403/500                │
│  └─ Domain modules (auth, sedes, devices, metrics, alerts, agents)   │
│     └─ Ref: ADR 0016, 0017, 0008                                     │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.III — Layouts y Navegación                                        │
│  ├─ Root layout (QueryClient + Toaster + Theme)                        │
│  ├─ Auth layout (login/register)                                       │
│  ├─ Dashboard layout (Sidebar + Topbar + canvas)                      │
│  ├─ Sidebar con items de monitoreo + admin + PermissionGate          │
│  ├─ Topbar (breadcrumb, search, notifications, offline indicator)   │
│  └─ SSR auth verification (+layout.server.ts)                         │
│     └─ Ref: ADR 0017, 0008, 0006                                     │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.IV — Dashboard de Monitoreo                                      │
│  ├─ KpiCards (dispositivos, online/offline, alertas, bandwidth)      │
│  ├─ AlertSummary (SSE/polling)                                        │
│  ├─ DeviceStatusChart (donut)                                         │
│  ├─ NetworkHealth (sparkline)                                          │
│  └─ AgentStatusWidget (agentes recientes)                            │
│     └─ Ref: ADR 0020, 0022                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.V — Inventario y Dispositivos                                     │
│  ├─ Sedes (CRUD, cards, filtros)                                      │
│  ├─ Devices (table, paginación, búsqueda, filtros)                    │
│  ├─ DeviceForm (modal, ArkType validation)                            │
│  ├─ DeviceDetail (tabs: info, métricas, alertas, topología)           │
│  └─ Soft delete: "archivar" (no eliminar) ← ADR 0006                │
│     └─ Ref: ADR 0020, 0006                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.VI — Métricas, Gráficos y SSE                                    │
│  ├─ MetricsChart (bandwidth RX/TX, LayerChart)                        │
│  ├─ LatencyChart (latencia + packet loss)                             │
│  ├─ MetricsSummary (promedio, pico, percentile)                       │
│  ├─ RealtimeConnector (SSE con reconexión + fallback)                │
│  └─ Offline fallback (cache + banner) ← ADR 0021                    │
│     └─ Ref: ADR 0020, 0017, 0021                                     │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.VII — Topología de Red                                             │
│  ├─ NetworkGraph (nodos + enlaces, D3/LayerChart)                    │
│  ├─ Colores por estado, tamaño por importancia                        │
│  ├─ Zoom, pan, click → details                                       │
│  ├─ Legend + Controls                                                  │
│  └─ TopologyStats (SPOF, path crítico)                                │
│     └─ Ref: ADR 0020                                                 │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.VIII — Alertas e Intrusiones                                      │
│  ├─ AlertTable (filtros, badges, acciones ack/resolve)                │
│  ├─ AlertDetail (timeline, métricas relacionadas)                     │
│  ├─ IntrusionList (MAC, IP, estado, acciones)                        │
│  └─ IntrusionDetail (historial, correlación, whitelist)             │
│     └─ Ref: ADR 0020, 0006                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.IX — Gestión de Agentes Distribuidos                              │
│  ├─ AgentList (estado, heartbeat, métricas del agente)                │
│  ├─ AgentConfigModal (JSON editor, validación)                        │
│  ├─ AgentMetrics (CPU, memory, queue size)                            │
│  └─ Acciones: restart, update config, unregister                      │
│     └─ Ref: ADR 0022, 0020                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.X — Admin (Usuarios, Roles, Auditoría)                          │
│  ├─ User management (CRUD, roles, soft delete)                        │
│  ├─ PermissionMatrix (recurso × acción)                             │
│  ├─ Role presets (Admin, Operator, Viewer)                            │
│  └─ Audit logs (filtros, export CSV, formato Bolivia)                │
│     └─ Ref: ADR 0006, 0008                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.XI — Local-First y Operación Offline                              │
│  ├─ Offline store (navigator.onLine, pending actions)                  │
│  ├─ Sync queue (FIFO, IndexedDB, reintentos)                          │
│  ├─ Sync engine (conflict resolution, token refresh)                  │
│  ├─ SQLite Wasm cache (@sqlite.org/sqlite-wasm)                        │
│  ├─ OfflineBanner (acciones pendientes, sync manual)                   │
│  └─ SyncStatusIndicator (topbar)                                      │
│     └─ Ref: ADR 0021, 0020                                           │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.XII — PWA, Service Worker e i18n                                 │
│  ├─ Web App Manifest (standalone, icons)                               │
│  ├─ Service Worker (cache static, network-first API)                  │
│  ├─ Push notifications para alertas críticas (opcional)             │
│  ├─ i18n español primario (date-fns es, formatters Bolivia)           │
│  └─ Términos técnicos del dominio en español                          │
│     └─ Ref: ADR 0017, 0021                                           │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta/Librería | URL | Útil para |
|----------------------|-----|-----------|
| **SvelteKit** | https://kit.svelte.dev | SSR, routing, layouts, server hooks |
| **Svelte 5 Runes** | https://svelte.dev/docs/svelte/what-are-runes | $state, $derived, $effect |
| **Tailwind CSS v4** | https://tailwindcss.com/docs | Utility classes |
| **shadcn-svelte** | https://shadcn-svelte.com | Componentes UI accesibles |
| **TanStack Query** | https://tanstack.com/query/latest | Caching, mutations, refetching |
| **TanStack Table v9** | https://tanstack.com/table/latest | Tablas con sorting/filtering/paginación (v9 alpha para Svelte 5) |
| **ArkType** | https://arktype.io | Validación runtime type-safe |
| **LayerChart** | https://layerchart.com | Gráficos Svelte (line, area, pie, donut) |
| **@lucide/svelte** | https://lucide.dev/guide/packages/svelte | Iconografía oficial para Svelte 5 |
| **date-fns** | https://date-fns.org | Formateo de fechas con timezone |
| **SSE** | https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events | Realtime streaming |
| **Service Workers** | https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API | Offline, caching, PWA |
| **Web Locks API** | https://developer.mozilla.org/en-US/docs/Web/API/Web_Locks_API | Sync exclusivo (Local-First) |
| **@sqlite.org/sqlite-wasm** | https://www.npmjs.com/package/@sqlite.org/sqlite-wasm | SQLite en browser vía WASM |

---

## Troubleshooting — Frontend por Bloque

### FE.IV — Dashboard

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| KPIs no cargan | Endpoint no implementado o 401 | Verificar backend, verificar auth |
| Gráficos vacíos | Sin métricas en DB | Generar datos de prueba en backend |
| Agentes no aparecen | ADR 0022 no implementado | Implementar endpoint /agents primero |

### FE.V — Dispositivos

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Tabla vacía | Sin dispositivos en DB | Crear sedes y dispositivos de prueba |
| Filtros no funcionan | Query params mal enviados | Verificar API client y backend |
| "Eliminar" no funciona | Backend usa archive, no DELETE | Cambiar acción a PUT /devices/:id/archive |
| Error de tipos en tabla | TanStack Table v8 con Svelte 5 | Migrar a `@tanstack/svelte-table@9.0.0-alpha.47` |

### FE.VI — Métricas

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Gráfico no renderiza | Sin datos o tipo incorrecto | Verificar datos y tipos generados |
| SSE no conecta | Endpoint /stream no implementado | Verificar backend Bloque VII o usar polling |
| Datos desactualizados en offline | Sin cache local | Implementar FE.XI (Local-First) |

### FE.VII — Topología

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Nodos vacíos | device_links no creados | Crear enlaces entre dispositivos vía API |
| Gráfico lento | Demasiados nodos (>100) | Implementar virtualización o nivel de zoom |
| Enlaces no animados | Sin métricas de tráfico | Verificar que metric_readings tenga datos |

### FE.XI — Local-First / Offline

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Acciones perdidas al volver online | Sync queue no procesada | Verificar sync-engine.ts y eventos online |
| Conflictos de datos | Múltiples ediciones offline | Implementar timestamp-based last-write-wins |
| IndexedDB llena | Sin cleanup | Limpiar cache antigua (>30 días) periódicamente |
| Token expirado durante sync | Refresh no funciona en background | Implementar refresh token rotation |

### FE.XII — PWA

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| App no instala | Manifest mal configurado | Validar en Chrome DevTools → Application |
| Cache no actualiza | SW en modo cache-first | Cambiar a network-first para HTML |
| Notificaciones no llegan | Push service no configurado | Configurar VAPID keys si se usa push |

---

## Notas de actualización de versiones (2026-05-16)

| Componente | Versión anterior | Versión actual | Notas |
|------------|------------------|----------------|-------|
| `@sveltejs/adapter-node` | — | **5.5.4** | Última estable (feb 2026). SSR standalone. |
| `@lucide/svelte` | `lucide-svelte` | **@lucide/svelte** | Paquete oficial para Svelte 5. `lucide-svelte` solo soporta Svelte 3/4. |
| `@tanstack/svelte-table` | v8 implícita | **9.0.0-alpha.47** | v8 no compatible con Svelte 5. v9 alpha es la versión oficial compatible. |
| `@tanstack/svelte-query` | — | **6.1.28** | Última estable (may 2026). |
| `vitest` | v4.1 | **4.1.6** | Última patch estable (may 2026). |
| `playwright` | v1.59 | **1.60.0** | Última estable (may 2026). |
| `pnpm` | v10.27 | **11.1** | Última estable (may 2026). Requiere Node 22+. |
| `node` | v26 | **26.1.0** (Current) / **24** LTS | Node 26.1.0 publicado 7 may 2026. |
| `@sqlite.org/sqlite-wasm` | `sql.js` | **3.53.0-build1** | Paquete oficial SQLite WASM como ES Module. Reemplaza `sql.js` legacy. |
| `arktype` | — | **2.2.0** | Última estable (mar 2026). |
| `layerchart` | — | **2.0.0-next.63** | Última next (may 2026). Próximo a v2 estable. |
| `tailwindcss` | v4 | **v4.1** | Última estable (abr 2026). |
| `shadcn-svelte` | — | **1.2.7** | Última estable (abr 2026). |

---

**Nota:** Este roadmap está basado en el ADR 0020 (Módulo de Monitoreo de Infraestructura Regional) para la Gobernación del Beni. La operación offline (ADR 0021) es crítica para sedes con conectividad inestable.
