# ADR 0015 — Jobs Asíncronos con Apalis

| Campo               | Valor                                                                 |
| ------------------- | --------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                           |
| **Fecha**           | 2026                                                                  |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                     |
| **Relacionado con** | ADR 0014 (Monitoreo), ADR 0003 (Stack Backend), ADR 0020 (Monitoreo Regional) |

---

# Contexto

El sistema necesita procesar tareas largas de forma asíncrona:

* Envío de emails transaccionales
* Procesamiento de alertas SNMP
* Jobs de mantenimiento programados
* Procesamiento de datos de red

El handler HTTP no debe bloquearse esperando estos procesos.

---

# Decisión

Usar **Apalis** como layer de jobs asíncronos en Rust.

---

# Arquitectura

```text
Request HTTP
     ↓
Use Case (sync)
     ↓
Enqueue Job → Apalis SQLite/Postgres
     ↓
Worker (async)
     ↓
Procesamiento
```

---

# Tipos de Jobs

## Jobs del Sistema

| Job              | Prioridad | Retry | Descripción                    |
| ----------------- | --------- | ----- | ------------------------------ |
| EmailJob          | Alta      | 3     | Envío de emails transaccionales|
| AlertJob          | Crítica   | 5     | Procesamiento de alertas      |
| CleanupJob        | Baja      | 1     | Mantenimiento y limpieza       |
| SyncJob           | Media     | 2     | Sincronización de datos        |

## Jobs de Monitoreo (ADR 0020)

| Job                    | Prioridad | Retry | Descripción                              |
| ---------------------- | --------- | ----- | ---------------------------------------- |
| MetricsAggregationJob  | Alta      | 3     | Agregación de métricas por hora/día     |
| AlertDispatchJob       | Crítica   | 5     | Envío de alertas por email/Telegram     |
| IntrusionDetectionJob  | Crítica   | 5     | Análisis de anomalías y detección        |
| CleanupMetricsJob      | Baja      | 1     | Limpieza de métricas antiguas (30 días) |
| TopologyUpdateJob      | Media     | 2     | Actualización del mapa de red            |

---

# Configuración

```rust
// apps/api/src/jobs/mod.rs

use apalis::prelude::*;

pub fn register_jobs(registry: &mut JobRegistry) {
    // Jobs del sistema
    registry.register::<EmailJob>("email");
    registry.register::<AlertJob>("alert");
    registry.register::<CleanupJob>("cleanup");
    
    // Jobs de Monitoreo (ADR 0020)
    registry.register::<MetricsAggregationJob>("metrics_aggregation");
    registry.register::<AlertDispatchJob>("alert_dispatch");
    registry.register::<IntrusionDetectionJob>("intrusion_detection");
    registry.register::<CleanupMetricsJob>("cleanup_metrics");
    registry.register::<TopologyUpdateJob>("topology_update");
}
```

---

# Monitoreo

Relacionado con ADR 0014 y ADR 0020.

Cada worker debe enviar heartbeat a Healthchecks.io.

Los jobs de monitoreo (MetricsAggregationJob, IntrusionDetectionJob, AlertDispatchJob) son esenciales para el módulo de Monitoreo de Infraestructura Regional definido en ADR 0020.

---

# Consecuencias

## ✅ Positivas

* Procesamiento asíncrono sin bloquear HTTP
* Retry automático configurable
* Priorities integradas
* Compatible con SQLite en desarrollo

---

## ⚠️ Negativas / Trade-offs

* Mayor complejidad que procesamiento síncrono
* Necesita worker separado en producción
* Requiere monitoreo (Healthchecks.io)

---

# Decisiones derivadas

* El worker de Apalis tiene monitoreo obligatorio
* Jobs críticos tienen retry > 3