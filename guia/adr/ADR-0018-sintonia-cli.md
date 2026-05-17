# ADR 0018 — Sintonía CLI: Generador Arquitectónico + RBAC + Guardián de Arquitectura

| Campo               | Valor                                                                                                                                           |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado — implementación progresiva desde Fase 2                                                                                             |
| **Fecha**           | 2026-05-16                                                                                                                                      |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                                                |
| **Versión**         | 2.1 (Corrección 2026-05-16)                                                                                                                     |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0005 (Migraciones SQL), ADR 0006 (RBAC + Audit), ADR 0010 (Testing), ADR 0016 (OpenAPI), ADR 0015 (Jobs), ADR 0020 (Monitoreo Regional) |

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

* `clap` v4.6.1
* `tera` v1.20.1
* `walkdir` v2.5.0
* `syn` v2.0
* `quote` v1.0.45

La CLI se implementa **después de construir manualmente 3 módulos reales**.

---

# Regla fundamental — "La regla de los 3 módulos"

El sistema NO automatiza patrones que todavía no se comprenden completamente.

## Orden obligatorio

| Módulo      | Forma de desarrollo | Objetivo                       |
| ----------- | ------------------- | ------------------------------ |
| `users`     | Manual              | Definir patrón base (auth)     |
| `sedes`     | Manual              | Confirmar consistencia CRUD    |
| `devices`   | Manual              | Validar relaciones y RBAC      |
| módulo 4+   | CLI                 | Automatizar patrón ya validado |

**Nota (2026):** Los 3 módulos iniciales corresponden al dominio de monitoreo regional (ADR 0020): `users` (RBAC base), `sedes` (entidad regional), `devices` (entidad con relaciones y métricas). No se usan módulos genéricos como `project` o `report` que no pertenecen al dominio del proyecto.

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
    │       ├── openapi.rs
    │       └── rbac.rs
    │
    ├── generators/
    │   ├── module.rs
    │   ├── entity.rs
    │   ├── repository.rs
    │   ├── use_case.rs
    │   ├── handler.rs
    │   ├── dto.rs
    │   ├── migration.rs
    │   ├── openapi.rs
    │   ├── tests.rs
    │   └── rbac.rs
    │
    ├── templates/
    │   ├── entity.rs.tera
    │   ├── repository.rs.tera
    │   ├── use_case.rs.tera
    │   ├── handler.rs.tera
    │   ├── dto.rs.tera
    │   ├── migration.sql.tera
    │   └── test.rs.tera
    │
    └── utils/
        ├── naming.rs
        ├── fs.rs
        ├── ast_editor.rs
        ├── module_registry.rs
        └── validation.rs
```

**Nota (2026):** Se elimina `proto.rs` y `proto.proto.tera` del CLI. El proyecto utiliza REST + OpenAPI (ADR 0016), no ConnectRPC/gRPC. Se elimina `rpc/` del generador.

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
* Soft Delete
* auditoría
* naming
* testing
* estructura hexagonal
* PASETO (no JWT)
* SSE (no WebSocket por defecto)

**No conoce:**

* ConnectRPC / gRPC / Protobuf (el proyecto usa REST + OpenAPI)
* JWT (prohibido por ADR 0008)

---

# Lo que genera `sintonia g module device`

```text
crates/domain/src/entities/device.rs
crates/domain/src/ports/device_repository.rs

crates/application/src/use_cases/devices/
  create_device.rs
  update_device.rs
  get_device.rs
  list_devices.rs
  soft_delete_device.rs
  update_device_status.rs

crates/database/src/repositories/
  device_repository.rs
  cached_device_repository.rs

crates/infrastructure/src/http/handlers/
  device_handler.rs
crates/infrastructure/src/http/dtos/
  device_dto.rs

tests/integration/device_api_test.rs
tests/unit/create_device_test.rs

data/migrations/
  20260515120000_create_devices.sql
```

**Nota:** No se generan archivos `.proto` ni servicios RPC. El registro OpenAPI se realiza vía AST editing en `apps/api/src/docs.rs`.

---

# Generación automática de RBAC

```sql
INSERT INTO permissions (id, name, description) VALUES
('perm_device_001', 'devices:read',   'Ver dispositivos'),
('perm_device_002', 'devices:create', 'Crear dispositivos'),
('perm_device_003', 'devices:update', 'Editar dispositivos'),
('perm_device_004', 'devices:delete', 'Archivar dispositivos'),
('perm_device_005', 'devices:export', 'Exportar dispositivos')
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'Admin'
AND p.name LIKE 'devices:%'
ON CONFLICT DO NOTHING;
```

**Nota:** Se usa `ON CONFLICT` (PostgreSQL) en lugar de `INSERT OR IGNORE` (SQLite). El proyecto usa PostgreSQL (ADR 0004).

---

# Soft Delete obligatorio

Toda entidad generada incluye:

```rust
pub deleted_at: Option<OffsetDateTime>,
```

Y las queries base:

```sql
WHERE deleted_at IS NULL
```

El CLI prohíbe:

```sql
DELETE FROM devices
```

---

# Integración automática con OpenAPI

Los handlers incluyen:

```rust
#[utoipa::path(
    get,
    path = "/api/v1/devices",
    responses(
        (status = 200, description = "Lista de dispositivos", body = [DeviceDto]),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "Sin permiso")
    ),
    security(("bearer_auth" = [])),
    tag = "devices"
)]
```

Además, el CLI registra automáticamente vía AST editing:

```rust
paths(
    device_handler::list_devices,
    device_handler::create_device,
    device_handler::get_device,
    device_handler::update_device,
    device_handler::archive_device,
)
```

en:

```rust
apps/api/src/docs.rs
```

---

# AST Editing seguro

Sintonía evita manipulación frágil basada únicamente en strings.

Usa:

* `syn` v2.0
* `quote` v1.0.45

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
```

