# ADR 0028 — Sintonía CLI: Generador Arquitectónico + RBAC + Guardián de Arquitectura

| Campo               | Valor                                                                                                                                           |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado — implementación progresiva desde Fase 2                                                                                             |
| **Fecha**           | 2026                                                                                                                                            |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                                                |
| **Relacionado con** | ADR 0001 (Monolito Modular), ADR 0005 (Migraciones SQL), ADR 0006 (RBAC + Audit), ADR 0010 (Testing), ADR 0021 (OpenAPI), ADR 0027 (ConnectRPC) |

---

# Contexto

La arquitectura hexagonal modular del Laboratorio 3030 prioriza:

* separación estricta entre dominio e infraestructura
* RBAC obligatorio
* Soft Delete por defecto
* auditoría de acciones críticas
* generación de contratos y documentación
* consistencia estructural entre módulos

Sin automatización, cada nuevo módulo requiere repetir:

* entidades
* puertos
* repositorios
* use cases
* handlers
* DTOs
* migraciones
* seeds RBAC
* tests
* registro en routers
* documentación OpenAPI

Esto consume tiempo y genera inconsistencias entre developers.

Además, conforme el sistema crece, existe el riesgo de:

* imports prohibidos entre crates
* bypass de Soft Delete
* uso accidental de JWT en lugar de PASETO
* módulos sin permisos RBAC
* endpoints no registrados en OpenAPI

Necesitamos una herramienta que:

* automatice patrones repetitivos
* haga cumplir la arquitectura
* reduzca errores humanos
* mantenga convenciones idénticas en todos los módulos
* permita escalar el sistema sin degradar calidad

---

# Decisión

Desarrollar **Sintonía CLI**, una herramienta interna escrita en Rust usando:

* `clap`
* `tera`
* `walkdir`
* `syn`
* `quote`

La CLI se implementa **después de construir manualmente 3 módulos reales**.

---

# Regla fundamental — “La regla de los 3 módulos”

El sistema NO automatiza patrones que todavía no se comprenden completamente.

## Orden obligatorio

| Módulo    | Forma de desarrollo | Objetivo                       |
| --------- | ------------------- | ------------------------------ |
| `user`    | Manual              | Definir patrón base            |
| `project` | Manual              | Confirmar consistencia         |
| `report`  | Manual              | Validar relaciones y RBAC      |
| módulo 4+ | CLI                 | Automatizar patrón ya validado |

La automatización viene después del entendimiento.

---

# Arquitectura del CLI

```text
apps/cli/
├── Cargo.toml
└── src/
    ├── main.rs
    │
    ├── commands/
    │   ├── new.rs
    │   ├── doctor.rs
    │   ├── check.rs
    │   ├── db.rs
    │   └── generate/
    │       ├── module.rs
    │       ├── entity.rs
    │       ├── migration.rs
    │       └── proto.rs
    │
    ├── generators/
    │   ├── module.rs
    │   ├── entity.rs
    │   ├── repository.rs
    │   ├── use_case.rs
    │   ├── handler.rs
    │   ├── dto.rs
    │   ├── migration.rs
    │   ├── proto.rs
    │   ├── tests.rs
    │   ├── openapi.rs
    │   └── rbac.rs
    │
    ├── templates/
    │   ├── entity.rs.tera
    │   ├── repository.rs.tera
    │   ├── use_case.rs.tera
    │   ├── handler.rs.tera
    │   ├── dto.rs.tera
    │   ├── migration.sql.tera
    │   ├── proto.proto.tera
    │   └── test.rs.tera
    │
    └── utils/
        ├── naming.rs
        ├── fs.rs
        ├── ast_editor.rs
        ├── module_registry.rs
        └── validation.rs
```

---

# Filosofía del generador

Sintonía CLI no es solamente scaffolding.

Es un:

* generador
* validador
* guardián arquitectónico
* enforce de convenciones

El CLI conoce:

* RBAC
* OpenAPI
* ConnectRPC
* Soft Delete
* auditoría
* naming
* testing
* estructura hexagonal

---

# Lo que genera `sintonia g module acta`

```text
crates/domain/src/entities/acta.rs
crates/domain/src/ports/acta_repository.rs

crates/application/src/use_cases/
  create_acta.rs
  update_acta.rs
  get_acta.rs
  list_actas.rs
  soft_delete_acta.rs

crates/database/src/repositories/
  sqlite_acta_repository.rs
  cached_acta_repository.rs

crates/infrastructure/src/http/handlers/
  acta_handler.rs
  acta_dto.rs

crates/infrastructure/src/rpc/
  acta_service.rs

proto/acta/v1/acta.proto

tests/integration/acta_api_test.rs
tests/unit/create_acta_test.rs

data/migrations/
  20260515_create_actas.sql
```

---

# Generación automática de RBAC

```sql
INSERT OR IGNORE INTO permissions (id, name, description) VALUES
('perm_acta_001', 'actas:read',   'Ver actas'),
('perm_acta_002', 'actas:create', 'Crear actas'),
('perm_acta_003', 'actas:update', 'Editar actas'),
('perm_acta_004', 'actas:delete', 'Eliminar actas');

INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'Admin'
AND p.name LIKE 'actas:%';
```

---

# Soft Delete obligatorio

Toda entidad generada incluye:

```rust
pub deleted_at: Option<DateTime<Utc>>,
```

Y las queries base:

```sql
WHERE deleted_at IS NULL
```

El CLI prohíbe:

```sql
DELETE FROM users
```

---

# Integración automática con OpenAPI

Los handlers incluyen:

```rust
#[utoipa::path]
```

Además, el CLI registra automáticamente:

```rust
paths(
    acta_handler::create_acta,
    acta_handler::list_actas,
)
```

en:

```rust
apps/api/src/docs.rs
```

---

# Integración automática con ConnectRPC

```protobuf
service ActaService {
    rpc CreateActa(CreateActaRequest)
        returns (CreateActaResponse);

    rpc ListActas(ListActasRequest)
        returns (ListActasResponse);
}
```

Después:

```bash
buf generate
```

Los tipos frontend/backend quedan sincronizados automáticamente.

---

# AST Editing seguro

Sintonía evita manipulación frágil basada únicamente en strings.

Usa:

* `syn`
* `quote`

para modificar:

* routers
* módulos
* registros OpenAPI
* exports

de forma segura.

---

# Marcadores obligatorios

```rust
// sintonia:routes
// sintonia:modules
// sintonia:openapi
// sintonia:rpc
```

Estos marcadores son:

* obligatorios
* permanentes
* nunca deben eliminarse

---

# Naming inteligente

```text
acta
→ Acta
→ actas
→ acta_id
→ ActaDto
→ CreateActaRequest
→ ListActasResponse
```

Incluye pluralización automática y convenciones internas.

---

# Comandos principales

## Proyecto

```bash
sintonia new <proyecto>
sintonia doctor
```

---

## Generación

```bash
sintonia g module <nombre>

sintonia g module <nombre> --dry-run

sintonia g module <nombre> --no-rbac

sintonia g entity <nombre>

sintonia g migration <nombre>

sintonia g proto <nombre>
```

---

## Base de datos

```bash
sintonia db migrate
sintonia db seed
sintonia db reset
```

---

## Validación arquitectónica

```bash
sintonia check arch
```

---

# `sintonia check arch`

La CLI valida automáticamente:

| Regla                  | Validación |
| ---------------------- | ---------- |
| `domain` sin `sqlx`    | ✔          |
| `domain` sin `axum`    | ✔          |
| JWT prohibido          | ✔          |
| PASETO obligatorio     | ✔          |
| Soft Delete activo     | ✔          |
| DTOs fuera del dominio | ✔          |
| OpenAPI registrado     | ✔          |
| `.proto` sincronizado  | ✔          |

---

# Ejemplo de salida

```bash
✔ crates/domain limpio
✔ PASETO configurado
✔ Soft Delete detectado
✔ OpenAPI sincronizado
✔ ConnectRPC sincronizado

Estado: Arquitectura válida
```

---

# Ejemplo de error

```bash
✗ sqlx encontrado en crates/domain
✗ jsonwebtoken encontrado en apps/api
✗ DELETE FROM users detectado
✗ Endpoint sin OpenAPI
```

---

# Idempotencia

El CLI nunca sobrescribe archivos existentes sin confirmación explícita.

```bash
⚠ acta_handler.rs ya existe
Usar --force para reemplazar
```

---

# Testing generado automáticamente

Cada módulo incluye:

## Unit tests

```rust
#[tokio::test]
async fn should_create_acta() {}
```

## Integration tests

```rust
#[tokio::test]
async fn should_return_201() {}
```

---

# Estrategia de implementación

| Fase   | Objetivo                               |
| ------ | -------------------------------------- |
| Fase 0 | 3 módulos manuales                     |
| Fase 1 | Generación básica                      |
| Fase 2 | AST editing                            |
| Fase 3 | Guardián arquitectónico                |
| Fase 4 | Generación ConnectRPC/OpenAPI completa |

---

# Alternativas consideradas

| Opción                      | Motivo de descarte               |
| --------------------------- | -------------------------------- |
| `cargo-generate`            | No entiende RBAC ni arquitectura |
| Bash scripts                | Frágiles y sin tipado            |
| Node generators             | Fuera del stack Rust             |
| Yeoman/Plop                 | Dependencia extra innecesaria    |
| Automatizar desde el inicio | Patrones aún no comprendidos     |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta      | Propósito                            |
| ---------------- | ------------------------------------ |
| `clap`           | CLI robusta y tipada                 |
| `tera`           | Templates reutilizables              |
| `syn`            | Parsing AST Rust                     |
| `quote`          | Generación AST                       |
| `walkdir`        | Exploración de archivos              |
| `comfy-table`    | Output visual profesional            |
| `clap_mangen`    | Generación automática de páginas man |
| `cargo_metadata` | Inspección del workspace Rust        |

---

# Consecuencias

## ✅ Positivas

* Arquitectura consistente en todos los módulos
* RBAC imposible de olvidar
* OpenAPI sincronizado automáticamente
* ConnectRPC sincronizado automáticamente
* Soft Delete garantizado
* Menos errores humanos
* Desarrollo mucho más rápido
* Onboarding más simple para nuevos developers

---

## ⚠️ Negativas / Trade-offs

### El CLI se convierte en un producto interno

→ requiere mantenimiento continuo

→ mitigado porque vive dentro del monorepo y sigue las mismas reglas de calidad

---

### Riesgo de developers que no entienden el sistema

→ mitigado con la regla obligatoria de los 3 módulos manuales

→ nadie usa el generador sin comprender primero la arquitectura

---

### AST editing añade complejidad

→ mitigado usando `syn` en lugar de regex frágiles

→ los marcadores limitan el alcance de modificaciones

---

# Decisiones derivadas

* Todos los módulos nuevos se crean con `sintonia g module`
* Soft Delete es obligatorio por defecto
* OpenAPI y ConnectRPC se registran automáticamente
* `sintonia check arch` corre en CI y pre-push hooks
* Los marcadores `// sintonia:*` son inamovibles
* El CLI vive en `apps/cli/`
* Los templates `.tera` son parte crítica del sistema y se versionan en git
