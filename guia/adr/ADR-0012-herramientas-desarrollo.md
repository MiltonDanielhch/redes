# ADR 0012 — Herramientas: just + pnpm + lefthook

| Campo               | Valor                                                                                                    |
| ------------------- | -------------------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                                               |
| **Fecha**           | 2026-05-15                                                                                               |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                         |
| **Relacionado con** | ADR 0010 (Testing), ADR 0011 (Estándares de Desarrollo), ADR 0013 (Deploy), ADR 0016 (OpenAPI) |

---

# Contexto

Sin un estándar de tooling:

* cada developer ejecuta comandos distintos
* se olvidan pasos críticos antes de commit o deploy
* aparecen diferencias entre entornos
* el onboarding toma demasiado tiempo
* los workflows se vuelven manuales e inconsistentes

En un monorepo Rust + TypeScript:

* Makefiles clásicos se vuelven difíciles de mantener
* npm scripts no integran bien herramientas Rust
* git hooks manuales son inconsistentes
* múltiples package managers generan conflictos

El objetivo es tener:

* onboarding en menos de 5 minutos
* comandos reproducibles
* automatización consistente
* calidad obligatoria
* workflows simples para humano + IA

---

# Decisión

Se adopan oficialmente:

| Herramienta | Rol                               |
| ----------- | --------------------------------- |
| `just`      | Task runner universal             |
| `pnpm`      | Gestión de paquetes JS            |
| `lefthook`  | Git hooks rápidos y reproducibles |

---

# 1 — just como task runner oficial

## Razón principal

`just` es:

* más legible que Makefile
* multiplataforma
* simple
* explícito
* ideal para monorepos mixtos Rust + JS

---

## Problemas reales de Makefile

| Problema                 | Impacto               |
| ------------------------ | --------------------- |
| Tabs obligatorios        | errores silenciosos   |
| Sintaxis antigua         | difícil mantenimiento |
| Variables inconsistentes | confusión             |
| Shell behavior extraño   | debugging difícil     |
| Mala UX                  | onboarding lento      |

---

## Ventajas reales de just

| Ventaja              | Beneficio           |
| -------------------- | ------------------- |
| Sintaxis moderna     | legibilidad         |
| Argumentos nombrados | comandos claros     |
| Cross-platform       | Linux/macOS/Windows |
| Excelente DX         | onboarding rápido   |
| Autodocumentado      | `just --list`       |

---

# 2 — Filosofía del tooling

## Un comando = una intención clara

Ejemplo:

```bash id="m4p61j"
just dev
just test
just deploy
just rollback
```

No:

```bash id="7c0c20"
cargo run --bin api --features local
```

---

# 3 — justfile oficial

```makefile
# justfile
# Mostrar comandos:
# just --list

set dotenv-load := true

# ─────────────────────────────────────────────────────────────────────────────
# Setup
# ─────────────────────────────────────────────────────────────────────────────

setup:
    cargo install cargo-watch cargo-nextest cargo-deny cargo-audit sqlx-cli
    cargo install bacon
    cargo install cargo-edit
    cargo install typos-cli
    cargo install just
    cargo install lefthook

    npm install -g pnpm

    pnpm install

    cp -n .env.example .env.local || true

    lefthook install

    sqlx database create
    just migrate

    @echo "✅ Setup completo"
    @echo "➡ Edita .env.local"
    @echo "➡ Ejecuta: just dev"

# ─────────────────────────────────────────────────────────────────────────────
# Desarrollo
# ─────────────────────────────────────────────────────────────────────────────

dev:
    cargo watch -x "run --bin api" & pnpm --filter web dev

dev-api:
    cargo watch -x "run --bin api"

dev-web:
    pnpm --filter web dev

watch:
    bacon

# ─────────────────────────────────────────────────────────────────────────────
# Calidad
# ─────────────────────────────────────────────────────────────────────────────

fmt:
    cargo fmt --all
    pnpm --filter web format

lint:
    cargo clippy --all-targets -- -D warnings
    pnpm --filter web lint

check:
    cargo check --workspace

audit:
    cargo deny check
    cargo audit

typos:
    typos .

quality:
    just fmt
    just lint
    just audit
    just typos

# ─────────────────────────────────────────────────────────────────────────────
# Testing
# ─────────────────────────────────────────────────────────────────────────────

test:
    cargo nextest run

test-all:
    cargo nextest run --all-targets

test-v:
    cargo nextest run --no-capture

coverage:
    cargo llvm-cov nextest --html

# ─────────────────────────────────────────────────────────────────────────────
# Base de datos
# ─────────────────────────────────────────────────────────────────────────────

migrate:
    sqlx migrate run

migrate-reset:
    sqlx database reset

migrate-new name:
    sqlx migrate add {{name}}

db-status:
    sqlx migrate info

prepare:
    cargo sqlx prepare --workspace

# ─────────────────────────────────────────────────────────────────────────────
# Tipos TypeScript
# ─────────────────────────────────────────────────────────────────────────────

types:
    buf generate

types-check:
    buf generate
    git diff --exit-code apps/web/src/lib/types/api.ts

# ─────────────────────────────────────────────────────────────────────────────
# Build
# ─────────────────────────────────────────────────────────────────────────────

build:
    cargo build --release
    pnpm --filter web build

# ─────────────────────────────────────────────────────────────────────────────
# Deploy
# ─────────────────────────────────────────────────────────────────────────────

deploy:
    just quality
    just test
    kamal deploy

rollback:
    kamal rollback

redeploy:
    kamal redeploy

logs:
    kamal logs -f

status:
    kamal details

# ─────────────────────────────────────────────────────────────────────────────
# Utilidades
# ─────────────────────────────────────────────────────────────────────────────

clean:
    cargo clean
    pnpm store prune
```

