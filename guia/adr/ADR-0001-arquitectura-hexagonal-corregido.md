# ADR 0001 — Arquitectura Hexagonal y Monolito Modular

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Revisado en** | ADR 0003 (Stack Backend), ADR 0004 (Persistencia PostgreSQL), ADR 0018 (CLI Arquitectónico) |

---

## Contexto

El primer problema de todo proyecto es elegir cómo organizar el código antes de escribir
la primera línea. La tentación de los microservicios es real, pero para un equipo pequeño
con un VPS de bajo costo introduce complejidad operacional innecesaria.

El proyecto de monitoreo de infraestructura de red para la Gobernación del Beni necesita:

- Simplicidad operacional
- Bajo consumo de recursos
- Despliegue rápido
- Escalabilidad estructural
- Código mantenible por un solo desarrollador
- Compatibilidad con agentes IA
- Separación estricta entre negocio e infraestructura

Necesitábamos una arquitectura que permitiera evolucionar el sistema sin reescribir el núcleo.

---

## Decisión

Usar un **monolito modular** como unidad de despliegue con **arquitectura hexagonal**
como disciplina interna de organización del código.

> 2 contenedores bien estructurados vencen a 20 microservicios mal pensados.

---

## Las tres capas del modelo

```txt
┌────────────────────────────────────────────┐
│             Adaptadores (afuera)           │
│ HTTP · CLI · Workers · SNMP · Email · S3  │
│                                            │
│ ┌────────────────────────────────────────┐ │
│ │         Puertos (contratos)            │ │
│ │ EscanearRed · GuardarMétrica · Alertar │ │
│ │                                        │ │
│ │ ┌────────────────────────────────────┐ │ │
│ │ │       Dominio (el corazón)         │ │ │
│ │ │ Entidades · Reglas de negocio      │ │ │
│ │ │ No conoce HTTP ni SQL              │ │ │
│ │ │ No conoce frameworks               │ │ │
│ │ └────────────────────────────────────┘ │ │
│ └────────────────────────────────────────┘ │
└────────────────────────────────────────────┘
```

Las dependencias siempre apuntan hacia adentro.

El dominio no conoce:

- Axum
- PostgreSQL
- Docker
- HTTP
- SNMP
- SvelteKit

El dominio solo conoce reglas de negocio.

---

## Estructura del monorepo

```txt
redes/
├── apps/
│   ├── api/           # Backend Axum
│   └── web/           # Dashboard SvelteKit 5
│
└── crates/
    ├── domain/        # Núcleo puro
    ├── application/   # Casos de uso
    ├── infrastructure/# HTTP, configuración y adaptadores
    └── database/      # SQLx + PostgreSQL
```

---

## Capacidades futuras (No implementadas)

Estas capacidades son compatibles con la arquitectura,
pero NO forman parte del MVP actual.

```txt
Posibles extensiones futuras (solo si el problema existe):

- agents/ (Agentes de monitoreo en sedes remotas)
- sync/ (Sincronización offline)
- events/ (NATS JetStream — si jobs son muy pesados)
```

> **Nota del proyecto actual:** El ADR 0020 define el proyecto de Monitoreo de Infraestructura Regional que solo requiere web (SvelteKit), API (Axum) y agentes de monitoreo.

Solo se implementarán si existe necesidad operacional real.

---

## Ventaja principal

Si `crates/domain` no declara dependencias externas,
es imposible importar infraestructura accidentalmente.

El compilador hace cumplir la arquitectura.

No depende de:

- convenciones,
- disciplina manual,
- code reviews.

---

## Ejemplo: puerto y adaptador en Rust

```rust
// crates/domain/src/ports/device_repository.rs

pub trait DeviceRepository: Send + Sync {
    async fn find_by_ip(&self, ip: &IpAddress)
        -> Result<Option<Device>, DomainError>;

    async fn save_metric(&self, metric: &NetworkMetric)
        -> Result<(), DomainError>;
}
```

```rust
// crates/database/src/repositories/postgres_device_repository.rs

impl DeviceRepository for PostgresDeviceRepository {
    async fn find_by_ip(
        &self,
        ip: &IpAddress,
    ) -> Result<Option<Device>, DomainError> {

        sqlx::query_as!(
            DeviceRow,
            "SELECT * FROM devices WHERE ip = $1",
            ip.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .map(Device::try_from)
        .transpose()
    }
}
```

Si mañana PostgreSQL cambia,
solo se reemplaza el adaptador.

El dominio permanece intacto.

---

## Fases del desarrollo

### Diseño

| Patrón | Rol |
|--------|-----|
| Arquitectura Hexagonal | Separación entre negocio e infraestructura |
| DDD táctico | Entidades y Value Objects |
| Puertos | Contratos del dominio |
| Onion Architecture | Dependencias hacia el centro |

---

### Construcción

| Elemento | Rol |
|----------|-----|
| Adaptadores | Implementaciones reales |
| Traits | Contratos tipados |
| Tipos fuertes | Prevención de errores |
| Rust | Seguridad y performance |

---

### Evolución

| Cambio | Impacto |
|--------|----------|
| Cambiar PostgreSQL | Solo cambia `database/` |
| Cambiar Axum | Solo cambia `infrastructure/` |
| Cambiar SNMP | Solo cambia adaptador SNMP |
| Cambiar reglas de negocio | Solo cambia `domain/` |

---

## Herramientas aprobadas

| Herramienta | Propósito |
|-------------|-----------|
| `cargo-boundary` | Validar fronteras arquitectónicas |
| `cargo-nextest` | Tests rápidos |
| `taplo` | Orden del workspace |
| `insta` | Snapshot testing |
| `cargo-expand` | Debug de macros |

---

## Mandamientos del proyecto

| Mandamiento | Descripción |
|-------------|-------------|
| Protección del dominio | El negocio nunca depende de infraestructura |
| Simplicidad operacional | Menos servicios, menos problemas |
| Independencia tecnológica | Frameworks reemplazables |
| Código entendible | Diseñado para humanos e IA |
| Escalabilidad estructural | El sistema puede crecer sin reescritura |

---

## Alternativas descartadas

| Opción | Motivo |
|--------|---------|
| Microservicios | Complejidad operacional excesiva |
| Monolito sin capas | Difícil mantenimiento |
| Kubernetes temprano | Innecesario para el tamaño actual |
| Event sourcing | Complejidad sin beneficio inmediato |

---

## Consecuencias

### ✅ Positivas

- Deploy simple
- Bajo consumo de RAM
- Fácil mantenimiento
- Escalable estructuralmente
- Código testeable
- Compatible con agentes IA
- Separación clara de responsabilidades

---

### ⚠️ Trade-offs

- Más archivos iniciales
- Curva de aprendizaje en hexagonal
- Requiere disciplina arquitectónica

---

## Impacto regional

La arquitectura permite adaptar el sistema a distintos equipos de red
sin modificar el núcleo del negocio.

Si la Gobernación cambia:

- switches,
- proveedores,
- protocolos,
- infraestructura,

solo cambian adaptadores externos.

La lógica principal permanece estable.

---

## Resultado esperado

Una base sólida para un sistema de monitoreo regional:

- mantenible,
- simple de operar,
- barata de desplegar,
- fácil de evolucionar,
- resistente a cambios tecnológicos.
