# ADR 0017 — Frontend: SvelteKit + Svelte 5 Runes + SSE + Local-First

> **Última revisión de versiones:** 2026-05-16  
> Se actualizaron las versiones de dependencias tras auditoría contra npmjs.com, GitHub, crates.io y docs.rs.

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Versión** | 2.2 (Corrección 2026-05-16) |
| **Relacionado con** | ADR 0003 (Axum), ADR 0008 (PASETO Auth), ADR 0010 (Testing), ADR 0016 (OpenAPI), ADR 0020 (Monitoreo Regional), ADR 0021 (Local-First Sync Offline), ADR 0022 (Agentes Distribuidos) |

---

# Contexto

El sistema de monitoreo regional de infraestructura y redes de la Gobernación del Beni requiere:

- dashboards reactivos en tiempo real
- consumo mínimo de RAM/CPU
- carga rápida incluso en conexiones lentas
- SSR para rendimiento inicial y seguridad
- tipado compartido con backend Rust vía OpenAPI
- experiencia fluida tipo SPA después de la carga inicial
- mantenimiento simple para un equipo pequeño
- accesibilidad compatible con estándares gubernamentales
- degradación controlada ante fallos de conectividad
- operación offline parcial en sedes con conectividad inestable (ADR 0021)

React/Next.js añade complejidad y overhead innecesario para este caso de uso.

---

# Decisión

Usar:

- **SvelteKit SSR** como framework frontend fullstack
- **Svelte 5** con sistema de **Runes** ($state, $derived, $effect)
- **REST + OpenAPI** como contrato de comunicación con backend (ADR 0016)
- **TanStack Query** (Svelte Query) para cache, fetching y estado server
- **TypeScript strict mode**
- **TailwindCSS v4** para UI
- **shadcn-svelte** para componentes accesibles
- **LayerChart** para visualización de métricas y topología
- **ArkType** para validación runtime type-safe
- **date-fns** (timezone America/La_Paz) para formateo de fechas
- **@sveltejs/adapter-node** para despliegue Docker/Coolify
- **SSE (Server-Sent Events)** para tiempo real por defecto
- **Local-First architecture** con sync queue e IndexedDB para operación offline (ADR 0021)
- **PWA** con Service Worker para carga instantánea y cache de assets

---

# Arquitectura frontend

```text
apps/web/
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/           ← shadcn-svelte (button, card, table, dialog, etc.)
│   │   │   ├── layout/       ← Sidebar, Topbar, Breadcrumb
│   │   │   ├── dashboard/    ← KpiCard, StatsOverview, AlertSummary
│   │   │   ├── devices/        ← DeviceTable, DeviceForm, DeviceDetail
│   │   │   ├── metrics/        ← MetricsChart, LatencyChart, RealtimeConnector
│   │   │   ├── topology/       ← NetworkGraph, TopologyControls, Legend
│   │   │   ├── alerts/         ← AlertTable, AlertDetail, AlertFilters
│   │   │   ├── intrusions/     ← IntrusionList, IntrusionDetail
│   │   │   ├── agents/         ← AgentList, AgentConfigModal, AgentMetrics
│   │   │   ├── admin/          ← UserForm, PermissionMatrix, AuditLogTable
│   │   │   ├── sync/           ← OfflineBanner, SyncStatusIndicator
│   │   │   └── auth/           ← LoginForm, RegisterForm
│   │   │
│   │   ├── api/                ← Módulos por dominio (auth, sedes, devices, metrics, alerts, topology, intrusions, agents, audit, users)
│   │   ├── stores/             ← auth.svelte.ts, offline-store.svelte.ts
│   │   ├── sync/               ← sync-queue.ts, sync-engine.ts, db.ts (SQLite Wasm opcional)
│   │   ├── validation/         ← ArkType schemas (LoginSchema, DeviceFormSchema, etc.)
│   │   ├── types/              ← Tipos manuales UI + tipos generados desde OpenAPI
│   │   ├── utils/              ← Helpers puros (formatters Bolivia, i18n)
│   │   └── generated/          ← api-types.ts (generado por openapi-typescript, NUNCA editar manualmente)
│   │
│   ├── routes/
│   │   ├── +layout.svelte      ← Root layout (QueryClientProvider, Toaster, Theme)
│   │   ├── +layout.server.ts   ← SSR auth verification (cookie httpOnly)
│   │   ├── (auth)/             ← login, register, forgot-password, reset-password, verify
│   │   └── (dashboard)/        ← Dashboard, sedes, devices, metrics, topology, alerts, intrusions, agents, admin
│   │       ├── +layout.svelte  ← Sidebar + Topbar + slot
│   │       ├── +page.svelte    ← Dashboard principal (KPIs)
│   │       ├── sedes/
│   │       ├── devices/
│   │       ├── metrics/
│   │       ├── topology/
│   │       ├── alerts/
│   │       ├── intrusions/
│   │       ├── agents/
│   │       └── admin/
│   │           ├── users/
│   │           ├── roles/
│   │           └── audit/
│   │
│   ├── service-worker.ts       ← Cache static assets, network-first API, push opcional
│   ├── hooks.server.ts         ← Cookie parsing, auth redirect, request_id
│   ├── app.html
│   └── app.d.ts
│
├── static/
│   └── manifest.json           ← PWA manifest (name: "Redes Beni", display: standalone)
├── tests/
│   ├── unit/                   ← Vitest
│   └── e2e/                    ← Playwright
├── package.json
└── pnpm-workspace.yaml
```

