# ADR 0021 — Local-First y Sincronización Offline

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Versión** | 2.1 (Corrección 2026-05-16) |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0020 (Monitoreo Regional), ADR 0017 (SvelteKit Frontend), ADR 0008 (PASETO), ADR 0015 (Jobs), ADR 0022 (Agentes Distribuidos) |

---

## Contexto

La Gobernación del Beni opera en una región con conectividad inestable. Muchas sedes:

* Experimentan cortes de internet frecuentes
* Tienen latencia alta hacia el centro de datos
* Necesitan continuar operando durante fallas de red
* Requieren sincronización de datos cuando la conectividad se restablece

Sin una estrategia Local-First, el sistema sería inutilizable durante los cortes, lo cual es inaceptable para operaciones críticas de monitoreo de infraestructura.

---

## Decisión

Implementar una arquitectura **Local-First** donde:

1. El frontend (SvelteKit) mantiene una **base de datos SQLite en WebAssembly** con los datos críticos
2. Las operaciones de escritura se **encolan localmente** cuando no hay conectividad
3. Al reconectar, se **sincronizan** los cambios pendientes con el servidor
4. El agente de sede (ADR 0022) mantiene su propia **SQLite local** para buffering de métricas

---

## Tecnologías SQLite Wasm

| Opción | Versión | Propósito | Notas |
|--------|---------|-----------|-------|
| `@sqlite.org/sqlite-wasm` | `3.53.0-build1` | **Opción principal** — SQLite oficial en Wasm con OPFS | ES Module, soporte Worker, requiere headers COOP/COEP |
| `sql.js` | `1.14.1` | Alternativa simple — pure JS (Emscripten) | Sin OPFS, más maduro, menor bundle |

**Recomendación:** Usar `@sqlite.org/sqlite-wasm` 3.53.0-build1 como opción principal para nuevas implementaciones. Soporta Origin Private File System (OPFS) para persistencia real en el filesystem del browser, SharedArrayBuffer para concurrencia entre tabs, y la API oficial de SQLite. `sql.js` 1.14.1 es válido como fallback si OPFS no está disponible o se requiere compatibilidad máxima.

---

## Estrategia de Datos

### Datos Críticos (siempre locales)

| Dato | Estrategia | Justificación |
|------|-----------|---------------|
| Inventario de dispositivos | Cache completo | Necesario para operación offline |
| Topología de red | Cache completo | Visualización sin conexión |
| Alertas recientes | Cache últimas 100 | Notificación offline |
| Sesión de usuario | Cache + token PASETO | Auth sin conexión |
| Configuración | Cache completa | UI funcional offline |

### Datos Volátiles (no cacheados)

| Dato | Estrategia | Justificación |
|------|-----------|---------------|
| Métricas realtime | Solo online | Datos que cambian cada segundo |
| Logs de auditoría | Solo online | Origen de verdad en servidor |
| Reportes históricos | Bajo demanda | Pueden ser grandes |

---

## Arquitectura de Sync

```text
┌─────────────────────────────────────────────────────────────────┐
│  Frontend (SvelteKit)                                          │
│  ├─ SQLite Wasm (@sqlite.org/sqlite-wasm 3.53.0-build1)       │
│  │   ├─ devices_cache                                          │
│  │   ├─ sedes_cache                                            │
│  │   ├─ alerts_cache                                           │
│  │   ├─ sync_queue (pending_ops)                               │
│  │   └─ session_cache                                          │
│  │                                                              │
│  ├─ Sync Engine (apps/web/src/lib/sync/)                       │
│  │   ├─ detect_connectivity()                                  │
│  │   ├─ enqueue_operation(op)                                  │
│  │   ├─ process_queue()                                        │
│  │   └─ resolve_conflict(local, remote)                        │
│  │                                                              │
│  └─ Service Worker (src/service-worker.ts)                       │
│      └─ intercept fetch, cache assets, network-first API        │
└──────────────────────────┬──────────────────────────────────────┘
                           │ HTTPS / REST
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  API Axum                                                      │
│  ├─ POST /api/v1/sync/push (batch de operaciones)            │
│  ├─ GET /api/v1/sync/pull (cambios desde last_sync)          │
│  ├─ POST /api/v1/sync/resolve (resolución de conflictos)     │
│  └─ Webhook de sync completado                                 │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  PostgreSQL (origen de verdad)                                  │
│  ├─ sync_log (registro de operaciones sincronizadas)         │
│  ├─ sync_tokens (tokens de sesión de sync)                     │
│  └─ Tablas principales con updated_at para delta sync          │
└─────────────────────────────────────────────────────────────────┘
```

