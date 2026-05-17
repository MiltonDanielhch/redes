# ADR 0011 — Estándares de Desarrollo: Ciclo Lab → Puente → Producción

> **Última revisión de versiones:** 2026-05-16  
> Se actualizaron las versiones de herramientas tras auditoría contra crates.io, GitHub y docs.rs.

| Campo               | Valor                                                                                                                    |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                                               |
| **Fecha**           | 2026-05-16                                                                                                               |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                         |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0010 (Testing), ADR 0012 (just + lefthook), ADR 0013 (Deploy), ADR 0018 (Sintonía CLI) |
| **Última revisión** | 2026-05-16 — Alineación de herramientas con roadmap + versiones actualizadas |

---

## Contexto

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

## Decisión

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

antes que "enterprise patterns" innecesarios.

---

## 1 — Restricciones de atomicidad

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

> "Siempre deja el código un poco más limpio de como lo encontraste."

Cada commit debe:

* simplificar
* eliminar duplicación
* mejorar naming
* reducir complejidad

aunque el cambio principal sea otro.

---

## 2 — Ciclo oficial de desarrollo

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

| Herramienta   | Uso                      | Versión mínima | Estado |
| ------------- | ------------------------ | --------------- | ------ |
| `bacon`       | feedback en tiempo real  | 3.22.0          | ✅ Activa |
| `cargo-watch` | recompilación automática | 8.5.3           | 🟡 Legacy — usar `bacon` |
| `just`        | comandos reproducibles   | 1.51.0          | ✅ Activa |
| `nextest`     | tests paralelos          | 0.9.135         | ✅ Activa |
| `clippy`      | linting estricto         | (via toolchain) | ✅ Activa |
| `typos`       | corrección ortográfica   | 1.46.2          | ✅ Activa |
| `cargo-deny`  | auditoría dependencias   | 0.19.6          | ✅ Activa |
| `cargo-audit` | vulnerabilidades         | 0.22.1          | ✅ Activa |

**Notas:**
- `bacon` reemplaza a `cargo-watch` — mejor UX y soporte Rust 2024
- `cargo-mutants` requiere Rust 1.88+ — **ya disponible** (toolchain actual: 1.95.0)
- `cargo-llvm-cov` opcional para cobertura visual

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
just prepare          # SQLx offline query verification
just types-check      # TypeScript/OpenAPI sync verification (ADR 0016)
```

---

## Reglas del CI

El CI debe fallar si:

* existe un warning
* `.sqlx/` está desactualizado (queries SQLx no verificadas)
* hay dependencias vulnerables (`cargo audit` falla)
* existe drift en tipos TS vs OpenAPI (`just types-check` falla)
* falla cualquier test
* se rompe la arquitectura (domain importa sqlx/axum)

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

## 3 — Código autodocumentado

## Regla principal

Los comentarios deben explicar:

* el "por qué"
* la decisión de negocio
* la razón arquitectónica

NO el "cómo".

---

## Ejemplo correcto

```rust
//! Ubicación: `crates/domain/src/entities/user.rs`
//!
//! Descripción: Entidad User con Soft Delete (ADR 0006).
//!
//! ADRs: 0006, 0011

// Soft Delete preserva audit_logs históricos (ADR 0006)
user.soft_delete();
```

---

## Ejemplo incorrecto

```rust
// Iterar usuarios y filtrar activos — el CÓMO es obvio del código
users.iter().filter(|u| u.is_active())
```

---

## 4 — Tipos fuertes sobre primitivas

## Incorrecto

```rust
//! Ubicación: `crates/application/src/use_cases/create_user.rs`
//!
//! Descripción: Ejemplo ANTI-PATRÓN — primitivas en lugar de newtypes.
//!
//! ADRs: 0011

fn create_user(id: String, role: String)
```

---

## Correcto

```rust
//! Ubicación: `crates/application/src/use_cases/create_user.rs`
//!
//! Descripción: Ejemplo correcto — newtypes para dominio tipado.
//!
//! ADRs: 0011, 0001

fn create_user(id: UserId, role: UserRole)
```

---

## 5 — Filosofía de dependencias

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

## 6 — Convenciones para IA

El proyecto está diseñado explícitamente para colaboración humano + IA.

---

## Reglas

### Nombres explícitos

```rust
//! Ubicación: `crates/application/src/use_cases/create_user_use_case.rs`
//!
//! Descripción: Caso de uso para creación de usuarios.
//!
//! ADRs: 0011

// CORRECTO: nombre explícito
create_user_use_case.rs
sqlite_user_repository.rs
```

```rust
//! Ubicación: `crates/application/src/helpers.rs`
//!
//! Descripción: ANTI-PATRÓN — nombre genérico.
//!
//! ADRs: 0011

