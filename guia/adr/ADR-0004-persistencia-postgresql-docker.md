# ADR 0004 — Persistencia con SQLite + Litestream

| Campo           | Valor                                                                                    |
| --------------- | ---------------------------------------------------------------------------------------- |
| **Estado**      | ✅ Aceptado                                                                               |
| **Fecha**       | 2026                                                                                     |
| **Autores**     | Milton Hipamo / Laboratorio 3030                                                        |
| **Revisado en** | ADR 0001 (Arquitectura Hexagonal), ADR 0003 (Stack Backend), ADR 0035 (Monitoreo Regional) |

> **Nota:** Este ADR fue actualizado de PostgreSQL para el proyecto de Monitoreo de Infraestructura Regional (ADR 0035) debido al bajo consumo de recursos y simplicidad operativa en VPS pequeños.

---

## Contexto

El sistema de monitoreo de infraestructura de red de la Gobernación del Beni
requiere una solución de persistencia que sea:

* estable,
* mantenible,
* predecible,
* fácil de desplegar,
* compatible con Docker,
* y adecuada para crecimiento progresivo.

La persistencia debe soportar:

* concurrencia real,
* consultas complejas,
* almacenamiento de métricas,
* relaciones estructuradas,
* agregaciones,
* auditoría,
* y futuras capacidades analíticas.

---

## Decisión

Usar PostgreSQL 16 ejecutándose en contenedor Docker
como motor principal de persistencia.

La integración con Rust se realizará mediante:

* SQLx
* migraciones versionadas
* consultas tipadas
* pool de conexiones async

---

## Stack aprobado

```yaml id="vbxuoz"
services:
  postgres:
    image: postgres:16-alpine

    container_name: redes_postgres

    restart: unless-stopped

    environment:
      POSTGRES_DB: redes
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres

    ports:
      - "5432:5432"

    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

---

## Configuración regional

| Configuración | Valor          |
| ------------- | -------------- |
| Encoding      | UTF-8          |
| Timezone      | America/La_Paz |
| Locale        | es_BO.UTF-8    |

---

## Integración con Rust

```toml id="vzjlwm"
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",
    "postgres",
    "macros",
    "uuid",
    "chrono",
] }
```

---

## Principios adoptados

| Principio               | Descripción                       |
| ----------------------- | --------------------------------- |
| Persistencia simple     | Una sola base principal           |
| Compatibilidad Docker   | Deploy consistente                |
| Seguridad de tipos      | SQL validado en compilación       |
| Migraciones versionadas | Cambios reproducibles             |
| Async nativo            | Integración eficiente con Tokio   |
| Operación predecible    | Fácil diagnóstico y mantenimiento |

---

## Estrategia de migraciones

Las migraciones se gestionan mediante:

```bash id="nrqgfm"
sqlx migrate add nombre_migracion
sqlx migrate run
```

Todas las modificaciones del esquema:

* deben ser versionadas,
* auditables,
* reproducibles,
* y compatibles con rollback manual.

---

## Pool de conexiones

Configuración inicial recomendada:

```txt id="hdunpj"
max_connections = 10
min_connections = 2
acquire_timeout = 30s
idle_timeout = 10m
```

La configuración podrá ajustarse
según carga real del sistema.

---

## Simplificaciones aplicadas

Para mantener simplicidad operacional
NO se incluyen en el MVP:

* sharding,
* replicación distribuida,
* multi-master,
* proxies SQL,
* event sourcing,
* CQRS,
* múltiples motores de base de datos,
* abstracciones ORM complejas.

El sistema utiliza:

* PostgreSQL,
* SQLx,
* Docker,
* migraciones simples.

---

## Alternativas descartadas

| Opción     | Motivo                                          |
| ---------- | ----------------------------------------------- |
| SQLite     | Limitaciones de concurrencia                    |
| MySQL      | Menor ergonomía con SQLx y funciones analíticas |
| MongoDB    | Complejidad innecesaria para el dominio         |
| Prisma ORM | Abstracción excesiva                            |
| Diesel     | Mayor fricción async                            |

---

## Herramientas aprobadas

| Herramienta      | Propósito             |
| ---------------- | --------------------- |
| `sqlx-cli`       | Migraciones           |
| `pgAdmin`        | Administración visual |
| `pg_dump`        | Backups               |
| `docker compose` | Orquestación simple   |
| `cargo-nextest`  | Testing               |

---

## Estrategia de backups

Backups iniciales:

* `pg_dump`
* snapshots del volumen Docker
* backups programados vía cron

La automatización avanzada se implementará
según necesidades operacionales futuras.

---

## Consecuencias

### ✅ Positivas

* Persistencia robusta
* Excelente soporte SQL
* Alta estabilidad
* Fácil integración con Rust
* Buen soporte para métricas y reportes
* Fácil despliegue con Docker
* Comunidad madura
* Excelente capacidad analítica

---

### ⚠️ Trade-offs

* Requiere gestión de backups
* Necesita monitoreo básico de almacenamiento
* Mayor complejidad que SQLite

---

## Impacto regional

PostgreSQL permite:

* almacenar métricas históricas,
* generar reportes de consumo,
* analizar comportamiento de red,
* mantener auditoría operativa,
* y escalar progresivamente sin reescritura.

Docker simplifica:

* despliegues,
* reinstalaciones,
* backups,
* recuperación operativa,
* y soporte remoto.

---

## Resultado esperado

Una capa de persistencia:

* estable,
* mantenible,
* segura,
* predecible,
* compatible con Rust async,
* fácil de operar,
* y preparada para crecimiento progresivo.
