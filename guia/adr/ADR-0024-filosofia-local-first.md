# ADR 0024 — Arquitectura Local-First: SQLite Wasm + Sync Queue + Offline-First

| Campo               | Valor                                                                                         |
| ------------------- | --------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado — implementación en Fase 2                                                         |
| **Fecha**           | 2026                                                                                          |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                              |
| **Relacionado con** | ADR 0004 (SQLite), ADR 0022 (SvelteKit + Svelte 5), ADR 0018 (Jobs), ADR 0035 (Monitoreo Regional) |

---

# Contexto

Las aplicaciones cloud-first tradicionales fallan completamente cuando la conexión a internet
es lenta, inestable o inexistente.

Para Bolivia y entornos regionales con conectividad intermitente, esto es inaceptable.

El sistema de monitoreo regional debe continuar funcionando incluso cuando:

* no hay internet
* la red gubernamental falla
* el backend está temporalmente inaccesible
* el usuario trabaja desde zonas remotas

Además:

* la experiencia debe sentirse instantánea
* los datos deben pertenecer primero al usuario/dispositivo
* la sincronización debe ser automática y silenciosa
* el frontend debe seguir siendo extremadamente ligero

---

# Decisión

Adoptar una arquitectura **Local-First** usando:

* **SQLite Wasm + OPFS** en navegador
* **Sync Queue** persistente
* sincronización eventual con backend Rust
* **Service Workers** para offline total
* estrategia **offline-first**
* **optimistic UI** por defecto

---

# Principios Local-First

| Principio          | Descripción                                   |
| ------------------ | --------------------------------------------- |
| Local primero      | El dispositivo responde antes que el servidor |
| Sync después       | Internet sincroniza, no bloquea               |
| Offline total      | La app sigue funcionando sin red              |
| Latencia cero      | Operaciones instantáneas                      |
| Persistencia local | Datos sobreviven reinicios                    |
| Resiliencia        | El sistema tolera desconexiones               |

---

# Arquitectura general

```text
┌───────────────────────────────────────┐
│              Frontend                 │
│         SvelteKit + Svelte 5          │
├───────────────────────────────────────┤
│                                       │
│  SQLite Wasm + OPFS                   │
│      │                                │
│      ├── tablas locales               │
│      ├── cache                        │
│      └── sync_queue                   │
│                                       │
│  Service Worker                       │
│      │                                │
│      ├── cache app shell              │
│      ├── offline mode                 │
│      └── retry sync                   │
│                                       │
└───────────────────────────────────────┘
                  │
                  ▼
┌───────────────────────────────────────┐
│             Backend Rust              │
│         Axum + SQLite/Postgres        │
└───────────────────────────────────────┘
```

---

# Flujo de datos

```text
Usuario modifica dato
    ↓
SQLite local guarda instantáneamente
    ↓
UI actualiza inmediatamente
    ↓
Sync Queue registra operación
    ↓
Background Sync intenta enviar cambios
    ↓
Servidor confirma sincronización
    ↓
Registro marcado como sincronizado ✓
```

---

# Objetivos de rendimiento

| Métrica          | Objetivo             |
| ---------------- | -------------------- |
| Escritura local  | <1ms                 |
| Lectura local    | instantánea          |
| Arranque offline | <2s                  |
| Reintento sync   | automático           |
| Persistencia     | sobreviven reinicios |

---

# Persistencia local — SQLite Wasm

## Motivación

IndexedDB puro es complejo y poco ergonómico para consultas avanzadas.

SQLite Wasm permite:

* SQL real
* índices
* joins
* queries complejas
* mismo modelo mental backend/frontend

---

# Inicialización de SQLite

```ts
// src/lib/local-db.ts
import { createDbWorker } from 'wa-sqlite';

export class LocalDatabase {
    private worker;

    async init() {
        this.worker = await createDbWorker();
    }
}
```

---

# Uso de OPFS

## Decisión

Usar:

```text
Origin Private File System (OPFS)
```

para persistencia real en disco.

## Beneficios

| Beneficio     | Resultado                |
| ------------- | ------------------------ |
| Persistencia  | Datos sobreviven recarga |
| Velocidad     | Cercana a SQLite nativo  |
| Seguridad     | Sandbox navegador        |
| Escalabilidad | Bases grandes posibles   |

---

# Cola de sincronización

## Tabla sync_queue

```sql
CREATE TABLE sync_queue (
    id          TEXT PRIMARY KEY,
    table_name  TEXT NOT NULL,
    operation   TEXT NOT NULL,
    payload     TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    synced_at   TEXT
);
```

---

# Operaciones soportadas

| Operación | Descripción    |
| --------- | -------------- |
| INSERT    | nuevo registro |
| UPDATE    | modificación   |
| DELETE    | eliminación    |

---

# Enqueue automático

```ts
async enqueueSync(
    table: string,
    operation: string,
    payload: unknown,
) {
    await db.exec(
        `INSERT INTO sync_queue (...) VALUES (...)`
    );
}
```

---

# Optimistic UI

La UI SIEMPRE actualiza primero localmente.

## Flujo

```text
click usuario
    ↓
actualización inmediata UI
    ↓
persistencia local
    ↓
sync en background
```

El usuario nunca espera internet para interactuar.

---

# Estrategia offline-first

## Reglas

