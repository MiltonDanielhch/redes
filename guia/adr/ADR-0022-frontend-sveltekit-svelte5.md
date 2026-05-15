````md
# ADR 0022 — Frontend: SvelteKit + Svelte 5 Runes + ConnectRPC

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0003 (Axum), ADR 0010 (Testing), ADR 0021 (OpenAPI), ADR 0027 (ConnectRPC), ADR 0035 (Monitoreo Regional) |

---

# Contexto

El sistema de monitoreo regional de infraestructura y redes de la Gobernación del Beni requiere:

- dashboards reactivos en tiempo real
- consumo mínimo de RAM/CPU
- carga rápida incluso en conexiones lentas
- SSR para rendimiento inicial
- tipado compartido con backend Rust
- experiencia fluida tipo SPA después de la carga inicial
- mantenimiento simple para un equipo pequeño
- accesibilidad compatible con estándares gubernamentales
- degradación controlada ante fallos de conectividad

React/Next.js añade complejidad y overhead innecesario para este caso de uso.

---

# Decisión

Usar:

- **SvelteKit** como framework frontend fullstack
- **Svelte 5** con sistema de **Runes**
- **ConnectRPC-Web** como cliente tipado
- **TypeScript strict mode**
- **TailwindCSS** para UI
- **adapter-node** para despliegue Docker/Kamal
- **SSE (Server-Sent Events)** para tiempo real por defecto

---

# Arquitectura frontend

```text
apps/web/
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/
│   │   │   └── features/
│   │   │
│   │   ├── rpc/
│   │   ├── stores/
│   │   ├── utils/
│   │   ├── auth/
│   │   ├── realtime/
│   │   ├── charts/
│   │   └── types/
│   │
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── login/
│   │   ├── dashboard/
│   │   │   ├── +layout.svelte
│   │   │   ├── dispositivos/
│   │   │   └── alertas/
│   │   │
│   │   └── reportes/
│   │
│   ├── hooks.server.ts
│   ├── app.html
│   └── app.d.ts
│
├── static/
├── tests/
└── package.json
````

---

# Política de componentes

| Carpeta               | Responsabilidad                              |
| --------------------- | -------------------------------------------- |
| `components/ui`       | Componentes reutilizables y presentacionales |
| `components/features` | Componentes específicos de negocio           |
| `routes/`             | Ensamblaje de páginas                        |
| `lib/utils`           | Helpers puros sin dependencias visuales      |

## Reglas

* Evitar lógica de negocio dentro de componentes visuales
* Los componentes UI no conocen RPC ni auth
* La lógica de realtime vive en `lib/realtime`

---

# Svelte 5 Runes

Se adopta el nuevo sistema de reactividad explícita de Svelte 5.

## Estado reactivo

```ts
let dispositivos = $state([]);
```

## Valores derivados

```ts
let dispositivosActivos = $derived(
    dispositivos.filter(d => d.online)
);
```

## Side effects controlados

```ts
$effect(() => {
    console.log("Dispositivos actualizados");
});
```

---

# SSR + SPA híbrido

| Estrategia   | Uso                      |
| ------------ | ------------------------ |
| SSR          | Login, dashboard inicial |
| CSR/SPA      | Navegación interna       |
| Streaming    | Métricas en tiempo real  |
| Lazy Loading | Gráficos pesados         |

---

# Estrategia de carga de datos

| Estrategia        | Uso                            |
| ----------------- | ------------------------------ |
| `+page.server.ts` | Datos sensibles o autenticados |
| `+page.ts`        | Datos cacheables               |
| Fetch cliente     | Tiempo real                    |
| SSE               | Métricas continuas             |

Ejemplo:

```ts
// +page.server.ts
export async function load({ locals }) {
    return {
        user: locals.user
    };
}
```

---

# Comunicación con backend

Se utilizará ConnectRPC-Web con generación automática de clientes TypeScript.

```bash
buf generate
```

Genera:

```text
src/lib/types/
```

Ejemplo:

```ts
const client = createPromiseClient(NetworkService, transport);

const response = await client.getDevices({});
```

---

# Estado global

## Reglas

| Tipo de estado  | Estrategia    |
| --------------- | ------------- |
| Estado local    | `$state`      |
| Estado derivado | `$derived`    |
| Auth            | Context API   |
| Cache RPC       | ConnectRPC    |
| Tiempo real     | SSE/WebSocket |

## Restricciones

* Evitar stores globales innecesarios
* No usar Redux/Zustand/MobX
* La cache pertenece al cliente RPC

---

# Autenticación

## Estrategia

* PASETO desde backend Rust
* Cookie HttpOnly Secure
* Refresh token rotativo
* Middleware SSR en `hooks.server.ts`

```ts
export async function handle({ event, resolve }) {
    const token = event.cookies.get("session");

    if (token) {
        event.locals.user = await validateToken(token);
    }

    return resolve(event);
}
```

## Reglas

* Nunca usar localStorage para tokens
* Cookies `Secure`, `HttpOnly`, `SameSite=Lax`
* Logout invalida sesión en backend

---

# Tiempo real

## Estrategia

Para métricas de monitoreo:

* Server-Sent Events (SSE) por defecto
* WebSockets solo si realmente se necesitan

## Motivos

* SSE consume menos recursos
* más simple
* suficiente para dashboards
* reconexión automática integrada en browser

## Política

| Tecnología | Uso                        |
| ---------- | -------------------------- |
| SSE        | Dashboards                 |
| Polling    | Fallback                   |
| WebSocket  | Comunicación bidireccional |

SSE es el default.
WebSocket requiere justificación arquitectónica.

---

# Degradación controlada

Si la conexión realtime falla:

* fallback automático a polling cada 30s
* mostrar último snapshot válido
* dashboard nunca debe quedar vacío
* notificación visual de estado degradado

---

# Manejo de errores

## Estrategia

* `ConnectError` como error estándar
* Error boundaries por layout
* Retry automático solo para GETs idempotentes
* Toasts no bloqueantes

```ts
import { ConnectError } from "@connectrpc/connect";