---

# Política de componentes

| Carpeta               | Responsabilidad                              |
| --------------------- | -------------------------------------------- |
| `components/ui`       | Componentes reutilizables y presentacionales (shadcn-svelte) |
| `components/features` | Componentes específicos de dominio (devices, metrics, alerts, topology, agents) |
| `routes/`             | Ensamblaje de páginas y layouts              |
| `lib/api/`            | Cliente HTTP por dominio (fetch + tipos OpenAPI) |
| `lib/stores/`         | Estado reactivo global con Svelte 5 Runes    |
| `lib/sync/`           | Lógica de operación offline (ADR 0021)       |
| `lib/validation/`     | Schemas ArkType (cliente + servidor)         |
| `lib/utils/`          | Helpers puros sin dependencias visuales      |

## Reglas

* Evitar lógica de negocio dentro de componentes visuales
* Los componentes UI no conocen la API ni auth
* La lógica de realtime vive en `lib/api/` y `lib/stores/`
* Los tipos generados desde OpenAPI (`lib/generated/api-types.ts`) nunca se editan manualmente
* Todo formulario usa ArkType para validación cliente y servidor

---

# Stack tecnológico detallado

| Componente | Tecnología | Versión/Config | ADR |
|------------|-----------|----------------|-----|
| Framework | SvelteKit | SSR, adapter-node | — |
| Reactividad | Svelte 5 Runes | $state, $derived, $effect | — |
| Lenguaje | TypeScript | strict mode | — |
| Gestor de paquetes | pnpm | v11.1 | ADR 0012 |
| Node | Node.js | v26.1.0 (Current) / v24 LTS | ADR 0012 |
| Estilos | TailwindCSS v4 | @tailwindcss/vite | FE.I |
| Componentes UI | shadcn-svelte | baseColor: slate | FE.I |
| Iconos | @lucide/svelte | — | FE.I |
| Data fetching | TanStack Query (Svelte Query) | staleTime 5min, retry 1 | FE.II |
| Tablas | TanStack Table v9 alpha (Svelte 5) | sorting, filtering, pagination | FE.V |
| Validación | ArkType | runtime type-safe | FE.II |
| Gráficos | LayerChart | line, area, pie, donut, graph | FE.VI, FE.VII |
| Fechas | date-fns | timezone America/La_Paz | FE.XII |
| API types | openapi-typescript | generado desde `/openapi.json` | ADR 0016 |
| HTTP client | fetch nativo | wrapper con interceptores | FE.II |
| Realtime | SSE (EventSource) | reconexión automática, fallback polling | FE.VI |
| Offline | IndexedDB + sync queue | FIFO, conflict resolution last-write-wins | FE.XI |
| PWA | Service Worker | cache static, network-first API | FE.XII |
| Testing unit | Vitest | v4.1.6 | ADR 0010 |
| Testing E2E | Playwright | v1.60.0 | ADR 0010 |
| Lint | eslint-plugin-svelte | — | ADR 0010 |
| Check | svelte-check | — | ADR 0010 |

