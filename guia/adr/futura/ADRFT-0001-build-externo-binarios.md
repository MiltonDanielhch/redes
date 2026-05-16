# ADR (futuro) — Build Externo: Distroless ~10MB

| Campo               | Valor                                                                   |
| ------------------- | ----------------------------------------------------------------------- |
| **Estado**          | ⏳ Pendiente                                                              |
| **Fecha**           | 2026                                                                    |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                        |
| **Relacionado con** | ADR 0013 (Containerfile + Kamal), ADR 0010 (CI), ADR 0012 (just deploy) |

---

# Contexto

El VPS de producción ($5, 1GB RAM) tiene recursos muy limitados.
Compilar un proyecto Rust con Axum, SQLx y Apalis directamente en producción:

* Consume >2GB RAM → puede matar servicios activos
* Tarda entre 5–15 minutos
* Requiere instalar toolchains y compiladores en producción
* Aumenta superficie de ataque
* Hace deploys lentos e impredecibles

Además:

* SQLite + Litestream requieren reinicios rápidos
* Kamal funciona mejor con imágenes pequeñas
* El deploy debe ser reproducible y determinístico

---

# Decisión

## Política oficial

> 🚫 Está prohibido compilar en producción.

Todo build ocurre:

* localmente
* o en CI

El servidor únicamente:

* descarga imagen
* reemplaza contenedor
* ejecuta healthcheck
* hace rollback si falla

---

# Estrategia

La arquitectura de build queda definida así:

```text
Developer / CI
    ↓
cargo build --release --target musl
    ↓
Imagen distroless ~10MB
    ↓
Registry (GHCR / Docker Hub)
    ↓
Kamal deploy
    ↓
VPS descarga imagen ya compilada
```

---

# Objetivos

## Seguridad

* Sin toolchains en producción
* Sin código fuente en el VPS
* Sin gcc, make, cargo ni rustup
* Imagen mínima → menos CVEs

## Performance

* Arranque ultra-rápido
* Bajo uso de RAM
* Binario estático
* Imagen pequeña

## Operaciones

* Deploy reproducible
* Rollback instantáneo
* Menos tiempo de downtime
* CI determinístico

---

# 1 — Compilación cruzada MUSL

## Target oficial

```bash
rustup target add x86_64-unknown-linux-musl
```

## Build release

```bash
cargo build \
  --release \
  --target x86_64-unknown-linux-musl
```

Resultado:

```text
target/x86_64-unknown-linux-musl/release/api
```

---

# ¿Por qué MUSL?

## Ventajas

| Beneficio                  | Explicación                      |
| -------------------------- | -------------------------------- |
| Binario estático           | No depende de libc del host      |
| Portable                   | Corre en cualquier Linux         |
| Ideal para distroless      | No necesita paquetes del sistema |
| Menos fallos en producción | El entorno deja de importar      |

---

# 2 — Containerfile multi-stage

## Arquitectura oficial

```dockerfile
# infra/docker/Containerfile

# ─────────────────────────────────────────────────────────────
# Stage 1 — Builder
# ─────────────────────────────────────────────────────────────

FROM rust:1.82-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY apps/api/ ./apps/api/
COPY data/ ./data/

ENV SQLX_OFFLINE=true

RUN cargo build \
    --release \
    --target x86_64-unknown-linux-musl \
    --bin api

# ─────────────────────────────────────────────────────────────
# Stage 2 — Runtime mínimo
# ─────────────────────────────────────────────────────────────

FROM gcr.io/distroless/cc-debian12

COPY --from=builder \
  /app/target/x86_64-unknown-linux-musl/release/api /api

COPY --from=builder \
  /app/data/migrations /migrations

COPY --from=ghcr.io/benbjohnson/litestream:latest-amd64 \
  /usr/local/bin/litestream /litestream

COPY infra/litestream/litestream.yml \
  /etc/litestream.yml

EXPOSE 8080

HEALTHCHECK \
  --interval=30s \
  --timeout=5s \
  --start-period=10s \
  --retries=3 \
  CMD ["/api", "health"]

ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]
```

---

# 3 — Runtime Distroless

## Imagen oficial

```dockerfile
FROM gcr.io/distroless/cc-debian12
```

## Distroless significa

* sin bash
* sin apt
* sin curl
* sin package manager
* sin herramientas de debugging
* solo runtime mínimo

---

# Beneficios reales

| Métrica              | Ubuntu    | Distroless |
| -------------------- | --------- | ---------- |
| Tamaño               | ~80–150MB | ~10MB      |
| Superficie de ataque | Alta      | Muy baja   |
| CVEs                 | Muchas    | Mínimas    |
| Tiempo pull          | Más lento | Muy rápido |

