# ADR 0027 — Comunicación: ConnectRPC + Buf + Contratos Multi-Plataforma

| Campo               | Valor                                                                             |
| ------------------- | --------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado — implementación gradual desde Fase 2                                  |
| **Fecha**           | 2026                                                                              |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                  |
| **Relacionado con** | ADR 0003 (Axum), ADR 0021 (OpenAPI), ADR 0022 (SvelteKit), ADR 0035 (Monitoreo Regional) |

---

# Contexto

El ecosistema del proyecto actual se enfoca en:

* Web (SvelteKit)
* API REST (Axum)
* Agentes de monitoreo distribuidos
* Futuros clientes móviles (post-MVP)

> **Nota:** Desktop (Tauri) y Mobile nativo no son parte del MVP actual (ADR 0035).

REST tradicional funciona bien para MVPs, pero genera problemas cuando el sistema crece:

* DTOs duplicados entre frontend/backend
* contratos desincronizados
* documentación que no coincide con implementación
* validaciones repetidas
* payloads JSON pesados para conexiones móviles lentas
* dificultad para compartir tipos entre plataformas

Además:

* gRPC tradicional requiere infraestructura adicional para funcionar en browsers
* GraphQL añade complejidad innecesaria para este stack
* OpenAPI genera clientes inconsistentes dependiendo del generador

Necesitamos:

* contratos estrictos y tipados
* generación automática de clientes
* compatibilidad nativa con navegador
* payloads eficientes
* soporte multiplataforma real
* integración limpia con Axum

---

# Decisión

Adoptar **ConnectRPC** con **Protocol Buffers (proto3)** como estándar principal de comunicación interna y externa desde Fase 2.

El stack queda compuesto por:

| Componente           | Tecnología                |
| -------------------- | ------------------------- |
| Contrato             | Protocol Buffers (proto3) |
| Transporte           | ConnectRPC                |
| Generación de código | Buf                       |
| Backend              | Axum + Tower              |
| Frontend             | Connect-Web               |
| Mobile               | Kotlin + Swift generators |
| Desktop              | Prost                     |

---

# Objetivos arquitectónicos

## 1 — Single Source of Truth

El archivo `.proto` es la única fuente de verdad.

No se escriben DTOs manuales en TypeScript.

---

## 2 — Compile-Time Safety

Si cambia un campo:

* `buf generate`
* el compilador detecta todos los usos rotos
* error antes de producción

---

## 3 — Browser Native

ConnectRPC funciona sobre:

* HTTP/1.1
* fetch API
* sin Envoy
* sin gRPC-Web proxy

Ideal para VPS pequeños.

---

# Arquitectura de contratos

```text
proto/
├── user/v1/
│   └── user.proto
│
├── auth/v1/
│   └── auth.proto
│
├── devices/v1/
│   └── devices.proto
│
└── alerts/v1/
    └── alerts.proto
```

---

# Ejemplo de contrato

```proto
// proto/user/v1/user.proto

syntax = "proto3";

package user.v1;

service UserService {
    rpc GetUser    (GetUserRequest)    returns (GetUserResponse);
    rpc CreateUser (CreateUserRequest) returns (CreateUserResponse);
    rpc ListUsers  (ListUsersRequest)  returns (ListUsersResponse);
}

message User {
    string id             = 1;
    string email          = 2;
    bool   email_verified = 3;
    string created_at     = 4;
}

message GetUserRequest {
    string id = 1;
}

message GetUserResponse {
    User user = 1;
}

message CreateUserRequest {
    string email    = 1;
    string password = 2;
}

message CreateUserResponse {
    User user = 1;
}

message ListUsersRequest {
    int32 page     = 1;
    int32 per_page = 2;
}

message ListUsersResponse {
    repeated User users = 1;
    int32 total         = 2;
}
```

---

# Integración con Axum

ConnectRPC se monta sobre el mismo servidor HTTP del backend.

No existe proceso gRPC separado.

```rust
// apps/api/src/router.rs

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", rest_router(state.clone()))
        .nest("/rpc", connectrpc_router(state))
        .route("/health", get(health_handler))
}
```

---

# Generación de código con Buf

## Instalación

```bash
brew install bufbuild/buf/buf
```

o vía:

```bash
just setup
```

---

# Configuración

## `buf.yaml`

```yaml
version: v1

lint:
  use:
    - STANDARD

breaking:
  use:
    - FILE
```

---

## `buf.gen.yaml`

```yaml
version: v1

plugins:
  - plugin: buf.build/connectrpc/es
    out: apps/web/src/lib/gen

  - plugin: buf.build/protocolbuffers/rust
    out: crates/proto/src

  - plugin: buf.build/protocolbuffers/kotlin
    out: apps/mobile/android/generated

  - plugin: buf.build/apple/swift
    out: apps/mobile/ios/generated
```

---

# Flujo de trabajo

```text
1. Editar .proto
2. Ejecutar buf generate
3. Clientes regenerados automáticamente
4. TypeScript/Rust/Kotlin/Swift quedan sincronizados
5. El compilador detecta usos inválidos
```