---

# Svelte 5 Runes

Se adopta el nuevo sistema de reactividad explícita de Svelte 5.

> **Nota:** Svelte 5.55.0 (mayo 2026) es la última estable. Incluye tipos exportados para `TweenOptions`, `SpringOptions`, `SpringUpdateOptions` y `Updater` desde `svelte/motion`. SvelteKit 2.57.0 incluye breaking changes en Remote Functions (requerir permiso del servidor para refrescar queries).

## Estado reactivo

```ts
// src/lib/stores/auth.svelte.ts
let user = $state<User | null>(null);
let accessToken = $state<string | null>(null);
let refreshToken = $state<string | null>(null);
```

## Valores derivados

```ts
let isLoggedIn = $derived(user !== null && accessToken !== null);
let isAdmin = $derived(user?.roles.includes("admin") ?? false);
let hasPermission = (permission: string) => $derived(
    user?.permissions.includes(permission) ?? false
);
```

## Side effects controlados

```ts
$effect(() => {
    if (accessToken && isTokenExpiringSoon(accessToken)) {
        triggerSilentRefresh();
    }
});
```

---

# SSR + SPA híbrido

| Estrategia   | Uso                      |
| ------------ | ------------------------ |
| SSR          | Login, dashboard inicial, SEO, auth verification |
| CSR/SPA      | Navegación interna, dashboards interactivos |
| Streaming    | Métricas en tiempo real vía SSE |
| Lazy Loading | Gráficos pesados (LayerChart), modales admin |

---

# Estrategia de carga de datos

| Estrategia        | Uso                            |
| ----------------- | ------------------------------ |
| `+page.server.ts` | Datos sensibles o autenticados (verificación cookie httpOnly, redirect si no auth) |
| `+page.ts`        | Datos cacheables (lista de sedes, dispositivos) |
| Fetch cliente     | Acciones POST/PUT, TanStack mutations |
| SSE               | Métricas continuas, alertas en tiempo real |
| IndexedDB         | Cache offline de datos de lectura (ADR 0021) |

Ejemplo SSR auth:

```ts
// +layout.server.ts
export async function load({ cookies, locals }) {
    const token = cookies.get("access_token"); // httpOnly, secure, sameSite=strict

    if (!token && isProtectedRoute(locals.route)) {
        throw redirect(302, "/login");
    }

    return { user: locals.user };
}
```

---

# Comunicación con backend

Se utiliza **REST + OpenAPI** como contrato oficial (ADR 0016). El frontend consume tipos generados automáticamente desde `/openapi.json` del backend.

## Generación de tipos

```bash
# Descargar spec y generar tipos TypeScript puros
npx openapi-typescript http://localhost:8080/openapi.json   --output apps/web/src/lib/generated/api-types.ts
```

## Cliente API

```ts
// src/lib/api/client.ts
const apiClient = {
    async fetch<T>(endpoint: string, options?: RequestInit): Promise<T> {
        const res = await fetch(`${PUBLIC_API_URL}${endpoint}`, {
            ...options,
            headers: {
                "Content-Type": "application/json",
                "Authorization": `Bearer ${getAccessToken()}`,
                "X-Request-Id": generateRequestId(),
                ...options?.headers,
            },
        });

        if (res.status === 401) {
            await handleUnauthorized();
        }

        return res.json();
    }
};
```

## Módulos por dominio

```ts
// src/lib/api/devices.ts
import { apiClient } from "./client";
import type { paths } from "$lib/generated/api-types";

type ListDevicesResponse = paths["/api/v1/devices"]["get"]["responses"]["200"]["content"]["application/json"];

export async function listDevices(params?: { sede_id?: string }): Promise<ListDevicesResponse> {
    return apiClient.fetch(`/api/v1/devices?${new URLSearchParams(params)}`);
}
```

**Nota:** No se usa ConnectRPC ni `buf generate`. El proyecto utiliza REST/Axum con OpenAPI como contrato único (ADR 0003, ADR 0016).

---

# Estado global

## Reglas

| Tipo de estado  | Estrategia    |
| --------------- | ------------- |
| Estado local    | `$state`      |
| Estado derivado | `$derived`    |
| Auth            | `auth.svelte.ts` (Runes + localStorage encriptado) |
| Cache server    | TanStack Query (QueryClient) |
| Tiempo real     | SSE (EventSource) |
| Offline         | `offline-store.svelte.ts` + IndexedDB |