---

# 4 — pnpm como package manager oficial

## Razón

`pnpm`:

* usa menos disco
* es más rápido
* tiene workspaces reales
* evita duplicación masiva de node_modules

---

# Workspace oficial

```yaml
# pnpm-workspace.yaml
packages:
  - "apps/web"
  - "apps/mailer"
```

---

# Beneficios

| Beneficio          | Resultado            |
| ------------------ | -------------------- |
| hardlinks          | menos espacio        |
| instalación rápida | CI más rápido        |
| workspaces reales  | monorepo limpio      |
| lockfile único     | builds reproducibles |

---

# 5 — lefthook como enforcement local

## Filosofía

La calidad no depende de memoria humana.

Los hooks garantizan:

* formato correcto
* compilación válida
* lint limpio
* tests mínimos

ANTES de llegar al repositorio.

---

# Configuración oficial

```yaml
# lefthook.yml

pre-commit:
  parallel: true
  commands:

    fmt-rust:
      glob: "*.rs"
      run: cargo fmt --all --check

    check:
      run: cargo check --workspace

    typos:
      run: typos .

pre-push:
  commands:

    lint:
      run: cargo clippy --all-targets -- -D warnings

    test:
      run: cargo nextest run

    audit:
      run: cargo deny check
```

---

# Instalación

```bash id="icd7gk"
lefthook install
```

Incluido automáticamente en:

```bash id="u0lzlf"
just setup
```

---

# 6 — Filosofía de onboarding

Un developer nuevo debe poder hacer:

```bash id="8xxmbj"
git clone ...
cd proyecto
just setup
just dev
```

sin leer documentación extensa.

---

# Resultado esperado

En menos de 5 minutos:

* entorno listo
* DB creada
* migraciones aplicadas
* hooks instalados
* frontend funcionando
* backend funcionando

---

# 7 — Integración con IA

El tooling está diseñado para:

* predictibilidad
* automatización
* repetibilidad
* comandos explícitos

---

## Beneficio para IA

Una IA puede inferir workflows fácilmente:

```bash id="uf7mzd"
just test
just lint
just deploy
```

sin conocimiento humano adicional.

---

# 8 — Política de CI

El CI debe ejecutar exactamente los mismos comandos locales.

---

## Regla

Nunca tener:

* comandos “especiales” del CI
* pasos distintos entre local y GitHub Actions

---

## Principio

> “Local = CI = Producción”

---

# Alternativas consideradas

| Herramienta         | Motivo de descarte        |
| ------------------- | ------------------------- |
| Makefile            | sintaxis antigua y frágil |
| npm scripts         | limitado a JS             |
| Taskfile            | dependencia adicional     |
| husky               | muy centrado en Node      |
| pre-commit (Python) | stack extra innecesario   |

---

# Herramientas y Librerías Recomendadas (Edición 2026)

| Herramienta      | Propósito              |
| ---------------- | ---------------------- |
| `mise`           | gestión de toolchains  |
| `bacon`          | feedback loop          |
| `cargo-nextest`  | tests paralelos        |
| `cargo-deny`     | auditoría supply-chain |
| `cargo-audit`    | CVEs                   |
| `typos`          | calidad textual        |
| `cargo-dist`     | releases               |
| `cargo-llvm-cov` | cobertura              |
| `sqlx-cli`       | migraciones            |
| `buf`            | generación tipos       |

---

# Consecuencias

## ✅ Positivas

* Onboarding extremadamente rápido
* Comandos consistentes
* Calidad automatizada
* Menos errores humanos
* Workflows reproducibles
* Mejor colaboración humano + IA
* CI más simple

---

## ⚠️ Negativas / Trade-offs

### Más herramientas instaladas

Requiere:

* Rust toolchain
* Node
* pnpm
* just
* lefthook

Mitigación:

```bash id="ey5d4j"
just setup
```

automatiza todo.

---

### Hooks pueden sentirse lentos

Mitigación:

* `nextest`
* paralelismo
* capas rápidas de tests

---

### Developers pueden saltarse hooks

```bash id="p9c4hz"
LEFTHOOK=0 git push
```

Mitigación:

* CI vuelve a validar todo
* bypass queda visible en historial

---

# Decisiones derivadas

* `just` es el entrypoint oficial del proyecto
* Todo workflow recurrente debe vivir en `justfile`
* `pnpm` es obligatorio — no usar npm/yarn
* `lefthook install` forma parte de `just setup`
* `just deploy` ejecuta validaciones antes de desplegar
* El CI ejecuta exactamente los mismos comandos locales
* El tooling prioriza simplicidad operacional sobre flexibilidad excesiva