---

# Integración frontend (SvelteKit)

```ts
import { createPromiseClient } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";

import { UserService } from "$lib/gen/user/v1/user_connect";

const transport = createConnectTransport({
    baseUrl: "https://api.tudominio.gob.bo",
});

export const client =
    createPromiseClient(UserService, transport);

const response = await client.getUser({
    id: "123",
});
```

---

# Estrategia de migración

## Fase 1 — MVP

* REST + OpenAPI
* DTOs TypeScript manuales
* ConnectRPC opcional

## Fase 2

* Introducción de `.proto`
* `buf generate`
* Frontend migra gradualmente
* REST continúa funcionando

## Fase 3

* ConnectRPC como estándar principal
* REST solo para integraciones públicas externas

---

# Validación compartida

Se adopta `protovalidate` para reglas universales.

```proto
message CreateUserRequest {
    string email = 1 [
        (buf.validate.field).string.email = true
    ];

    string password = 2 [
        (buf.validate.field).string.min_len = 8
    ];
}
```

Las reglas se aplican automáticamente en:

* backend Rust
* frontend TS
* Android
* iOS

---

# Tiempo real

ConnectRPC permite:

* unary RPC
* server streaming
* client streaming
* bidirectional streaming

Para el proyecto:

| Caso               | Estrategia         |
| ------------------ | ------------------ |
| Dashboard realtime | SSE                |
| Logs streaming     | Server streaming   |
| Eventos críticos   | WebSocket opcional |

---

# Seguridad

## Reglas

* Auth mediante PASETO (ADR 0008)
* Cookies HttpOnly Secure
* TLS obligatorio
* Rate limit desde Axum/Tower

---

# Integración con OpenAPI

ConnectRPC NO reemplaza OpenAPI.

Ambos conviven:

| Tecnología | Uso                         |
| ---------- | --------------------------- |
| OpenAPI    | documentación pública       |
| ConnectRPC | comunicación interna tipada |

---

# Commands útiles

## Generar tipos

```bash
buf generate
```

## Lint de contratos

```bash
buf lint
```

## Verificar breaking changes

```bash
buf breaking
```

## Pipeline completo

```bash
just types
```

---

# `justfile`

```makefile
types:
    buf generate
    pnpm check

types-check:
    buf generate
    git diff --exit-code
```

---

# Docker / CI

## Containerfile

```dockerfile
RUN buf generate
```

El CI falla si:

* falta código generado
* el proto rompe compatibilidad
* existen breaking changes

---

# Alternativas consideradas

| Opción           | Motivo de descarte                               |
| ---------------- | ------------------------------------------------ |
| REST puro        | DTOs duplicados y contratos débiles              |
| gRPC tradicional | Requiere proxy extra para browsers               |
| GraphQL          | Complejidad innecesaria                          |
| tRPC             | Excelente pero limitado al ecosistema TypeScript |
| OpenAPI codegen  | Generadores inconsistentes                       |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta     | Propósito                                  |
| --------------- | ------------------------------------------ |
| `connect-query` | Integración con TanStack Query             |
| `protovalidate` | Validación universal basada en proto       |
| `buf lint`      | Validación de contratos                    |
| `buf breaking`  | Detección de breaking changes              |
| `axum-connect`  | Integración limpia con Axum                |
| `buf studio`    | Testing visual de RPCs                     |
| `grpcurl`       | Debugging desde terminal                   |
| `cargo-watch`   | Regeneración automática durante desarrollo |

---

# Consecuencias

## ✅ Positivas

* Contratos estrictos y compartidos
* Un solo `.proto` para todas las plataformas
* Eliminación de DTOs manuales
* Payloads más pequeños que JSON
* Preparado para móvil desde el inicio
* Compatible con navegadores sin infraestructura pesada
* Compile-time safety real

---

## ⚠️ Negativas / Trade-offs

### Curva de aprendizaje de Protocol Buffers

→ Mitigación:

* proto3 es pequeño y simple
* ejemplos base incluidos
* `buf lint` ayuda automáticamente

---

### Requiere herramientas adicionales

→ Mitigación:

* `just setup` instala buf
* CI valida automáticamente

---

### Payloads binarios difíciles de inspeccionar

→ Mitigación:

* mantener REST/OpenAPI para debugging
* usar `grpcurl` y `buf studio`

---

### Código generado no debe editarse

→ Mitigación:

* carpetas `/generated` ignoradas manualmente
* regeneración automática en CI

---

# Decisiones derivadas

* Todos los contratos viven en `/proto`
* `buf` gestiona generación y lints
* ConnectRPC comparte el mismo servidor Axum
* El frontend consume únicamente tipos generados
* `buf generate` forma parte obligatoria del CI
* Los archivos generados nunca se editan manualmente
* OpenAPI sigue existiendo para documentación pública
* ConnectRPC será obligatorio para móvil y desktop en Fase 3