## Restricciones

* Evitar stores globales innecesarias (Svelte 5 Runes reemplaza Svelte stores clásicos)
* No usar Redux/Zustand/MobX
* La cache de datos server pertenece a TanStack Query
* Los tokens de auth se persisten en **localStorage encriptado** (subtle crypto) para multi-tab; la cookie httpOnly se usa para SSR

---

# Autenticación

## Estrategia

* PASETO v4 desde backend Rust (ADR 0008)
* **localStorage encriptado** para access_token y refresh_token (persistencia multi-tab)
* **Cookie httpOnly, Secure, SameSite=Strict** para SSR auth verification
* Refresh token rotativo (one-time use)
* Auto-refresh silencioso cuando el token expira en < 2 minutos

```ts
// src/lib/stores/auth.svelte.ts
export const authStore = {
    user: $state<User | null>(null),
    accessToken: $state<string | null>(null),
    refreshToken: $state<string | null>(null),

    setAuth(user, access, refresh) {
        this.user = user;
        this.accessToken = access;
        this.refreshToken = refresh;
        persistEncrypted({ access, refresh }); // localStorage + subtle crypto
    },

    clearAuth() {
        this.user = null;
        this.accessToken = null;
        this.refreshToken = null;
        clearPersisted();
        QueryClient.clear();
        goto("/login");
    }
};
```

## SSR verification

```ts
// hooks.server.ts
export async function handle({ event, resolve }) {
    const token = event.cookies.get("access_token"); // httpOnly cookie seteada en login

    if (token) {
        try {
            event.locals.user = await validateTokenServerSide(token);
        } catch {
            event.cookies.delete("access_token", { path: "/" });
        }
    }

    return resolve(event);
}
```

## Reglas

* Nunca guardar tokens en localStorage sin encriptar
* Cookies `Secure`, `HttpOnly`, `SameSite=Strict` para SSR
* Logout invalida sesión en backend y limpia localStorage + cookies
* Mutex en refresh token para evitar race conditions

---

# Tiempo real

## Estrategia

Para métricas de monitoreo y alertas:

* **Server-Sent Events (SSE)** por defecto
* **Polling** como fallback si SSE falla
* **WebSocket** solo si se justifica arquitectónicamente (requiere ADR adicional)

## Motivos

* SSE consume menos recursos que WebSockets
* Más simple (HTTP nativo, reconexión automática del browser)
* Suficiente para dashboards unidireccionales (server → client)
* Reconexión automática integrada en EventSource

## Política

| Tecnología | Uso                        |
| ---------- | -------------------------- |
| SSE        | Dashboards, métricas, alertas en tiempo real |
| Polling    | Fallback cada 30s si SSE desconectado > 30s |
| WebSocket  | Comunicación bidireccional (requiere justificación) |

## Implementación

```ts
// src/lib/api/realtime.ts
export function connectMetricsStream(deviceId: string, onMessage: (data) => void) {
    const source = new EventSource(`/api/v1/stream/metrics?device_id=${deviceId}`);

    source.onmessage = (event) => {
        onMessage(JSON.parse(event.data));
    };

    source.onerror = () => {
        source.close();
        // Fallback a polling después de 30s sin reconexión
        setTimeout(() => startPollingFallback(deviceId, onMessage), 30000);
    };

    return () => source.close();
}
```

---

# Degradación controlada y Local-First (ADR 0021)

Si la conexión realtime falla o el dispositivo está offline:

* fallback automático a polling cada 30s
* mostrar último snapshot válido desde cache (TanStack Query + IndexedDB)
* dashboard nunca debe quedar vacío
* notificación visual de estado degradado (banner offline)
* acciones del usuario se encolan en IndexedDB (sync queue FIFO)
* al volver online: procesar cola, resolver conflictos last-write-wins

```ts
// src/lib/sync/offline-store.svelte.ts
let isOnline = $state(navigator.onLine);
let syncStatus = $state<'idle' | 'syncing' | 'error'>('idle');
let pendingActions = $state<PendingAction[]>([]);

window.addEventListener('online', () => {
    isOnline = true;
    processSyncQueue();
});

window.addEventListener('offline', () => {
    isOnline = false;
});
```