---

# 4 — Optimizaciones release

## Configuración oficial

```toml
# Cargo.toml

[profile.release]
opt-level     = "z"
lto           = true
codegen-units = 1
panic         = "abort"
strip         = true
```

---

# Explicación técnica

## opt-level = "z"

Optimiza por tamaño.

Ideal para VPS pequeños.

---

## lto = true

Link Time Optimization.

El linker elimina código muerto entre crates.

Reduce:

* tamaño
* RAM
* tiempo de arranque

---

## codegen-units = 1

Máxima optimización global.

Trade-off:

* build más lento
* binario más eficiente

---

## panic = "abort"

Sin stack unwinding.

Reduce:

* tamaño del binario
* complejidad runtime

---

## strip = true

Elimina:

* símbolos debug
* metadata innecesaria

---

# Resultados esperados

| Métrica      | Sin optimizar | Optimizado |
| ------------ | ------------- | ---------- |
| Binario      | ~25MB         | ~8–12MB    |
| Imagen final | ~120MB        | ~10MB      |
| RAM idle     | ~45MB         | ~30MB      |
| Startup      | ~200ms        | ~80ms      |

---

# 5 — SQLX OFFLINE obligatorio

## Política

La compilación nunca depende de DB activa.

---

# Flujo oficial

## Antes de commit

```bash
just prepare
```

## Comando real

```bash
cargo sqlx prepare --workspace
```

Esto genera:

```text
.sqlx/
```

---

# Regla

> `.sqlx/` se versiona en git.

---

# CI

```bash
just prepare
git diff --exit-code .sqlx/
```

Si `.sqlx/` cambió:

* el CI falla
* el developer olvidó regenerar metadata

---

# 6 — Deploy con Kamal

## Pipeline oficial

```makefile
deploy:
    just audit
    just test
    kamal deploy
```

---

# Qué hace Kamal

```text
1. Build imagen
2. Push registry
3. VPS descarga imagen
4. Healthcheck
5. Swap atómico
6. Rollback automático si falla
```

---

# Política de deploy

## Producción jamás ejecuta:

```bash
cargo build
cargo install
rustup
```

---

# Comparativa de estrategias

| Estrategia         | Tiempo    | RAM VPS | Riesgo |
| ------------------ | --------- | ------- | ------ |
| Build en VPS       | 10–15 min | >2GB    | Alto   |
| SCP binario        | ~30s      | Bajo    | Medio  |
| Kamal + distroless | 2–3 min   | ~0      | Bajo   |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta      | Propósito                               |
| ---------------- | --------------------------------------- |
| `cargo-zigbuild` | Cross-compilation portable usando Zig   |
| `cargo-bloat`    | Detectar qué crates agrandan el binario |
| `upx`            | Compresión extrema del binario          |
| `cargo-dist`     | Releases automatizados                  |
| `sccache`        | Cache distribuido de compilación        |
| `cargo-chef`     | Cache eficiente de Docker layers        |

---

# Reglas Operativas

## Obligatorio

✅ Build fuera del VPS
✅ MUSL target
✅ Distroless runtime
✅ SQLX offline
✅ Imagen <15MB

---

## Prohibido

🚫 `cargo build` en producción
🚫 Ubuntu runtime completo
🚫 Alpine para Rust MUSL crítico
🚫 Código fuente en el VPS
🚫 Toolchains instalados en producción

---

# Consecuencias

## ✅ Positivas

### Deploys rápidos

El VPS solo descarga y reemplaza.

---

### Seguridad real

No existe código fuente ni compilador en producción.

---

### Menor uso de RAM

El deploy no compite con usuarios activos.

---

### Reproducibilidad

Staging y producción usan exactamente el mismo binario.

---

### Escalabilidad futura

El pipeline ya está listo para:

* múltiples VPS
* autoscaling
* builders remotos

---

## ⚠️ Negativas / Trade-offs

### Builds locales más lentos

Causa:

* LTO
* MUSL
* codegen-units=1

Mitigación:

* cache incremental
* sccache
* cargo-chef

---

### `.sqlx/` requiere disciplina

Mitigación:

* pre-push hook
* CI obligatorio

---

### MUSL puede complicar crates nativos

Mitigación:

* preferir crates pure Rust
* evitar bindings C innecesarios

---

# Decisiones derivadas

* `.sqlx/` se guarda en git
* `SQLX_OFFLINE=true` es obligatorio
* El runtime oficial es `distroless`
* El tamaño máximo permitido es ~15MB
* `cargo bloat` se usa si el binario crece inesperadamente
* Kamal es el único mecanismo oficial de deploy
* El VPS jamás contiene el source code del proyecto