// INCORRECTO: nombre genérico
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

Evita archivos "multi-propósito".

---

## 7 — Política de complejidad operacional

Con un servidor físico potente propio:

* se elimina dependencia de VPS externos baratos
* se evita infraestructura distribuida innecesaria
* se prioriza operación simple y local-first

---

## Consecuencia arquitectónica

Quedan despriorizados:

* multi-node premature scaling
* Kubernetes
* Redis "porque sí"
* microservicios
* service mesh
* colas distribuidas innecesarias

---

## Principio

> "La complejidad operacional es deuda técnica."

---

## 8 — Comparativa SDLC

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

## Herramientas y Librerías Recomendadas (Edición 2026)

| Herramienta        | Propósito                   | Versión mínima | Estado |
| ------------------ | --------------------------- | --------------- | ------ |
| `bacon`            | Feedback loop ultra-rápido  | 3.22.0          | ✅ Activa |
| `cargo-nextest`    | Testing paralelo            | 0.9.135         | ✅ Activa |
| `cargo-deny`       | Auditoría de supply chain   | 0.19.6          | ✅ Activa |
| `cargo-audit`      | Vulnerabilidades conocidas  | 0.22.1          | ✅ Activa |
| `cargo-llvm-cov`   | Cobertura                   | 0.8.7           | 🟡 Opcional |
| `cargo-mutants`    | Mutation testing            | 27.0.0          | ✅ Disponible (MSRV 1.88) |
| `typos`            | Calidad textual             | 1.46.2          | ✅ Activa |
| `clippy::pedantic` | Lints avanzados             | (toolchain)     | ✅ Activa |
| `just`             | Automatización reproducible | 1.51.0          | ✅ Activa |
| `lefthook`         | Enforcement local           | 2.1.6           | ✅ Activa |
| `tracing`          | Observabilidad estructurada | (workspace)     | ✅ Activa |

---

## Consecuencias

### ✅ Positivas

* Arquitectura extremadamente mantenible
* Feedback loop muy corto
* Fácil onboarding para IA y humanos
* Bajo costo operacional
* Refactors seguros
* Código altamente navegable
* Menor riesgo de sobreingeniería

### ⚠️ Negativas / Trade-offs

**Mayor disciplina requerida**

El minimalismo exige:

* borrar código innecesario
* resistir abstracciones prematuras
* evitar "future-proofing" innecesario

---

**Más archivos pequeños**

Puede aumentar cantidad de archivos.

Mitigación:

* naming consistente
* estructura predecible
* búsqueda rápida del editor

---

**Lints estrictos generan fricción inicial**

Mitigación:

* templates
* snippets
* automatización con `just`
* integración IA

---

## Decisiones derivadas

* `cargo clippy -D warnings` obligatorio en CI
* `cargo fmt --check` obligatorio antes de merge
* `.sqlx/` debe estar sincronizado siempre
* Producción nunca compila código
* Los ADRs son la fuente oficial de decisiones técnicas
* La simplicidad operacional tiene prioridad sobre escalabilidad prematura
* Todo componente nuevo debe justificar su existencia arquitectónica
* `bacon` es la herramienta oficial de feedback loop (reemplaza `cargo-watch`)
* `cargo-mutants` ya está disponible para el toolchain actual (Rust 1.95.0)

---

## Registro de cambios de versiones

| Fecha | Componente | Anterior | Actual | Notas |
|-------|------------|----------|--------|-------|
| 2026-05-16 | just | 1.40 | **1.51.0** | Nuevas funciones: módulos, `[no-cd]`, path functions |
| 2026-05-16 | nextest | 0.9 | **0.9.135** | Runner de tests actualizado |
| 2026-05-16 | typos | 1.46.1 | **1.46.2** | Última estable (16 may 2026). Fix: no corrige a `criterias` ni `replaceables` |
| 2026-05-16 | cargo-deny | 0.18 | **0.19.6** | Fix de segfault (0.19.5), fix de advisory parsing (0.19.4), SARIF fixes (0.19.2) |
| 2026-05-16 | cargo-audit | 0.21 | **0.22.1** | Última estable (feb 2026). MSRV 1.85.0 |
| 2026-05-16 | cargo-llvm-cov | 0.6.16 | **0.8.7** | Salto de versión major. Requiere Rust ≥ 1.95.0 |
| 2026-05-16 | cargo-mutants | 27.0.0 (postergado) | **27.0.0** | Ya disponible. MSRV 1.88 (cubierto por toolchain 1.95.0) |
| 2026-05-16 | lefthook | 1.11 | **2.1.6** | Salto a v2. Fixes de packaging, normalización de paths, soporte git debug |