---

# Manejo de errores

## Estrategia

* Error boundaries por layout (SvelteKit)
* Retry automático solo para GETs idempotentes (TanStack Query)
* Toasts no bloqueantes (sonner)
* Manejo específico por código HTTP:
  * 401 → intentar refresh → re-login si falla
  * 403 → redirect /dashboard + toast "Sin permiso"
  * 429 → mostrar countdown, deshabilitar botón
  * 500 → toast genérico + log

```ts
// src/lib/api/client.ts
async function handleResponse<T>(res: Response): Promise<T> {
    if (res.status === 401) {
        const refreshed = await attemptSilentRefresh();
        if (!refreshed) {
            authStore.clearAuth();
            throw new Error("SESSION_EXPIRED");
        }
    }

    if (res.status === 403) {
        goto("/dashboard");
        toast.error("No tienes permiso para acceder a este recurso");
    }

    if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
    }

    return res.json();
}
```

---

# UI y diseño

## Stack visual

| Herramienta      | Uso                              |
| ---------------- | -------------------------------- |
| TailwindCSS v4   | Layout, spacing, tipografía      |
| shadcn-svelte    | Componentes accesibles (button, card, dialog, table, form, toast) |
| Lucide Icons     | Iconografía consistente          |
| LayerChart       | Gráficos de métricas y topología |

## shadcn-svelte

Instalación:

```bash
npx shadcn-svelte@latest init
# baseColor: slate
# aliases: $lib/components/ui, $lib/utils
```

Componentes base instalados:
- button, card, badge, separator, avatar, table
- input, label, form, select, textarea, checkbox
- alert, dialog, dropdown-menu, toast, sonner
- tabs, navigation-menu, sidebar, sheet, breadcrumb
- data-table, pagination, command
- tooltip, skeleton, progress, calendar

---

# Accesibilidad

## Reglas

* WCAG AA mínimo
* navegación completa por teclado
* contraste accesible (ratio 4.5:1 mínimo)
* `aria-label` obligatorio en icon buttons
* shadcn-svelte como base (construido sobre Bits UI con accesibilidad nativa)
* focus indicators visibles
* skip-to-content link

---

# Formateo e i18n (Español Bolivia)

## Locale por defecto

* Idioma primario: **es** (español)
* Timezone: **America/La_Paz**
* date-fns locale `es` para formateo

## Formatters de dominio

```ts
// src/lib/utils/formatters.ts
export function formatDate(date: string | Date): string {
    return format(new Date(date), "dd/MM/yyyy HH:mm:ss", { locale: es });
}

export function formatNetworkSpeed(bps: number): string {
    if (bps > 1_000_000_000) return `${(bps / 1_000_000_000).toFixed(2)} Gbps`;
    if (bps > 1_000_000) return `${(bps / 1_000_000).toFixed(2)} Mbps`;
    if (bps > 1_000) return `${(bps / 1_000).toFixed(2)} Kbps`;
    return `${bps} bps`;
}

export function formatLatency(ms: number): string {
    return `${ms.toFixed(2)} ms`;
}

export function formatBytes(bytes: number): string {
    const units = ["B", "KB", "MB", "GB", "TB"];
    let i = 0;
    while (bytes >= 1024 && i < units.length - 1) {
        bytes /= 1024;
        i++;
    }
    return `${bytes.toFixed(2)} ${units[i]}`;
}

export function formatPercent(value: number): string {
    return `${value.toFixed(1)}%`;
}
```

## Términos técnicos del dominio

| Técnico (código) | Español (UI)                |
| ---------------- | --------------------------- |
| switch           | Switch                      |
| router           | Router                      |
| access_point     | Punto de Acceso             |
| firewall         | Firewall                    |
| bandwidth_saturation | Saturación de Ancho de Banda |
| packet_loss      | Pérdida de Paquetes         |
| intrusion        | Intrusión / Dispositivo No Autorizado |
| topology         | Topología de Red            |
| agent            | Agente de Monitoreo         |

---

# Testing frontend

## Capas

| Tipo        | Herramienta     | Cuándo corre     |
| ----------- | --------------- | ---------------- |
| Unit        | Vitest          | Pre-commit, CI   |
| Componentes | Testing Library | CI               |
| E2E         | Playwright      | CI only          |