Estos marcadores son:

* obligatorios
* permanentes
* nunca deben eliminarse

**Nota:** Se elimina `// sintonia:rpc`. El proyecto no usa ConnectRPC/gRPC.

---

# Naming inteligente

```text
device
→ Device
→ devices
→ device_id
→ DeviceDto
→ CreateDeviceRequest
→ ListDevicesResponse
→ DeviceStatus (enum si aplica)
```

Incluye pluralización automática y convenciones internas del proyecto.

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
```

**Nota:** Se elimina `sintonia g proto`. El proyecto no usa Protobuf.

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
| `time` crate (no `chrono`) | ✔    |
| `jsonwebtoken` ausente | ✔          |
| `pasetors` presente    | ✔          |

**Nota:** Se elimina "`.proto` sincronizado" y "ConnectRPC sincronizado" de las validaciones. El proyecto usa REST + OpenAPI (ADR 0016).

---

# Ejemplo de salida

```bash
✔ crates/domain limpio
✔ PASETO configurado
✔ Soft Delete detectado
✔ OpenAPI sincronizado
✔ REST API válida
✔ time crate en uso (no chrono)

Estado: Arquitectura válida
```

---

# Ejemplo de error

```bash
✗ sqlx encontrado en crates/domain
✗ jsonwebtoken encontrado en apps/api
✗ DELETE FROM devices detectado
✗ Endpoint sin OpenAPI
✗ chrono encontrado en crates/domain (usar time)
```

---

# Idempotencia

El CLI nunca sobrescribe archivos existentes sin confirmación explícita.

```bash
⚠ device_handler.rs ya existe
Usar --force para reemplazar
```

---

# Testing generado automáticamente

Cada módulo incluye:

## Unit tests

```rust
#[tokio::test]
async fn should_create_device() {
    // Arrange
    let repo = InMemoryDeviceRepository::new();
    let use_case = CreateDeviceUseCase::new(repo);

    // Act
    let result = use_case.execute(CreateDeviceInput { ... }).await;

    // Assert
    assert!(result.is_ok());
}
```

## Integration tests

```rust
#[tokio::test]
async fn should_return_201_on_create_device() {
    let app = spawn_app().await;
    let token = app.login_admin().await;

    let response = app
        .client
        .post("/api/v1/devices")
        .bearer_auth(token)
        .json(&json!({ ... }))
        .send()
        .await;

    assert_eq!(response.status(), 201);
}
```

---

# Estrategia de implementación

| Fase   | Objetivo                               |
| ------ | -------------------------------------- |
| Fase 0 | 3 módulos manuales (users, sedes, devices) |
| Fase 1 | Generación básica (entity, repository, use case, handler, DTO, migration) |
| Fase 2 | AST editing (routers, OpenAPI registry) |
| Fase 3 | Guardián arquitectónico (`check arch`) |
| Fase 4 | Generación OpenAPI completa + RBAC seeds automáticos |

**Nota:** Se elimina "Generación ConnectRPC/OpenAPI completa" de la Fase 4. El proyecto solo usa OpenAPI (ADR 0016).

---

# Alternativas consideradas

| Opción                      | Motivo de descarte               |
| --------------------------- | -------------------------------- |
| `cargo-generate`            | No entiende RBAC ni arquitectura |
| Bash scripts                | Frágiles y sin tipado            |
| Node generators             | Fuera del stack Rust             |
| Yeoman/Plop                 | Dependencia extra innecesaria    |
| Automatizar desde el inicio | Patrones aún no comprendidos     |
| ConnectRPC generators       | Proyecto usa REST + OpenAPI, no gRPC |

---

# Herramientas y Librerías para Optimizar (Edición 2026-05-16)

| Herramienta      | Versión fijada | Propósito                            |
| ---------------- | -------------- | ------------------------------------ |
| `clap`           | `4.6.1`        | CLI robusta y tipada                 |
| `tera`           | `1.20.1`       | Templates reutilizables              |
| `syn`            | `2.0`          | Parsing AST Rust                     |
| `quote`          | `1.0.45`       | Generación AST                       |
| `walkdir`        | `2.5.0`        | Exploración de archivos              |
| `comfy-table`    | `7.2.2`        | Output visual profesional            |
| `clap_mangen`    | `0.3.0`        | Generación automática de páginas man |
| `cargo_metadata` | `0.23.1`       | Inspección del workspace Rust        |
| `time`           | `0.3.47`       | Crate de fechas (no `chrono`)        |
| `cargo-nextest`  | `0.9.135`      | Runner de tests (CI y local)         |

**Cambios respecto a v2.0:**
- Se fijan versiones exactas de todas las dependencias del CLI basadas en latest estable al 2026-05-16
- `clap` 4.6.1 (2026-05-14), `tera` 1.20.1 (2025-10-30), `quote` 1.0.45 (2026-03-03)
- `walkdir` 2.5.0, `comfy-table` 7.2.2 (2026-01-13), `cargo_metadata` 0.23.1
- `time` 0.3.47 (2026-02-04), `cargo-nextest` 0.9.135 (2026-05-14)
- `clap_mangen` 0.3.0 (compatible con clap 4.6.x)
- `syn` 2.0 (latest estable v2.x, compatible con quote 1.0.45)
- Se eliminan referencias a ConnectRPC/gRPC/Protobuf en toda la arquitectura y validaciones

---

# Consecuencias

## ✅ Positivas

* Arquitectura consistente en todos los módulos
* RBAC imposible de olvidar
* OpenAPI sincronizado automáticamente
* Soft Delete garantizado
* Menos errores humanos
* Desarrollo mucho más rápido
* Onboarding más simple para nuevos developers
* Validación arquitectónica automática en CI

---

## ⚠️ Negativas / Trade-offs

### El CLI se convierte en un producto interno

→ requiere mantenimiento continuo

→ mitigado porque vive dentro del monorepo y sigue las mismas reglas de calidad

→ tests del CLI corren con `cargo nextest run -p cli`

---

### Riesgo de developers que no entienden el sistema

→ mitigado con la regla obligatoria de los 3 módulos manuales

→ nadie usa el generador sin comprender primero la arquitectura

→ `sintonia check arch` detecta desviaciones

---

### AST editing añade complejidad

→ mitigado usando `syn` v2.0 en lugar de regex frágiles

→ los marcadores limitan el alcance de modificaciones

→ tests de integración validan que los edits no rompen compilación

---

# Decisiones derivadas

* Todos los módulos nuevos se crean con `sintonia g module`
* Soft Delete es obligatorio por defecto
* OpenAPI se registra automáticamente vía AST editing
* `sintonia check arch` corre en CI y pre-push hooks (lefthook)
* Los marcadores `// sintonia:*` son inamovibles
* El CLI vive en `apps/cli/`
* Los templates `.tera` son parte crítica del sistema y se versionan en git
* **No se generan archivos `.proto` ni servicios RPC** — el proyecto usa REST + OpenAPI
* **JWT está prohibido** — el generador usa PASETO en todos los handlers y tests
* **Se usa `time` crate `0.3.47`** — el generador no usa `chrono` en entidades ni migraciones
* Los seeds RBAC se generan para PostgreSQL (`ON CONFLICT`), no SQLite
* El CLI respeta la estructura de crates del workspace: `domain`, `application`, `database`, `infrastructure`
* Los tests del CLI usan `cargo nextest` v0.9.135 como runner oficial