**Nota (2026):** El Sync Engine vive en `apps/web/src/lib/sync/` (TypeScript/Svelte), no en `crates/sync/`. El crate `crates/sync/` (Rust) es para el **agente de sede** (ADR 0022), no para el frontend. El frontend usa `@sqlite.org/sqlite-wasm` vía WebAssembly, no un crate Rust compilado a Wasm.

---

## Sync Queue

### Formato de Operación

```typescript
interface SyncOperation {
  id: string;           // UUID v7 local (uuid@14.0.0)
  type: 'create' | 'update' | 'acknowledge' | 'resolve';
  entity: 'device' | 'sede' | 'alert' | 'intrusion';
  entity_id: string;
  payload: Record<string, unknown>;
  timestamp: number;    // epoch ms local
  retry_count: number;  // intentos de sync
  status: 'pending' | 'syncing' | 'failed' | 'resolved';
  conflict_resolution?: 'local_wins' | 'remote_wins' | 'merge';
}
```

**Nota:** Los tipos `acknowledge` y `resolve` son operaciones de monitoreo (alertas e intrusiones) que deben funcionar offline. Se elimina `'delete'` como tipo de operación offline — el proyecto usa **Soft Delete** (ADR 0006), por lo que las operaciones de eliminación se traducen a `update` con `deleted_at`. Los IDs usan **UUID v7** (uuid@14.0.0) para ordenación temporal natural y trazabilidad.

### Proceso de Sync

```
1. Detectar conectividad (navigator.onLine + ping a /health)
2. Si online:
   a. Obtener last_sync_timestamp del servidor
   b. Enviar batch de operaciones pendientes (POST /sync/push)
   c. Recibir cambios del servidor desde last_sync (GET /sync/pull)
   d. Aplicar cambios remotos a SQLite local
   e. Resolver conflictos según estrategia configurada
   f. Actualizar last_sync_timestamp
3. Si offline:
   a. Encolar operación en sync_queue
   b. Marcar UI como "modo offline"
   c. Continuar operación localmente
```

---

## Resolución de Conflictos

### Estrategia por defecto: Last-Write-Wins (LWW)

```
Si local_timestamp > remote_timestamp:
    local_wins
Si local_timestamp < remote_timestamp:
    remote_wins
Si local_timestamp == remote_timestamp:
    server_wins (tie-breaker)
```

### Estrategias configurables por entidad

| Entidad | Estrategia | Justificación |
|---------|-----------|---------------|
| devices | LWW | Último estado es el válido |
| sedes | LWW | Último estado es el válido |
| alerts | remote_wins | El servidor tiene la verdad sobre alertas (generadas por jobs) |
| metric_readings | append_only | Nunca se actualizan, solo se insertan |
| intrusions | remote_wins | Seguridad: el servidor decide sobre intrusiones detectadas |

---

## Tokens PASETO en Modo Offline

### Problema

Los tokens PASETO tienen expiración (15 minutos de access token). En modo offline, no se puede refrescar el token.

### Solución

```
1. Access token: 15 minutos (normal)
2. Refresh token: 7 días (almacenado en session_cache de SQLite local)
3. Offline grace period: 24 horas
   - Si el token expiró hace < 24h, permitir operaciones locales de lectura y sync queue
   - Marcar UI como "sesión expirada, sincronizar al reconectar"
   - NO permitir operaciones de admin (crear usuarios, cambiar roles)
4. Al reconectar:
   - Intentar refresh con refresh_token
   - Si falla, redirigir a login
   - Si éxito, reanudar sync queue automáticamente
```

**Nota:** El refresh token se almacena en **localStorage encriptado** (subtle crypto) para persistencia multi-tab, y también en **SQLite session_cache** para disponibilidad offline. La cookie httpOnly se usa para SSR (ADR 0017 v2.0).

---

## Esquema SQLite Local (Frontend)