export function handleRpcError(error: unknown) {
    if (error instanceof ConnectError) {
        console.error(error.code, error.message);
    }
}
```

---

# UI y diseño

## Stack visual

| Herramienta  | Uso                    |
| ------------ | ---------------------- |
| TailwindCSS  | Layout/UI              |
| Lucide Icons | Iconografía            |
| LayerChart   | Gráficos               |
| Bits UI      | Componentes accesibles |

---

# Accesibilidad

## Reglas

* WCAG AA mínimo
* navegación completa por teclado
* contraste accesible
* `aria-label` obligatorio en icon buttons
* Bits UI preferido por accesibilidad nativa

---

# Testing frontend

## Capas

| Tipo        | Herramienta     |
| ----------- | --------------- |
| Unit        | Vitest          |
| Componentes | Testing Library |
| E2E         | Playwright      |

```bash
pnpm test
pnpm test:e2e
```

---

# Seguridad frontend

## Headers CSP

```ts
content-security-policy:
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
connect-src 'self' https://api.tudominio.gob.bo;
```

## Reglas

* Nunca guardar tokens en localStorage
* Cookies HttpOnly obligatorias
* Sanitizar HTML dinámico
* Rate limit desde backend

---

# Observabilidad frontend

## Estrategia

* request_id propagado desde backend
* errores enviados a Sentry
* logs estructurados en development
* métricas Web Vitals

---

# Optimización de build

```js
// vite.config.ts
export default defineConfig({
    build: {
        target: 'es2022',
        sourcemap: false,
    }
});
```

## Estrategias

* code splitting automático
* lazy imports
* tree shaking
* imágenes optimizadas
* prerender parcial

---

# Presupuesto de performance

| Métrica       | Objetivo    |
| ------------- | ----------- |
| JS inicial    | <200KB gzip |
| Lighthouse    | >90         |
| Primera carga | <2s         |
| TTFB local    | <300ms      |

---

# Docker deployment

```dockerfile
FROM node:22-alpine AS builder

WORKDIR /app

COPY package.json pnpm-lock.yaml ./

RUN corepack enable && pnpm install --frozen-lockfile

COPY . .

RUN pnpm build

FROM node:22-alpine

WORKDIR /app

ENV NODE_ENV=production

COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./

EXPOSE 3000

CMD ["node", "build"]
```

---

# Arquitectura backend/frontend

SvelteKit NO actúa como Backend For Frontend (BFF).

El backend Rust mantiene:

* autenticación
* lógica de negocio
* ConnectRPC
* realtime
* permisos
* auditoría

SvelteKit funciona únicamente como:

* SSR frontend
* routing
* rendering
* hydration
* experiencia SPA

---

# Alternativas consideradas

| Opción          | Motivo de descarte                                      |
| --------------- | ------------------------------------------------------- |
| React + Next.js | Overhead innecesario                                    |
| Astro           | Excelente SSR pero menos ideal para dashboards realtime |
| Vue/Nuxt        | Ecosistema menos alineado con el stack                  |
| Angular         | Demasiado pesado para VPS pequeños                      |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta              | Propósito                    |
| ------------------------ | ---------------------------- |
| `vite-bundle-visualizer` | Analizar tamaño del bundle   |
| `playwright`             | E2E testing                  |
| `vitest`                 | Unit testing rápido          |
| `svelte-check`           | Validación TypeScript/Svelte |
| `unplugin-icons`         | Íconos optimizados           |
| `msw`                    | Mocking de APIs para tests   |
| `eslint-plugin-svelte`   | Calidad de código            |
| `zod`                    | Validación runtime frontend  |

---

# Consecuencias

## ✅ Positivas

* Bundle extremadamente pequeño
* Excelente rendimiento en hardware limitado
* Menor consumo RAM que React/Next
* Realtime eficiente para dashboards
* Tipos compartidos automáticamente con backend
* SSR mejora UX inicial
* Curva de mantenimiento menor

## ⚠️ Negativas / Trade-offs

* Svelte 5 Runes aún está evolucionando
  → Mitigado con arquitectura simple y tipado fuerte

* Menor cantidad de librerías que React
  → El ecosistema actual ya cubre dashboards modernos

* Menos developers disponibles en mercado
  → El código Svelte suele ser más corto y fácil de entender

---

# Decisiones derivadas

* Todo el frontend usa TypeScript strict
* ConnectRPC es la única forma de comunicación frontend/backend
* No usar Redux/Zustand/MobX
* El frontend se despliega en contenedor separado
* Cookies HttpOnly obligatorias para auth
* SSE es preferido sobre WebSockets
* `pnpm check` y `svelte-check` corren en CI
* Playwright corre solo en CI
* Vitest corre en pre-push
* `pnpm build` obligatorio antes de deploy
* Los tipos generados por `buf generate` nunca se editan manualmente

```
```