```bash
pnpm test        # Vitest
pnpm test:e2e    # Playwright
```

---

# Seguridad frontend

## Headers CSP

```ts
// hooks.server.ts
const csp = `
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
connect-src 'self' https://api.tudominio.gob.bo;
img-src 'self' data: blob:;
font-src 'self';
frame-ancestors 'none';
base-uri 'self';
form-action 'self';
`.replace(/\s+/g, " ").trim();

export async function handle({ event, resolve }) {
    const response = await resolve(event);
    response.headers.set("Content-Security-Policy", csp);
    response.headers.set("X-Frame-Options", "DENY");
    response.headers.set("X-Content-Type-Options", "nosniff");
    response.headers.set("Referrer-Policy", "strict-origin-when-cross-origin");
    return response;
}
```

## Reglas

* Nunca guardar tokens en localStorage sin encriptar (subtle crypto)
* Cookies HttpOnly obligatorias para SSR auth
* Sanitizar HTML dinámico (DOMPurify si es necesario)
* Rate limit desde backend (ADR 0009)
* Validar todos los inputs con ArkType antes de enviar

---

# Observabilidad frontend

## Estrategia

* request_id propagado desde backend (header X-Request-Id)
* errores enviados a Sentry (opcional)
* logs estructurados en development
* métricas Web Vitals (LCP, FID, CLS)

---

# Optimización de build

```ts
// vite.config.ts
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
    plugins: [tailwindcss(), sveltekit()],
    build: {
        target: "es2022",
        sourcemap: false,
    },
});
```

## Estrategias

* code splitting automático (Vite)
* lazy imports para gráficos y admin
* tree shaking
* imágenes optimizadas (WebP/AVIF)
* prerender parcial para páginas públicas

---

# Presupuesto de performance

| Métrica       | Objetivo    |
| ------------- | ----------- |
| JS inicial    | <200KB gzip |
| Lighthouse    | >90         |
| Primera carga | <2s         |
| TTFB local    | <300ms      |
| Hydration     | <100ms      |

---

# PWA y Service Worker (ADR 0021)

```ts
// src/service-worker.ts
const CACHE = "redes-beni-v1";
const ASSETS = [
    "/",
    "/dashboard",
    "/manifest.json",
    "/favicon.png",
    // static assets
];

self.addEventListener("install", (event) => {
    event.waitUntil(caches.open(CACHE).then((cache) => cache.addAll(ASSETS)));
    self.skipWaiting();
});

self.addEventListener("fetch", (event) => {
    // Network-first para API calls
    if (event.request.url.includes("/api/")) {
        event.respondWith(
            fetch(event.request).catch(() => caches.match(event.request))
        );
        return;
    }

    // Cache-first para static assets
    event.respondWith(
        caches.match(event.request).then((response) => {
            return response || fetch(event.request);
        })
    );
});
```

## Web App Manifest

```json
// static/manifest.json
{
    "name": "Redes Beni — Monitoreo de Infraestructura",
    "short_name": "Redes Beni",
    "start_url": "/dashboard",
    "display": "standalone",
    "background_color": "#0f172a",
    "theme_color": "#0f172a",
    "icons": [
        { "src": "/icon-192.png", "sizes": "192x192" },
        { "src": "/icon-512.png", "sizes": "512x512" }
    ]
}
```

---

# Docker deployment

```dockerfile
# Containerfile (Node 26, no 22)
FROM node:26.1.0-alpine AS builder

WORKDIR /app

COPY package.json pnpm-lock.yaml ./

RUN corepack enable && pnpm install --frozen-lockfile

COPY . .

RUN pnpm build

FROM node:26.1.0-alpine

WORKDIR /app

ENV NODE_ENV=production

COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./
COPY --from=builder /app/node_modules ./node_modules

EXPOSE 3000

CMD ["node", "build"]
```

**Nota:** Se utiliza Node 26.1.0 (Current, mayo 2026) para alinearse con ADR 0012. Para producción conservadora, usar Node 24 LTS.

---

# Arquitectura backend/frontend

SvelteKit NO actúa como Backend For Frontend (BFF).

El backend Rust mantiene:

* autenticación (PASETO, argon2id)
* lógica de negocio (hexagonal)
* REST API + OpenAPI
* realtime (SSE)
* permisos RBAC
* auditoría

SvelteKit funciona únicamente como:

* SSR frontend
* routing
* rendering
* hydration
* experiencia SPA
* cache cliente (TanStack Query)
* operación offline (Local-First)

---

# Alternativas consideradas

| Opción          | Motivo de descarte                                      |
| --------------- | ------------------------------------------------------- |
| React + Next.js | Overhead innecesario, bundle más pesado                 |
| Astro           | Excelente SSR pero menos ideal para dashboards realtime interactivos |
| Vue/Nuxt        | Ecosistema menos alineado con el stack Rust/Svelte del proyecto |
| Angular         | Demasiado pesado para VPS pequeños                      |
| ConnectRPC/gRPC | Incompatible con stack REST/OpenAPI/Axum establecido (ADR 0003, ADR 0016) |

---

# Herramientas y Librerías (Edición 2026)

| Herramienta              | Propósito                              | Versión |
| ------------------------ | -------------------------------------- | ------- |
| `vite-bundle-visualizer` | Analizar tamaño del bundle             | latest |
| `playwright`             | E2E testing                            | 1.60.0 |
| `vitest`                 | Unit testing rápido                    | 4.1.6 |
| `svelte-check`           | Validación TypeScript/Svelte           | latest |
| `@lucide/svelte`          | Iconografía (Svelte 5)                 | latest |
| `layerchart`             | Gráficos Svelte (line, area, pie, graph) | 2.0.0-next.63 |
| `openapi-typescript`     | Tipos TypeScript desde OpenAPI         | latest |
| `tanstack-svelte-query` | Cache y data fetching                  | 6.1.28 |
| `@tanstack/svelte-table` | Tablas con sorting/filtering/pagination | 9.0.0-alpha.47 |
| `arktype`                | Validación runtime type-safe           | 2.2.0 |
| `date-fns`               | Formateo de fechas con timezone        | latest |
| `sonner`                 | Toasts no bloqueantes                  | latest |
| `eslint-plugin-svelte`   | Calidad de código                      | latest |

**Cambios respecto a v2.1:**
- **Iconos:** `lucide-svelte` reemplazado por **`@lucide/svelte`** (paquete oficial para Svelte 5). `lucide-svelte` solo soporta Svelte 3/4.
- **Tablas:** `@tanstack/svelte-table` v8 solo soporta Svelte 3/4. Para Svelte 5 se usa la **v9 alpha** (`9.0.0-alpha.47`).
- **pnpm** actualizado a **11.1** (mayo 2026). Requiere Node.js 22+; compatible con Node 26.
- **Playwright** actualizado a **1.60.0** (mayo 2026).
- **Vitest** actualizado a **4.1.6** (mayo 2026).
- **Node.js** actualizado a **26.1.0** (Current, mayo 2026) / **24** LTS para producción.

---

# Consecuencias

## ✅ Positivas

* Bundle extremadamente pequeño (Svelte no tiene runtime pesado)
* Excelente rendimiento en hardware limitado (VPS 1GB)
* Menor consumo RAM que React/Next
* Realtime eficiente para dashboards (SSE nativo)
* Tipos compartidos automáticamente con backend vía OpenAPI
* SSR mejora UX inicial y seguridad
* Curva de mantenimiento menor (menos boilerplate)
* Operación offline robusta para sedes del Beni (ADR 0021)
* PWA permite instalación como app nativa en dispositivos de campo

## ⚠️ Negativas / Trade-offs

* Svelte 5 Runes aún está evolucionando
  → Mitigado con arquitectura simple y tipado fuerte

* Menor cantidad de librerías que React
  → El ecosistema actual (shadcn-svelte, LayerChart, TanStack) ya cubre dashboards modernos

* Menos developers disponibles en mercado
  → El código Svelte suele ser más corto y fácil de entender

* Local-First añade complejidad frontend
  → Necesaria para operación en sedes con conectividad inestable (ADR 0021)

* `@tanstack/svelte-table` v9 aún en alpha
  → Riesgo menor; la API es estable y se mantiene bajo seguimiento. Alternativa: `tanstack-table-8-svelte-5` comunitario.

---

# Decisiones derivadas