```sql
-- Devices cache
CREATE TABLE devices_cache (
    id TEXT PRIMARY KEY,
    hostname TEXT NOT NULL,
    ip TEXT,
    mac TEXT,
    device_type TEXT,
    status TEXT,
    sede_id TEXT,
    last_seen_at INTEGER,
    updated_at INTEGER,
    is_dirty BOOLEAN DEFAULT FALSE
);

-- Sedes cache
CREATE TABLE sedes_cache (
    id TEXT PRIMARY KEY,
    nombre TEXT NOT NULL,
    ubicacion TEXT,
    secretaria TEXT,
    updated_at INTEGER,
    is_dirty BOOLEAN DEFAULT FALSE
);

-- Alerts cache (últimas 100)
CREATE TABLE alerts_cache (
    id TEXT PRIMARY KEY,
    alert_type TEXT,
    severity TEXT,
    device_id TEXT,
    message TEXT,
    status TEXT,
    created_at INTEGER,
    updated_at INTEGER
);

-- Sync queue
CREATE TABLE sync_queue (
    id TEXT PRIMARY KEY,
    op_type TEXT NOT NULL CHECK(op_type IN ('create', 'update', 'acknowledge', 'resolve')),
    entity TEXT NOT NULL CHECK(entity IN ('device', 'sede', 'alert', 'intrusion')),
    entity_id TEXT NOT NULL,
    payload TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    retry_count INTEGER DEFAULT 0,
    status TEXT DEFAULT 'pending' CHECK(status IN ('pending', 'syncing', 'failed', 'resolved')),
    error_message TEXT
);

-- Sync metadata
CREATE TABLE sync_meta (
    key TEXT PRIMARY KEY,
    value TEXT
);
INSERT INTO sync_meta VALUES ('last_sync', '0');
INSERT INTO sync_meta VALUES ('sync_status', 'idle');
```

**Nota:** Se agregan tablas `sedes_cache` y `alerts_cache`. Se restringen `op_type` y `entity` con `CHECK` constraints. Se elimina `'delete'` de `op_type` (Soft Delete obligatorio, ADR 0006).

---

## Endpoints de Sync (Backend)

```
POST /api/v1/sync/push
  Body: { operations: SyncOperation[] }
  Response: { 
    accepted: string[], 
    rejected: { id, reason }[], 
    conflicts: Conflict[] 
  }
  Auth: Requiere PASETO válido o grace period < 24h (solo lectura)

GET /api/v1/sync/pull
  Query: ?since=<timestamp>&entities=device,sede,alert,intrusion
  Response: { 
    changes: EntityChange[], 
    next_sync_token: string,
    server_timestamp: number
  }
  Auth: Requiere PASETO válido

POST /api/v1/sync/resolve
  Body: { operation_id, resolution: 'local_wins' | 'remote_wins' }
  Response: { resolved: boolean }
  Auth: Requiere PASETO válido
```

---

## Agentes y Sync (ADR 0022)

El agente de sede también usa SQLite local para buffering de métricas:

```text
Sede sin internet
   ↓
Agente almacena métricas en SQLite local (crates/sync/ en Rust)
   ↓
Reconexión
   ↓
Sync automático vía crates/sync (Rust) → API REST
   ↓
API recibe batch de métricas
   ↓
PostgreSQL (origen de verdad)
   ↓
Jobs Apalis procesan agregaciones
```

**Nota:** El agente (Rust) y el frontend (TypeScript) comparten la **misma estrategia de sync** pero implementaciones distintas:
- **Agente**: `crates/sync/` (Rust) + SQLite local + HTTP client reqwest
- **Frontend**: `apps/web/src/lib/sync/` (TypeScript) + `@sqlite.org/sqlite-wasm` + fetch

---

## Sync Engine Frontend (TypeScript)

