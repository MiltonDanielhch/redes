# ADR 0004 — Persistencia con PostgreSQL + Docker

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026-05-16 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0002 (Configuración), ADR 0003 (Stack Backend), ADR 0005 (Migraciones), ADR 0013 (Docker Compose), ADR 0020 (Monitoreo Regional), ADR 0021 (Local-First) |

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

* concurrencia real (múltiples agentes + API + jobs),
* consultas complejas (joins, agregaciones, window functions),
* almacenamiento de métricas time-series,
* relaciones estructuradas (sedes, dispositivos, links),
* agregaciones históricas (rollup por hora/día),
* auditoría permanente,
* y futuras capacidades analíticas.

---

## Decisión

Usar **PostgreSQL 16** ejecutándose en contenedor Docker
como motor principal de persistencia del backend.

La integración con Rust se realizará mediante:

* SQLx 0.8.5 (queries compile-time checked)
* Migraciones versionadas (SQLx migrations)
* Consultas tipadas (`#[sqlx::query_as]`)
* Pool de conexiones async (PgPoolOptions)

**Nota sobre SQLite:** SQLite se usa **únicamente** en el frontend/browser para operación offline (ADR 0021 — Local-First). Nunca como motor principal del servidor. El backend solo habla PostgreSQL.

---

## Stack aprobado

### Docker Compose (servicio PostgreSQL)

```yaml
services:
  postgres:
    image: postgres:16.4-alpine
    container_name: redes_postgres
    restart: unless-stopped

    environment:
      POSTGRES_DB: redes
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-postgres}
      POSTGRES_INITDB_ARGS: "--encoding=UTF8 --locale=es_BO.UTF-8"
      TZ: America/La_Paz
      PGTZ: America/La_Paz

    ports:
      - "5432:5432"

    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./infra/docker/postgresql.conf:/etc/postgresql/postgresql.conf:ro
      - ./data/backups:/backups

    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres -d redes"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 30s

    command:
      - "postgres"
      - "-c"
      - "config_file=/etc/postgresql/postgresql.conf"

    networks:
      - redes_network

  # API depende de PostgreSQL estar saludable
  api:
    build: ./apps/api
    depends_on:
      postgres:
        condition: service_healthy
    # ... (ver ADR 0013 para compose completo)

volumes:
  postgres_data:
    driver: local

networks:
  redes_network:
    driver: bridge
```

---

### Configuración PostgreSQL optimizada para VPS

```conf
# infra/docker/postgresql.conf
# Optimizado para VPS con 1-2 vCPU, 2-4GB RAM

# Memoria
shared_buffers = 256MB
effective_cache_size = 768MB
work_mem = 4MB
maintenance_work_mem = 64MB

# WAL (Write-Ahead Logging) — para backups incrementales
wal_level = replica
max_wal_size = 1GB
min_wal_size = 80MB
wal_buffers = 16MB
checkpoint_completion_target = 0.9

# Conexiones
max_connections = 50

# Logging
log_destination = 'stderr'
logging_collector = on
log_directory = '/var/log/postgresql'
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
log_rotation_age = 1d
log_min_duration_statement = 1000  # Log queries > 1s
log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '
log_checkpoints = on
log_connections = on
log_disconnections = on
log_lock_waits = on

# Timezone
timezone = 'America/La_Paz'
lc_messages = 'es_BO.UTF-8'
lc_monetary = 'es_BO.UTF-8'
lc_numeric = 'es_BO.UTF-8'
lc_time = 'es_BO.UTF-8'

# Performance
random_page_cost = 1.1  # SSD
effective_io_concurrency = 200
```

---

### Configuración regional

| Configuración | Valor | Notas |
| -------------- | ------- | ----- |
| Encoding | UTF-8 | Soporte completo Unicode |
| Timezone | America/La_Paz | UTC-4, sin DST |
| Locale | es_BO.UTF-8 | Español Bolivia |
| Collation | es_BO.UTF-8 | Ordenamiento correcto de acentos |

---

## Integración con Rust

```toml
[dependencies]
sqlx = { version = "0.8.5", features = [
    "runtime-tokio-rustls",
    "postgres",
    "macros",
    "migrate",
    "chrono",
    "uuid",
    "time",
] }
```

