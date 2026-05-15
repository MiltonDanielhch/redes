# ADR 0003 — Stack Backend: Rust 2024 + Axum 0.8 + Tokio

| Campo           | Valor                                                           |
| --------------- | --------------------------------------------------------------- |
| **Estado**      | ✅ Aceptado                                                      |
| **Fecha**       | 2026                                                            |
| **Autores**     | Milton Hipamo / Laboratorio 3030                                |
| **Revisado en** | ADR 0001 (Arquitectura Hexagonal), ADR 0014 (Deploy Distroless), ADR 0035 (Monitoreo Regional) |

---

## Contexto

El sistema de monitoreo de infraestructura de red de la Gobernación del Beni
necesita un backend capaz de:

* manejar múltiples conexiones concurrentes,
* consumir pocos recursos,
* responder rápidamente,
* ejecutarse en VPS pequeños,
* y mantenerse estable durante largos periodos de operación.

El stack backend debe cumplir:

* alto rendimiento,
* seguridad de memoria,
* bajo consumo de RAM,
* concurrencia eficiente,
* binario único,
* despliegue simple,
* compatibilidad con Docker.

---

## Decisión

Usar:

* Rust Edition 2024
* Axum 0.8
* Tokio

como stack principal del backend.

---

## Motivos principales

### Rust

Rust permite:

* seguridad de memoria sin garbage collector,
* ausencia de data races,
* alta performance,
* bajo consumo de recursos,
* binarios pequeños y estáticos.

Esto es importante para infraestructura regional
con recursos limitados.

---

### Axum

Axum proporciona:

* integración nativa con Tokio,
* middleware basado en Tower,
* tipado fuerte,
* handlers simples,
* arquitectura modular,
* excelente compatibilidad con Rust moderno.

Encaja correctamente con la arquitectura hexagonal del ADR 0001.

---

### Tokio

Tokio proporciona:

* runtime asíncrono eficiente,
* concurrencia cooperativa,
* tareas ligeras,
* networking de alto rendimiento,
* ecosistema maduro.

---

## Stack aprobado

```toml
# apps/api/Cargo.toml

axum = { version = "0.8", features = ["macros"] }

tokio = { version = "1", features = [
    "rt-multi-thread",
    "macros",
    "signal",
] }

tower = "0.5"

tower-http = { version = "0.6", features = [
    "cors",
    "trace",
    "timeout",
    "compression-gzip",
] }

serde = { version = "1", features = ["derive"] }
serde_json = "1"

tracing = "0.1"
tracing-subscriber = "0.3"

sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",
    "postgres",
    "macros",
] }
```

---

## Middleware aprobado

```rust
let app = Router::new()
    .merge(api_router())
    .with_state(state)
    .layer(
        ServiceBuilder::new()

            // Request ID
            .layer(SetRequestIdLayer::x_request_id(
                MakeRequestUuid
            ))

            // Tracing
            .layer(TraceLayer::new_for_http())

            // Compresión
            .layer(CompressionLayer::new())

            // CORS
            .layer(cors_layer)

            // Timeout global
            .layer(
                TimeoutLayer::new(
                    Duration::from_secs(30)
                )
            )
    );
```

---

## Principios adoptados

| Principio             | Descripción                            |
| --------------------- | -------------------------------------- |
| Binario único         | Deploy simple                          |
| Runtime eficiente     | Menor consumo de RAM                   |
| Async real            | Miles de conexiones concurrentes       |
| Seguridad             | Memory safety sin GC                   |
| Middleware composable | Capas desacopladas                     |
| Compile-time safety   | Errores detectados antes de producción |

---

## Graceful shutdown

El backend debe cerrar correctamente:

* conexiones HTTP,
* tareas async,
* pool PostgreSQL,
* workers.

```rust
let shutdown = async {
    tokio::signal::ctrl_c()
        .await
        .expect("signal error");
};

axum::serve(listener, app)
    .with_graceful_shutdown(shutdown)
    .await?;
```

---

## Simplificaciones aplicadas

Para mantener simplicidad operacional
NO se incluyen en el MVP:

* Kubernetes
* gRPC interno
* Service Mesh
* Event sourcing
* CQRS
* Microservicios
* Runtime distribuido
* Kafka
* NATS

El sistema actual es:

* un backend Axum,
* un frontend SvelteKit,
* PostgreSQL,
* Docker.

---

## Herramientas aprobadas

| Herramienta     | Propósito          |
| --------------- | ------------------ |
| `tower-http`    | Middleware HTTP    |
| `tracing`       | Observabilidad     |
| `axum-test`     | Testing del router |
| `utoipa`        | OpenAPI            |
| `cargo-nextest` | Tests rápidos      |
| `tokio-console` | Debug async        |

---

## Alternativas descartadas

| Opción         | Motivo                           |
| -------------- | -------------------------------- |
| Node.js        | Mayor consumo de RAM             |
| FastAPI        | Menor performance                |
| Spring Boot    | JVM demasiado pesada             |
| Go + Fiber     | Menor seguridad de tipos         |
| Microservicios | Complejidad operacional excesiva |

---

## Consecuencias

### ✅ Positivas

* Bajo consumo de recursos
* Excelente performance
* Alta concurrencia
* Seguridad de memoria
* Binario pequeño
* Deploy simple
* Excelente integración con Docker
* Compatible con VPS económicos

---

### ⚠️ Trade-offs

* Curva de aprendizaje alta
* Compilaciones más lentas
* Ecosistema más joven que Java o Node

---

## Impacto regional

El stack backend permite desplegar
el sistema de monitoreo en infraestructura limitada.

Beneficios directos:

* menor costo operativo,
* menor consumo de RAM,
* menor consumo de CPU,
* menos reinicios,
* menos fallos por concurrencia,
* mejor estabilidad en producción.

Esto es importante para oficinas regionales
con soporte técnico limitado.

---

## Resultado esperado

Un backend:

* rápido,
* estable,
* seguro,
* eficiente,
* fácil de desplegar,
* barato de operar,
* y preparado para crecer sin reescritura.