```typescript
// apps/web/src/lib/sync/sync-engine.ts

class SyncEngine {
  private db: Database; // @sqlite.org/sqlite-wasm
  private isOnline = $state(navigator.onLine);
  private syncStatus = $state<'idle' | 'syncing' | 'error'>('idle');

  constructor() {
    // Detectar cambios de conectividad
    window.addEventListener('online', () => this.onOnline());
    window.addEventListener('offline', () => this.onOffline());

    // Verificar conectividad real (navigator.onLine puede mentir)
    setInterval(() => this.checkRealConnectivity(), 30000);
  }

  async enqueue(op: SyncOperation): Promise<void> {
    // Si online, intentar enviar inmediatamente
    if (this.isOnline && await this.checkRealConnectivity()) {
      try {
        await this.pushSingle(op);
        return;
      } catch {
        // Falló, encolar
      }
    }

    // Encolar en SQLite local
    await this.db.exec(`
      INSERT INTO sync_queue (id, op_type, entity, entity_id, payload, timestamp, status)
      VALUES (?, ?, ?, ?, ?, ?, 'pending')
    `, [op.id, op.type, op.entity, op.entity_id, JSON.stringify(op.payload), Date.now()]);

    this.syncStatus = 'idle';
  }

  async processQueue(): Promise<void> {
    if (!this.isOnline) return;

    this.syncStatus = 'syncing';

    const pending = await this.db.exec(
      `SELECT * FROM sync_queue WHERE status = 'pending' ORDER BY timestamp ASC LIMIT 50`
    );

    if (pending.length === 0) {
      this.syncStatus = 'idle';
      return;
    }

    try {
      const response = await fetch('/api/v1/sync/push', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ operations: pending })
      });

      const result = await response.json();

      // Marcar aceptados como resolved
      for (const id of result.accepted) {
        await this.db.exec(`UPDATE sync_queue SET status = 'resolved' WHERE id = ?`, [id]);
      }

      // Marcar rechazados como failed con razón
      for (const { id, reason } of result.rejected) {
        await this.db.exec(
          `UPDATE sync_queue SET status = 'failed', error_message = ? WHERE id = ?`,
          [reason, id]
        );
      }

      // Manejar conflictos
      for (const conflict of result.conflicts) {
        await this.resolveConflict(conflict);
      }

      // Pull cambios remotos
      await this.pullChanges();

      this.syncStatus = 'idle';
    } catch (error) {
      this.syncStatus = 'error';
      console.error('Sync failed:', error);
    }
  }

  private async resolveConflict(conflict: Conflict): Promise<void> {
    const strategy = this.getConflictStrategy(conflict.entity);

    switch (strategy) {
      case 'local_wins':
        await this.retryWithForce(conflict.operation_id);
        break;
      case 'remote_wins':
        await this.db.exec(`UPDATE sync_queue SET status = 'resolved' WHERE id = ?`, [conflict.operation_id]);
        await this.applyRemoteChange(conflict.remote_version);
        break;
      case 'append_only':
        // Métricas: siempre aceptar ambas
        await this.db.exec(`UPDATE sync_queue SET status = 'resolved' WHERE id = ?`, [conflict.operation_id]);
        break;
    }
  }

  private async pullChanges(): Promise<void> {
    const lastSync = await this.getLastSyncTimestamp();

    const response = await fetch(`/api/v1/sync/pull?since=${lastSync}&entities=device,sede,alert,intrusion`);
    const { changes, server_timestamp } = await response.json();

    for (const change of changes) {
      await this.applyRemoteChange(change);
    }

    await this.setLastSyncTimestamp(server_timestamp);
  }
}
```

---

## Service Worker y Cache (PWA)

```typescript
// src/service-worker.ts

const CACHE = 'redes-beni-v1';
const STATIC_ASSETS = [
  '/',
  '/dashboard',
  '/manifest.json',
  '/icon-192.png',
  '/icon-512.png'
];

// Precache assets estáticos
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.addAll(STATIC_ASSETS))
  );
  self.skipWaiting();
});

// Network-first para API calls (con fallback a cache si offline)
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);

  // API calls: network-first, fallback a cache
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(
      fetch(event.request)
        .then((response) => {
          // Cachear respuesta exitosa
          if (response.ok) {
            const clone = response.clone();
            caches.open(CACHE).then((cache) => cache.put(event.request, clone));
          }
          return response;
        })
        .catch(() => caches.match(event.request))
    );
    return;
  }

  // Static assets: cache-first
  event.respondWith(
    caches.match(event.request).then((response) => {
      return response || fetch(event.request);
    })
  );
});
```

**Nota:** El Service Worker implementa **network-first para API** (con fallback a cache) y **cache-first para assets estáticos**. Esto garantiza que los datos se actualicen cuando hay conexión, pero la app sigue funcionando offline. NO se cachean datos sensibles (auth tokens, audit logs).

---

## Configuración Vite (OPFS + COOP/COEP)

Para usar `@sqlite.org/sqlite-wasm` con OPFS y SharedArrayBuffer:

```typescript
// vite.config.ts
import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  optimizeDeps: {
    exclude: ['@sqlite.org/sqlite-wasm'],
  },
});
```

**Nota:** OPFS y SharedArrayBuffer requieren headers COOP/COEP. Esto es obligatorio para la concurrencia entre tabs y el rendimiento de SQLite Wasm oficial. Si el despliegue no puede configurar estos headers, usar `sql.js` 1.14.1 como fallback (sin OPFS, persistencia en IndexedDB o memory).

---

## Dependencias npm

```bash
# Opción principal: SQLite oficial con OPFS
pnpm add @sqlite.org/sqlite-wasm@3.53.0-build1

# Alternativa: sql.js simple (fallback)
pnpm add sql.js@1.14.1

# UUID v7 para IDs de sync (monotónicos, sortables)
pnpm add uuid@14.0.0
```