---

# Historial de cambios

| Versión | Fecha       | Cambios realizados |
| ------- | ----------- | ------------------ |
| 1.0     | 2026 (orig) | Versión inicial con ConnectRPC, Protobuf, `project`/`report` como módulos de ejemplo, SQLite (`INSERT OR IGNORE`), `chrono` implícito |
| 2.0     | 2026-05-16  | Elimina ConnectRPC/gRPC/Protobuf del CLI y validaciones; reemplaza módulos genéricos (`project`, `report`, `acta`) por dominio real (`users`, `sedes`, `devices`); actualiza SQL a PostgreSQL (`ON CONFLICT`); agrega validación de `time` vs `chrono` y `jsonwebtoken` ausente; actualiza Fase 4; actualiza ejemplos de salida |
| 2.1     | 2026-05-16  | Fija versiones exactas de todas las dependencias del CLI: `clap` 4.6.1, `tera` 1.20.1, `syn` 2.0, `quote` 1.0.45, `walkdir` 2.5.0, `comfy-table` 7.2.2, `clap_mangen` 0.3.0, `cargo_metadata` 0.23.1, `time` 0.3.47, `cargo-nextest` 0.9.135; compatibilidad verificada entre `syn` 2.0 y `quote` 1.0.45; elimina toda referencia residual a Protobuf/ConnectRPC |