| Situación       | Comportamiento   |
| --------------- | ---------------- |
| Online          | Sync automático  |
| Offline         | Operación local  |
| Reconexión      | Retry automático |
| Backend caído   | Cola persiste    |
| Refresh browser | Datos sobreviven |

---

# Service Worker

## Objetivo

Permitir:

* app completamente offline
* cache de assets
* cache de rutas críticas
* background sync

---

# Implementación base

```ts
// service-worker.ts
const CACHE = 'lab-cache-v1';

self.addEventListener('install', event => {
    event.waitUntil(
        caches.open(CACHE)
            .then(c => c.addAll([
                '/',
                '/dashboard',
                '/login',
            ]))
    );
});
```

---

# Estrategia de cache

| Recurso   | Estrategia             |
| --------- | ---------------------- |
| App shell | Cache First            |
| API       | Network First          |
| Assets    | Cache First            |
| Dashboard | Stale While Revalidate |

---

# Resolución de conflictos

## MVP

Usar:

```text
Last-Write-Wins (LWW)
```

basado en:

```text
updated_at UTC
```

---

# Casos de conflicto

| Escenario             | Resolución             |
| --------------------- | ---------------------- |
| mismo usuario offline | último cambio gana     |
| dos usuarios editando | último sync gana       |
| delete vs update      | delete tiene prioridad |

---

# Campos obligatorios

Todos los modelos offline deben incluir:

```sql
updated_at TEXT NOT NULL
```

en UTC.

---

# Estrategia futura — CRDTs

## Fase futura

Evaluar:

* Automerge
* Yjs
* CRDT replication

si existe edición colaborativa compleja.

---

# Estado de sincronización en UI

## Obligatorio

La UI debe mostrar:

| Estado       | Indicador |
| ------------ | --------- |
| Sincronizado | ✓         |
| Pendiente    | ⟳         |
| Error        | ⚠         |
| Offline      | 📴        |

El usuario siempre debe saber el estado real.

---

# Seguridad

## Reglas

* datos sensibles cifrados localmente
* nunca guardar secrets del backend
* auth sigue usando cookies HttpOnly
* SQLite local pertenece al usuario actual

---

# Limpieza automática

## Política

La cola `sync_queue` elimina registros:

```text
>30 días sincronizados
```

usando:

```text
CleanupJob (ADR 0018)
```

---

# Compatibilidad navegador

| Navegador    | Soporte |
| ------------ | ------- |
| Chrome 89+   | ✅       |
| Firefox 89+  | ✅       |
| Safari 15+   | ✅       |
| Edge moderno | ✅       |

---

# Fallback

Si Wasm no está disponible:

```text
modo online puro
```

La app sigue funcionando.

---

# Integración con Monitoreo Regional (ADR 0035)

El patrón Local-First es esencial para ADR 0035:

| Escenario | Persistencia  |
| --------- | ------------- |
| Sedes con conectividad inestable | SQLite local + sync queue |
| Dashboard web | SQLite + SSE |
| Agentes en sedes remotas | SQLite + buffering offline |

---

# Estrategia de sincronización

## Background Sync

```text
cada X segundos:
    enviar pendientes
    recibir cambios
    reconciliar estados
```

---

# Reintentos

## Política

| Error        | Acción            |
| ------------ | ----------------- |
| Sin internet | retry exponencial |
| 500 backend  | retry             |
| 401 auth     | refresh token     |
| conflicto    | merge             |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta      | Propósito               |
| ---------------- | ----------------------- |
| `wa-sqlite`      | SQLite WebAssembly      |
| `OPFS`           | Persistencia rápida     |
| `PowerSync`      | Motor de sincronización |
| `Automerge`      | CRDTs futuros           |
| `TanStack Query` | Estado sync UI          |
| `workbox`        | Service workers         |
| `idb`            | Fallback IndexedDB      |
| `zod`            | Validación runtime      |

---

# Alternativas consideradas

| Opción           | Motivo de descarte  |
| ---------------- | ------------------- |
| Cloud-only       | Inaceptable offline |
| IndexedDB puro   | API compleja        |
| PouchDB/CouchDB  | Overhead excesivo   |
| Firebase offline | Vendor lock-in      |
| Realm            | Ecosistema pesado   |

---

# Consecuencias

## ✅ Positivas

* La app funciona completamente offline
* UX extremadamente rápida
* Latencia prácticamente cero
* Resistente a caídas de red
* Menor carga al backend
* Misma filosofía SQLite frontend/backend
* Excelente experiencia en Bolivia

---

## ⚠️ Negativas / Trade-offs

### Complejidad de sincronización

La sincronización distribuida siempre añade complejidad.

→ Mitigado con LWW simple en MVP.

---

### WebAssembly requerido

SQLite Wasm depende de navegadores modernos.

→ Existe fallback online-only.

---

### Riesgo de conflictos

LWW puede sobrescribir cambios concurrentes.

→ Aceptable para Fase 2 y equipos pequeños.

---

### Storage local limitado

El navegador puede limitar almacenamiento.

→ Cleanup automático y sync incremental.

---

# Decisiones derivadas

* Todos los modelos offline tienen `updated_at`
* La app debe funcionar sin internet
* El estado de sync es obligatorio en UI
* Service Workers son parte del core
* SQLite local es la fuente primaria temporal
* La sincronización es eventual, no inmediata
* Los datos críticos siguen persistiendo en backend
* `CleanupJob` limpia colas antiguas
* La arquitectura prepara naturalmente operación offline para sedes remotas (ADR 0035)
