# ADR 0023 — Internacionalización: Paraglide JS + SvelteKit i18n + Formatters Bolivia

| Campo               | Valor                                                                      |
| ------------------- | -------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                 |
| **Fecha**           | 2026                                                                       |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                           |
| **Relacionado con** | ADR 0022 (SvelteKit + Svelte 5), ADR 0027 (ConnectRPC), ADR 0010 (Testing) |

---

# Contexto

El sistema de monitoreo regional de la Gobernación del Beni nace en español, pero debe estar
preparado para operar en múltiples idiomas sin reescrituras futuras.

Los problemas clásicos de i18n en aplicaciones frontend modernas son:

* strings mágicos tipo `t("welcome")` que rompen silenciosamente
* bundles gigantes descargando todos los idiomas
* fechas y monedas incorrectas para Bolivia
* SEO roto por rutas sin locale
* traducciones incompletas detectadas recién en producción
* falta de tipado entre frontend y mensajes

Además:

* el proyecto prioriza performance extrema en VPS pequeños
* el frontend usa SvelteKit + TypeScript strict
* el stack busca ser IA-ready y compile-time first

---

# Decisión

Adoptar:

* **Paraglide JS** (ecosistema inlang) para traducciones compiladas
* **SvelteKit i18n** para routing por idioma
* **Intl API** nativa para localización regional Bolivia
* **TypeScript strict mode** obligatorio
* generación automática de mensajes tipados

---

# Objetivos del sistema i18n

| Objetivo           | Estrategia                         |
| ------------------ | ---------------------------------- |
| Seguridad de tipos | Traducciones compiladas            |
| Bundle pequeño     | Solo cargar locale activo          |
| SEO correcto       | Rutas por idioma                   |
| UX regional        | Formatters Bolivia                 |
| DX moderna         | Autocomplete + compile-time errors |
| Escalabilidad      | Agregar idiomas sin refactors      |

---

# Arquitectura i18n

```text
apps/web/
├── messages/
│   ├── es.json
│   ├── en.json
│   └── pt.json
│
├── src/
│   ├── paraglide/        # generado automáticamente
│   │
│   ├── lib/
│   │   ├── i18n/
│   │   │   ├── formatters.ts
│   │   │   ├── locale.ts
│   │   │   └── currency.ts
│   │   │
│   │   └── components/
│   │
│   ├── routes/
│   │   ├── dashboard/
│   │   └── en/
│   │
│   └── hooks.server.ts
│
├── project.inlang/
└── package.json
```

---

# Paraglide JS — traducciones compiladas

En lugar de usar strings dinámicos en runtime, Paraglide genera funciones TypeScript.

## Ejemplo generado

```ts
// generado automáticamente — NO editar
export const welcome_user = (params: { name: string }) =>
    `Bienvenido ${params.name}`;

export const dashboard_title = () =>
    `Panel de monitoreo`;
```

## Uso en Svelte

```svelte
<script lang="ts">
    import * as m from '$paraglide/messages';

    export let user: string;
</script>

<h1>{m.welcome_user({ name: user })}</h1>
```

---

# Beneficio principal: errores en compilación

Si una clave cambia:

```ts
m.dashboard_title()
```

y el mensaje fue eliminado:

```bash
❌ Property 'dashboard_title' does not exist
```

El build falla inmediatamente.

No existen errores silenciosos en producción.

---

# Configuración de SvelteKit i18n

```ts
// svelte.config.ts
import { paraglideVitePlugin } from '@inlang/paraglide-js';

export default {
    kit: {
        adapter: adapterNode(),
    },

    vitePlugin: {
        plugins: [
            paraglideVitePlugin({
                project: './project.inlang',
                outdir: './src/paraglide',
            }),
        ],
    },
};
```

---

# Estrategia de rutas

| Idioma            | URL             |
| ----------------- | --------------- |
| Español (default) | `/dashboard`    |
| Inglés            | `/en/dashboard` |
| Portugués         | `/pt/dashboard` |

## Reglas

* español es el idioma por defecto
* el locale default NO usa prefijo
* SEO usa URLs separadas por idioma
* `<html lang="">` se define automáticamente

---

# Formateadores regionales Bolivia (L10n)

## Moneda

```ts
// src/lib/i18n/formatters.ts
const LOCALES = {
    es: 'es-BO',
    en: 'en-US',
    pt: 'pt-BR',
} as const;

export function formatCurrency(
    amount: number,
    locale: keyof typeof LOCALES,
) {
    return new Intl.NumberFormat(LOCALES[locale], {
        style: 'currency',
        currency: locale === 'es' ? 'BOB' : 'USD',
    }).format(amount);
}
```

---

# Fechas y timezone Bolivia

```ts
export function formatDate(
    iso: string,
    locale: keyof typeof LOCALES,
) {
    return new Intl.DateTimeFormat(LOCALES[locale], {
        dateStyle: 'short',
        timeStyle: 'short',
        timeZone: 'America/La_Paz',
    }).format(new Date(iso));
}
```