---

## Consecuencias

### ✅ Positivas

* El sistema funciona durante cortes de internet
* Los usuarios pueden consultar inventario y topología offline
* Las operaciones críticas (acknowledge alert, resolve intrusion) no se pierden
* Re-sincronización automática transparente al reconectar
* UX consistente online/offline
* El agente de sede puede operar autónomamente y sincronizar métricas diferidas
* Datos críticos siempre disponibles en sedes remotas
* UUID v7 garantiza ordenación temporal de operaciones sin dependencias externas

### ⚠️ Trade-offs

* Complejidad adicional en frontend (SQLite Wasm + sync engine)
* Tamaño del bundle aumenta (~500KB por `@sqlite.org/sqlite-wasm`, ~300KB por `sql.js`)
* Conflictos de sync requieren atención del usuario (UI de resolución)
* Datos locales pueden quedar desactualizados hasta el próximo sync
* Necesita estrategia de purge de cache (retención 30 días recomendada)
* Grace period de 24h para tokens introduce riesgo de seguridad menor (mitigado: solo lectura + sync queue, no admin)
* OPFS requiere headers COOP/COEP — puede no ser compatible con todos los hosting/CDN

---

## Decisiones derivadas

* `@sqlite.org/sqlite-wasm` 3.53.0-build1 es la tecnología elegida para Local-First en frontend (OPFS, ES Module, oficial)
* `sql.js` 1.14.1 es **fallback oficial** si OPFS no está disponible o headers COOP/COEP no pueden configurarse
* La sync queue usa **UUID v7** (uuid@14.0.0) para ordenación temporal y trazabilidad
* La resolución de conflictos es **configurable por entidad** (LWW por defecto, remote_wins para alertas/intrusiones, append_only para métricas)
* Los tokens tienen **grace period de 24h** para operaciones offline (solo lectura + sync queue)
* El agente comparte la **misma estrategia de sync** que el frontend pero con implementación Rust (`crates/sync/`)
* **No se usa IndexedDB** como store principal — menos potente que SQLite para queries complejas (JOINs, aggregations)
* IndexedDB se usa solo como **fallback** si WebAssembly no está disponible
* El Service Worker cachea assets estáticos (cache-first) y API calls (network-first con fallback)
* **Soft Delete** se aplica en sync: las operaciones de "eliminación" se traducen a `update` con `deleted_at` (ADR 0006)
* La sync queue tiene **límite de 50 operaciones por batch** para evitar timeouts
* **Retry con backoff exponencial**: 1s, 2s, 4s, 8s, 16s (máximo 5 intentos)
* Los datos cacheados tienen **TTL de 30 días** — purge automático en reconexión
* `navigator.onLine` no es suficiente — se verifica con **ping real a `/health`** cada 30s
* El sync engine vive en `apps/web/src/lib/sync/` (TypeScript), no en `crates/sync/` (Rust)
* Vite debe configurar headers COOP/COEP para OPFS/SharedArrayBuffer

---

## Historial de cambios

| Versión | Fecha       | Cambios realizados |
| ------- | ----------- | ------------------ |
| 1.0     | 2026 (orig) | Versión inicial con ambigüedad sobre ubicación del sync engine (`crates/sync/` vs frontend), sin tablas `sedes_cache`/`alerts_cache`, sin restricciones CHECK en sync_queue, sin mención de Soft Delete en operaciones offline, sin límite de batch, sin estrategia de purge |
| 2.0     | 2026-05-16  | Clarifica que `crates/sync/` es para agente Rust y `apps/web/src/lib/sync/` para frontend; agrega tablas `sedes_cache` y `alerts_cache`; restringe `op_type` con CHECK (elimina 'delete', usa 'update' para Soft Delete); agrega límite de 50 ops/batch, retry backoff, TTL 30 días, ping real a /health; agrega implementación TypeScript del SyncEngine; agrega Service Worker con estrategia network-first/cache-first; documenta grace period de seguridad |
| 2.1     | 2026-05-16  | Especifica tecnología SQLite Wasm: `@sqlite.org/sqlite-wasm` 3.53.0-build1 (opción principal) y `sql.js` 1.14.1 (fallback); agrega `uuid` 14.0.0 para UUID v7; agrega configuración Vite para COOP/COEP; actualiza dependencias npm con versiones exactas; documenta requisitos de OPFS y SharedArrayBuffer |
