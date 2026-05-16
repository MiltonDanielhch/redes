# ADR 0011 — Estándares de Desarrollo: Ciclo Lab → Puente → Producción

| Campo               | Valor                                                                                                                    |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                                               |
| **Fecha**           | 2026-05-15                                                                                                               |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                         |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0010 (Testing), ADR 0012 (just + lefthook), ADR 0013 (Deploy) |

---

# Contexto

El desarrollo de software tiende a degradarse con el tiempo por:

* deuda técnica acumulada
* builds lentos
* módulos gigantes difíciles de mantener
* procesos manuales
* falta de validaciones automáticas
* arquitectura inconsistente entre features

En un entorno de:

* infraestructura limitada
* un solo servidor físico potente
* equipo pequeño (1 persona + IA)
* foco en velocidad de entrega

…la complejidad operacional innecesaria debe eliminarse agresivamente.

El objetivo es que:

* el código siga siendo entendible en 2 años
* una IA pueda navegar el proyecto sin contexto humano constante
* el feedback loop sea extremadamente rápido
* deployar sea rutinario y no un evento de estrés

---

# Decisión

Se adopta oficialmente el modelo:

```text
Laboratorio → Puente → Producción
```

como estándar obligatorio de desarrollo del proyecto.

La prioridad arquitectónica es:

1. simplicidad
2. mantenibilidad
3. velocidad de iteración
4. observabilidad
5. automatización

antes que “enterprise patterns” innecesarios.

---

# 1 — Restricciones de atomicidad

## Límites estructurales

| Unidad       | Límite recomendado    | Acción al superarse               |
| ------------ | --------------------- | --------------------------------- |
| Función      | ~30 líneas reales     | Extraer funciones                 |
| Archivo      | ~200 líneas           | Dividir módulo                    |
| Trait        | Responsabilidad única | Segregar interfaces               |
| Crate        | Dependencias mínimas  | Separar responsabilidades         |
| Handler HTTP | Solo orquestación     | Mover lógica al application layer |

---

## Filosofía

### Código pequeño → contexto pequeño → IA más efectiva

El tamaño controlado:

* mejora refactors
* mejora testing
* reduce bugs ocultos
* reduce costo cognitivo
* mejora generación de código con IA

---

## Regla del Boy Scout

> “Siempre deja el código un poco más limpio de como lo encontraste.”

Cada commit debe:

* simplificar
* eliminar duplicación
* mejorar naming
* reducir complejidad

aunque el cambio principal sea otro.

---

# 2 — Ciclo oficial de desarrollo

---

## LABORATORIO (desarrollo local)

Entorno de:

* experimentación
* TDD
* prototipado rápido
* refactors agresivos

### Reglas

* Nunca optimizar prematuramente
* Preferir claridad antes que abstracción
* Crear tests antes de lógica compleja
* Feedback loop < 5 segundos

---

## Flujo obligatorio

```bash
cargo check --workspace
cargo fmt
cargo clippy
cargo nextest run
```

antes de commit.

---

## Herramientas recomendadas

| Herramienta   | Uso                      |
| ------------- | ------------------------ |
| `bacon`       | feedback en tiempo real  |
| `cargo-watch` | recompilación automática |
| `just`        | comandos reproducibles   |
| `nextest`     | tests paralelos          |
| `clippy`      | linting estricto         |
| `typos`       | corrección ortográfica   |
| `cargo-deny`  | auditoría dependencias   |

---

## PUENTE (CI / staging)

Entorno de validación automática.

Aquí se verifica:

* arquitectura
* contratos
* seguridad
* compatibilidad
* reproducibilidad

---

## Pipeline mínimo

```bash
cargo nextest run --all-targets
cargo clippy --all-targets -- -D warnings
cargo deny check
cargo audit
just prepare
just types-check
```

---

## Reglas del CI

El CI debe fallar si:

* existe un warning
* `.sqlx/` está desactualizado
* hay dependencias vulnerables
* existe drift en tipos TS
* falla cualquier test
* se rompe la arquitectura

---

## PRODUCCIÓN

Producción NO es entorno de compilación.

El servidor:

* solo ejecuta binarios ya compilados
* no compila Rust
* no instala crates
* no genera assets

---

## Principios

### Fail-fast

Si algo crítico falla:

* migraciones
* configuración
* secretos
* healthcheck

…el proceso debe morir inmediatamente.

---

### Rollback inmediato

Un deploy fallido debe revertirse automáticamente en segundos.

---

### Observabilidad obligatoria

Toda request debe tener:

* `request_id`
* tracing
* contexto de usuario
* logs estructurados