## Reglas

* la DB siempre guarda UTC
* conversión al timezone ocurre en cliente
* Bolivia usa `America/La_Paz`

---

# Números regionales

```ts
export function formatNumber(
    value: number,
    locale: keyof typeof LOCALES,
) {
    return new Intl.NumberFormat(
        LOCALES[locale]
    ).format(value);
}
```

---

# Detección de idioma

## Prioridad

| Prioridad | Fuente                   |
| --------- | ------------------------ |
| 1         | URL (`/en/...`)          |
| 2         | Cookie `locale`          |
| 3         | Header `Accept-Language` |
| 4         | Fallback `es`            |

---

# Middleware de locale

```ts
// hooks.server.ts
export async function handle({ event, resolve }) {
    const locale =
        event.cookies.get('locale')
        ?? 'es';

    event.locals.locale = locale;

    return resolve(event);
}
```

---

# Cambio de idioma

```svelte
<script lang="ts">
    import { setLocale } from '$lib/i18n/locale';

    function changeLocale(locale: string) {
        setLocale(locale);
    }
</script>

<button onclick={() => changeLocale('en')}>
    English
</button>
```

---

# Agregar un nuevo idioma

## Paso 1

```bash
apps/web/messages/fr.json
```

## Paso 2

Agregar locale:

```ts
locales: ['es', 'en', 'pt', 'fr']
```

## Paso 3

Regenerar:

```bash
just types
```

---

# Validación estricta

Si falta una clave:

```bash
❌ Missing translation:
dashboard_title
```

El build falla automáticamente.

---

# Integración con CI

```bash
just types
pnpm check
svelte-check
```

CI bloquea:

* traducciones faltantes
* claves inválidas
* errores TypeScript
* locales incompletos

---

# Comparativa: Paraglide vs i18next

| Característica | i18next        | Paraglide    |
| -------------- | -------------- | ------------ |
| Tipado         | Parcial        | Completo     |
| Bundle         | Runtime pesado | Ultra ligero |
| Errores        | Runtime        | Compile-time |
| Autocomplete   | Limitado       | Completo     |
| DX             | Buena          | Excelente    |
| Tree shaking   | Limitado       | Sí           |

---

# Estrategia de performance

## Objetivos

| Métrica               | Objetivo           |
| --------------------- | ------------------ |
| JS extra por i18n     | <10KB              |
| Cambio de idioma      | instantáneo        |
| Runtime i18n          | mínimo             |
| Traducciones cargadas | solo locale activo |

---

# Seguridad

## Reglas

* nunca interpolar HTML inseguro
* sanitizar contenido dinámico
* traducciones solo desde archivos versionados
* locales válidos definidos explícitamente

---

# Testing i18n

## Unit tests

```bash
pnpm test
```

## Verificaciones

* todas las claves existen
* formatters correctos
* fechas Bolivia válidas
* currencies correctas
* fallback locale funcional

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta            | Propósito               |
| ---------------------- | ----------------------- |
| `@inlang/paraglide-js` | Traducciones compiladas |
| `i18n-ally`            | DX en VS Code           |
| `zod-i18n-map`         | Validación traducida    |
| `typesafe-i18n`        | Alternativa tipada      |
| `Intl API`             | Formateo nativo         |
| `svelte-check`         | Validación Svelte/TS    |
| `vitest`               | Testing i18n            |
| `msw`                  | Mocking de locale APIs  |

---

# Alternativas consideradas

| Opción      | Motivo de descarte       |
| ----------- | ------------------------ |
| i18next     | Runtime pesado           |
| lingui      | Menor integración Svelte |
| vue-i18n    | Ecosistema Vue           |
| next-intl   | Dependencia React        |
| JSON manual | Sin tipado               |

---

# Consecuencias

## ✅ Positivas

* Traducciones type-safe
* Errores detectados antes de producción
* Bundle extremadamente pequeño
* SEO correcto por idioma
* Excelente DX
* Compatible con SSR y SPA
* Adaptado a Bolivia (`es-BO`, BOB, timezone)

---

## ⚠️ Negativas / Trade-offs

### Curva de aprendizaje

Paraglide usa un paradigma distinto al clásico `t("key")`.

→ Mitigado con autocomplete y compile-time safety.

---

### Paso extra de generación

Las traducciones deben compilarse antes del build.

→ `just types` automatiza el proceso.

---

### Traducciones obligatorias

No se puede activar un idioma incompleto.

→ Esto previene producción rota.

---

# Decisiones derivadas

* `es` es el locale por defecto
* `America/La_Paz` es el timezone oficial
* Todas las fechas se guardan en UTC
* `just types` incluye generación Paraglide
* Los archivos en `src/paraglide/` son generados automáticamente
* Nunca editar archivos generados manualmente
* Todo el frontend usa TypeScript strict
* Los locales válidos se definen explícitamente
* El frontend nunca usa strings hardcodeados para UI crítica