* Todo el frontend usa TypeScript strict
* **REST + OpenAPI** es la única forma de comunicación frontend/backend (no ConnectRPC)
* No usar Redux/Zustand/MobX (Svelte 5 Runes + TanStack Query son suficientes)
* El frontend se despliega en contenedor separado (Coolify, ADR 0019)
* Cookies HttpOnly para SSR auth; localStorage encriptado para persistencia multi-tab
* SSE es preferido sobre WebSockets (ADR 0020)
* `pnpm check` y `svelte-check` corren en CI
* Playwright corre solo en CI
* Vitest corre en pre-push
* `pnpm build` obligatorio antes de deploy
* Los tipos generados por `openapi-typescript` nunca se editan manualmente
* ArkType valida todo input de formularios
* LayerChart es la librería oficial de gráficos del proyecto
* shadcn-svelte es la base de componentes UI accesibles
* El Service Worker implementa network-first para API y cache-first para assets
* Los formatters de red (Mbps, ms, bytes) usan locale es_BO
* `@lucide/svelte` es el paquete oficial de iconos para Svelte 5

---

# Historial de cambios

| Versión | Fecha       | Cambios realizados |
| ------- | ----------- | ------------------ |
| 1.0     | 2026 (orig) | Versión inicial con ConnectRPC, buf generate, zod, Node 22 |
| 2.0     | 2026-05-16  | Reemplaza ConnectRPC/buf por REST/OpenAPI; reemplaza zod por ArkType; agrega shadcn-svelte, TanStack Query/Table, LayerChart, date-fns; agrega Local-First, PWA, Service Worker; agrega secciones de agentes, topología, formateo Bolivia; actualiza Node a v24 |
| 2.1     | 2026-05-16  | Actualiza Svelte a 5.55.0, SvelteKit a 2.57.0, Vitest a 4.1, Playwright a 1.59, LayerChart a 2.0.0-next.63, ArkType a 2.2.0, TanStack Svelte Query a 6.1.28, Lucide a 0.564.0, Node a v26 Current/v24 LTS, pnpm a 10.27 |
| 2.2     | 2026-05-16  | **Correcciones críticas de versiones:** reemplaza `lucide-svelte` por `@lucide/svelte` (Svelte 5); actualiza pnpm a 11.1; Playwright a 1.60.0; Vitest a 4.1.6; Node a 26.1.0; aclara `@tanstack/svelte-table` v9 alpha para Svelte 5. |

---

## Registro de cambios de versiones

| Fecha | Componente | Anterior | Actual | Notas |
|-------|------------|----------|--------|-------|
| 2026-05-16 | Svelte | 5.45.0 | **5.55.0** | Tipos exportados desde `svelte/motion`. Última estable (may 2026). |
| 2026-05-16 | SvelteKit | 2.x | **2.57.0** | Breaking changes en Remote Functions (requerir permiso servidor para refrescar queries). |
| 2026-05-16 | Vitest | 3.x | **4.1.6** | Test Tags, context builder pattern, browser mode estable, Vite-native. Última patch (may 2026). |
| 2026-05-16 | Playwright | 1.x | **1.60.0** | Screencast API, CLI debugger, async disposables, Chrome for Testing default. Última estable (may 2026). |
| 2026-05-16 | LayerChart | 1.x | **2.0.0-next.63** | Próximo a v2 estable. Última next (may 2026). |
| 2026-05-16 | ArkType | 1.x | **2.2.0** | TypeScript-syntax validation. Última estable (mar 2026). |
| 2026-05-16 | TanStack Svelte Query | 5.x | **6.1.28** | Última estable (may 2026). |
| 2026-05-16 | TanStack Svelte Table | 8.x | **9.0.0-alpha.47** | v8 no soporta Svelte 5. v9 alpha es la versión compatible. |
| 2026-05-16 | Lucide | `lucide-svelte` | **`@lucide/svelte`** | Paquete oficial para Svelte 5. `lucide-svelte` es solo para Svelte 3/4. |
| 2026-05-16 | Node.js | 24 | **26.1.0** (Current) / **24** LTS | Node 26.1.0 Current (may 2026). Node 24 LTS estable para producción. |
| 2026-05-16 | pnpm | 10.27 | **11.1** | Última estable (may 2026). Requiere Node 22+. |