---

# 3 — Código autodocumentado

## Regla principal

Los comentarios deben explicar:

* el “por qué”
* la decisión de negocio
* la razón arquitectónica

NO el “cómo”.

---

## Ejemplo correcto

```rust
// Soft Delete preserva audit_logs históricos (ADR 0006)
user.soft_delete();
```

---

## Ejemplo incorrecto

```rust
// Iterar usuarios y filtrar activos
users.iter().filter(...)
```

---

# 4 — Tipos fuertes sobre primitivas

## Incorrecto

```rust
fn create_user(id: String, role: String)
```

---

## Correcto

```rust
fn create_user(id: UserId, role: UserRole)
```

---

# 5 — Filosofía de dependencias

## Regla

Cada dependencia debe justificar:

* costo de compilación
* complejidad
* mantenimiento
* superficie de ataque
* impacto cognitivo

---

## Prohibido

Agregar librerías para problemas triviales.

---

## Preferir stdlib cuando:

* la solución es simple
* el código propio es pequeño
* la dependencia agrega demasiada abstracción

---

# 6 — Convenciones para IA

El proyecto está diseñado explícitamente para colaboración humano + IA.

---

## Reglas

### Nombres explícitos

```rust
create_user_use_case.rs
sqlite_user_repository.rs
```

NO:

```rust
service.rs
helpers.rs
utils.rs
```

---

### Estructura predecible

Cada módulo debe tener:

```text
domain/
application/
infrastructure/
tests/
```

cuando aplique.

---

### Un concepto por archivo

Evita archivos “multi-propósito”.

---

# 7 — Política de complejidad operacional

Con un servidor físico potente propio:

* se elimina dependencia de VPS externos baratos
* se evita infraestructura distribuida innecesaria
* se prioriza operación simple y local-first

---

## Consecuencia arquitectónica

Quedan despriorizados:

* multi-node premature scaling
* Kubernetes
* Redis “porque sí”
* microservicios
* service mesh
* colas distribuidas innecesarias

---

## Principio

> “La complejidad operacional es deuda técnica.”

---

# 8 — Comparativa SDLC

| Métrica         | Tradicional       | Laboratorio 3030   |
| --------------- | ----------------- | ------------------ |
| Feedback loop   | Horas / días      | Segundos / minutos |
| Deploy          | Manual y riesgoso | Automatizado       |
| Testing         | Parcial           | Capas claras       |
| Documentación   | Obsoleta          | ADRs + código      |
| Escalabilidad   | Prematura         | Evolutiva          |
| Infraestructura | Compleja          | Minimalista        |
| Observabilidad  | Reactiva          | Integrada          |

---

# Herramientas y Librerías Recomendadas (Edición 2026)

| Herramienta        | Propósito                   |
| ------------------ | --------------------------- |
| `bacon`            | Feedback loop ultra-rápido  |
| `cargo-nextest`    | Testing paralelo            |
| `cargo-deny`       | Auditoría de supply chain   |
| `cargo-audit`      | Vulnerabilidades conocidas  |
| `cargo-llvm-cov`   | Cobertura                   |
| `cargo-mutants`    | Mutation testing            |
| `typos`            | Calidad textual             |
| `clippy::pedantic` | Lints avanzados             |
| `just`             | Automatización reproducible |
| `lefthook`         | Enforcement local           |
| `tracing`          | Observabilidad estructurada |

---

# Consecuencias

## ✅ Positivas

* Arquitectura extremadamente mantenible
* Feedback loop muy corto
* Fácil onboarding para IA y humanos
* Bajo costo operacional
* Refactors seguros
* Código altamente navegable
* Menor riesgo de sobreingeniería

---

## ⚠️ Negativas / Trade-offs

### Mayor disciplina requerida

El minimalismo exige:

* borrar código innecesario
* resistir abstracciones prematuras
* evitar “future-proofing” innecesario

---

### Más archivos pequeños

Puede aumentar cantidad de archivos.

Mitigación:

* naming consistente
* estructura predecible
* búsqueda rápida del editor

---

### Lints estrictos generan fricción inicial

Mitigación:

* templates
* snippets
* automatización con `just`
* integración IA

---

# Decisiones derivadas

* `cargo clippy -D warnings` obligatorio en CI
* `cargo fmt --check` obligatorio antes de merge
* `.sqlx/` debe estar sincronizado siempre
* Producción nunca compila código
* Los ADRs son la fuente oficial de decisiones técnicas
* La simplicidad operacional tiene prioridad sobre escalabilidad prematura
* Todo componente nuevo debe justificar su existencia arquitectónica