**Features requeridas:**
- `runtime-tokio-rustls` — Async runtime + TLS
- `postgres` — Driver PostgreSQL
- `macros` — `#[sqlx::query_as]`, `sqlx::migrate!`
- `migrate` — Migraciones embebidas en binario
- `chrono` — Tipos de fecha/hora
- `uuid` — Tipos UUID nativos
- `time` — Tipos `time` crate

---

## Pool de conexiones (PgPoolOptions)

```rust
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
        .connect(database_url)
        .await
}
```

| Parámetro | Valor | Justificación |
|-----------|-------|---------------|
| max_connections | 20 | Soporta API + jobs + agentes concurrentes |
| min_connections | 5 | Evitar latencia de conexión en carga baja |
| acquire_timeout | 5s | Fail-fast si DB saturada |
| idle_timeout | 300s | Liberar conexiones inactivas |
| max_lifetime | 1800s | Rotar conexiones periódicamente |
| test_before_acquire | true | Verificar conexión antes de usar |

---

## Estrategia de migraciones

### Workflow de desarrollo

```bash
# Crear nueva migración
sqlx migrate add create_devices_table

# Escribir SQL en el archivo generado (up.sql y down.sql)
# Ejecutar migraciones
sqlx migrate run

# Verificar estado
sqlx migrate info

# Preparar queries para CI (compile-time checking)
cargo sqlx prepare --workspace

# Revertir última migración (cuidado en prod)
sqlx migrate revert
```

### Reglas de migraciones

| Regla | Descripción |
|-------|-------------|
| R1 | Toda migración tiene `up.sql` y `down.sql` |
| R2 | Las migraciones son idempotentes donde sea posible |
| R3 | `down.sql` solo para desarrollo, nunca en producción |
| R4 | Migraciones de datos (seeds) separadas de esquema |
| R5 | `sqlx prepare` ejecutado en CI antes de build |
| R6 | Migraciones aplicadas manualmente en producción (no automático) |

---

## Estrategia de backups

### Nivel 1: pg_dump (full backup)

```bash
# Backup diario completo
pg_dump -h localhost -U postgres -d redes -F c -f /backups/redes_$(date +%Y%m%d_%H%M%S).dump

# Restauración
pg_restore -h localhost -U postgres -d redes --clean /backups/redes_YYYYMMDD_HHMMSS.dump
```

### Nivel 2: WAL Archiving (point-in-time recovery)

```bash
# Configurar pgbackrest (herramienta recomendada)
# infra/docker/pgbackrest.conf

[global]
repo1-path=/backups
repo1-retention-full=7
repo1-retention-diff=4

[redes]
pg1-path=/var/lib/postgresql/data

# Backup incremental
pgbackrest --stanza=redes backup --type=incr

# Backup full semanal
pgbackrest --stanza=redes backup --type=full

# Point-in-time recovery
pgbackrest --stanza=redes restore --type=time --target="2026-05-16 14:30:00"
```

### Nivel 3: Snapshots de volumen Docker

```bash
# Snapshot del volumen (para recuperación rápida)
docker run --rm -v redes_postgres_data:/data -v $(pwd)/snapshots:/backup alpine     tar czf /backup/postgres_$(date +%Y%m%d).tar.gz -C /data .
```

### Política de retención

| Tipo | Frecuencia | Retención |
|------|-----------|-----------|
| pg_dump full | Diario | 7 días |
| pgbackrest incremental | Cada 6h | 4 días |
| pgbackrest full | Semanal | 4 semanas |
| Docker snapshot | Semanal | 2 semanas |

---

## Principios adoptados

| Principio | Descripción |
|-----------|-------------|
| Persistencia simple | Una sola base principal (PostgreSQL) |
| Compatibilidad Docker | Deploy consistente entre entornos |
| Seguridad de tipos | SQL validado en compilación (SQLx) |
| Migraciones versionadas | Cambios reproducibles y auditables |
| Async nativo | Integración eficiente con Tokio |
| Operación predecible | Fácil diagnóstico y mantenimiento |
| WAL archiving | Point-in-time recovery para desastres |
| Healthcheck | Dependencias esperan a que DB esté lista |

---

## Simplificaciones aplicadas

Para mantener simplicidad operacional
NO se incluyen en el MVP:

* Sharding (particionamiento manual si volumen > 10M métricas/mes)
* Replicación master-slave (evaluar si uptime crítico)
* Multi-master
* Proxies SQL (PgBouncer)
* Event sourcing
* CQRS
* Múltiples motores de base de datos en backend
* ORM complejos (Diesel, Prisma)

**Nota sobre SQLite:** SQLite se usa **únicamente** en el frontend/browser para operación offline (ADR 0021 — Local-First Sync Offline). No es un motor de base de datos del backend. El agente local usa SQLite embebido para buffer offline, pero eso es local al agente, no al servidor.

El sistema utiliza:

* PostgreSQL 16 (backend principal)
* SQLx 0.8.5 (driver async Rust)
* Docker Compose (orquestación)
* Migraciones SQLx (versionadas)
* pgbackrest (backups incrementales)

---

## Alternativas descartadas

| Opción | Motivo |
|--------|--------|
| SQLite | Limitaciones de concurrencia (writer lock), no es backend multi-usuario |
| MySQL | Menor ergonomía con SQLx, funciones analíticas menos maduras |
| MongoDB | Complejidad innecesaria para dominio relacional (sedes, dispositivos, links) |
| Prisma ORM | Abstracción excesiva, menos control sobre queries |
| Diesel | Mayor fricción async, compile-time más lento |
| TimescaleDB | Overkill para MVP (evaluar si métricas > 10M/mes) |

---

## Herramientas aprobadas

| Herramienta | Propósito | Versión |
|-------------|-----------|---------|
| `sqlx-cli` | CLI de migraciones y prepare | `0.8.5` |
| `pgAdmin` | Administración visual | `4` |
| `pg_dump` / `pg_restore` | Backups full | PostgreSQL 16 |
| `pgbackrest` | Backups incrementales + PITR | `2.53` |
| `docker compose` | Orquestación de servicios | `2.25+` |
| `cargo-nextest` | Tests rápidos | `0.9` |
| `pgbench` | Benchmark de PostgreSQL | PostgreSQL 16 |

---

## Monitoreo de PostgreSQL

```sql
-- Queries lentas (> 1s)
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
WHERE mean_exec_time > 1000
ORDER BY mean_exec_time DESC;

-- Espacio en disco por tabla
SELECT schemaname, tablename,
       pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Conexiones activas
SELECT count(*), state
FROM pg_stat_activity
GROUP BY state;
```

---

## Consecuencias

### ✅ Positivas

* Persistencia robusta y probada en producción
* Excelente soporte SQL (window functions, CTEs, JSONB)
* Alta estabilidad (ACID, MVCC)
* Fácil integración con Rust async (SQLx)
* Buen soporte para métricas time-series (con índices apropiados)
* Fácil despliegue con Docker (imagen oficial, documentada)
* Comunidad madura y extensa
* Capacidad analítica (reportes, agregaciones, rollup)
* Point-in-time recovery con WAL archiving
* Healthcheck nativo para orquestación

---

### ⚠️ Trade-offs

* Requiere gestión de backups (automatizar con pgbackrest)
* Necesita monitoreo de espacio en disco y queries lentas
* Mayor complejidad operativa que SQLite (pero necesaria para concurrencia)
* Configuración de performance requiere ajuste según VPS

---

## Impacto regional

PostgreSQL permite:

* almacenar métricas históricas de todas las sedes,
* generar reportes de consumo de ancho de banda,
* analizar comportamiento de red en el tiempo,
* mantener auditoría operativa permanente,
* y escalar progresivamente sin reescritura (particionamiento, índices).

Docker simplifica:

* despliegues consistentes (dev = staging = prod),
* reinstalaciones rápidas,
* backups de volumen,
* recuperación operativa,
* y soporte remoto (mismo entorno en todas partes).

---

## Resultado esperado

Una capa de persistencia:

* estable (ACID, transacciones),
* mantenible (migraciones versionadas, SQLx),
* segura (queries compile-time checked, TLS),
* predecible (pool configurado, healthcheck),
* compatible con Rust async (Tokio + SQLx),
* fácil de operar (Docker, pgAdmin),
* con backups confiables (pg_dump + pgbackrest),
* y preparada para crecimiento progresivo (particionamiento, índices, WAL).
