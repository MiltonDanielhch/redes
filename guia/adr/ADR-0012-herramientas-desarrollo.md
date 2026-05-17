# ADR 0012 — Herramientas: mise + just + pnpm + lefthook

> **Última revisión de versiones:** 2026-05-16  
> Se actualizaron las versiones de herramientas tras auditoría contra crates.io, GitHub, nodejs.org y docs.rs.

| Campo               | Valor                                                                                                    |
| ------------------- | -------------------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado                                                                                               |
| **Fecha**           | 2026-05-16                                                                                               |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                         |
| **Relacionado con** | ADR 0010 (Testing), ADR 0011 (Estándares de Desarrollo), ADR 0013 (Deploy), ADR 0016 (OpenAPI), ADR 0019 (Coolify) |
| **Última revisión** | 2026-05-16 — Alineación con mise 2026.5 + pnpm 10 + Node 26 + Coolify |

---

## Contexto

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

## Decisión

Se adopan oficialmente:

| Herramienta | Rol | Versión mínima |
| ----------- | --------------------------------- | --------------- |
| `mise` | Gestor de toolchains (Rust, Node, pnpm, just) | 2026.5 |
| `just` | Task runner universal | 1.51.0 |
| `pnpm` | Gestión de paquetes JS | 10.27 |
| `lefthook` | Git hooks rápidos y reproducibles | 2.1.6 |

---

## 0 — mise: gestor de toolchains

`mise` (anteriormente `rtx`) es el gestor de versiones oficial del proyecto. Centraliza:

- Rust toolchain
- Node.js
- pnpm
- just

> **Nota:** mise usa versionado por fecha (`v2026.5.9` publicado 15 may 2026). Se recomienda usar `mise use mise@latest` o seguir la serie `2026.5.x`.

### Configuración (`mise.toml`)

```toml
[tools]
rust = "1.95"
node = "26"
pnpm = "10"
just = "1.51"

[env]
RUST_LOG = "info"
DATABASE_URL = "postgres://user:pass@localhost:5432/redes"
```

### Instalación

```bash
# macOS/Linux
curl https://mise.run | sh

# Activar shell
echo 'eval "$(~/.local/bin/mise activate)"' >> ~/.bashrc
```

---

## 1 — just como task runner oficial

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

## 2 — Filosofía del tooling

## Un comando = una intención clara

Ejemplo:

```bash
just dev
just test
just deploy
just rollback
```

No:

```bash
cargo run --bin api --features local
```

---

## 3 — justfile oficial

```makefile
# justfile
# Mostrar comandos: just --list

set dotenv-load := true

# ─────────────────────────────────────────────────────────────────────────────
# Setup
# ─────────────────────────────────────────────────────────────────────────────

setup:
    # Verificar mise está activo
    mise doctor

    # Instalar dependencias Rust
    cargo install cargo-nextest cargo-deny cargo-audit sqlx-cli --locked
    cargo install bacon --locked
    cargo install typos-cli --locked

    # Instalar lefthook (no via cargo — usa el binario oficial)
    npm install -g lefthook@2.1.6

    # Instalar dependencias JS
    pnpm install

    # Configurar entorno
    cp -n .env.example .env.local || true
    lefthook install

    # Base de datos
    sqlx database create
    just migrate

    @echo "✅ Setup completo"
    @echo "➡ Edita .env.local"
    @echo "➡ Ejecuta: just dev"

# ─────────────────────────────────────────────────────────────────────────────
# Desarrollo
# ─────────────────────────────────────────────────────────────────────────────

dev:
    # Backend + Frontend en paralelo
    bacon & pnpm --filter web dev

dev-api:
    bacon

dev-web:
    pnpm --filter web dev

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

quality:
    just fmt
    just lint
    just audit
    just typos

# ─────────────────────────────────────────────────────────────────────────────
# Testing
# ─────────────────────────────────────────────────────────────────────────────

test:
    cargo nextest run --profile default -E 'not test(e2e)'

test-e2e:
    cargo nextest run --profile e2e -E 'test(e2e)'

test-all:
    cargo nextest run --profile ci

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
# Tipos TypeScript (OpenAPI → TS)
# ─────────────────────────────────────────────────────────────────────────────

types:
    # Generar tipos desde OpenAPI del backend corriendo
    # Requiere backend en localhost:8080
    curl -s http://localhost:8080/openapi.json > /tmp/openapi.json
    openapi-typescript /tmp/openapi.json --output apps/web/src/lib/generated/api-types.ts

types-check:
    just types
    git diff --exit-code apps/web/src/lib/generated/api-types.ts

# ─────────────────────────────────────────────────────────────────────────────
# Build
# ─────────────────────────────────────────────────────────────────────────────

build:
    cargo build --release
    pnpm --filter web build

build-agent:
    cargo build --release --bin agent --profile release-size

# ─────────────────────────────────────────────────────────────────────────────
# Deploy (Coolify)
# ─────────────────────────────────────────────────────────────────────────────

deploy:
    just quality
    just test-all
    just build
    # Push a Coolify (vía webhook o CLI)
    @echo "🚀 Deploy a Coolify — usar dashboard o webhook"

logs:
    @echo "Ver logs en Coolify dashboard"

status:
    @echo "Ver estado en Coolify dashboard"

# ─────────────────────────────────────────────────────────────────────────────
# Utilidades
# ─────────────────────────────────────────────────────────────────────────────

clean:
    cargo clean
    pnpm store prune
```

---

## 4 — pnpm como package manager oficial

## Razón

`pnpm`:

* usa menos disco (hardlinks)
* es más rápido
* tiene workspaces reales
* evita duplicación masiva de node_modules
* supply-chain protection por defecto (pnpm 10)

> **Nota:** pnpm 10.27.0 es la última estable (diciembre 2025). Node.js 26 (mayo 2026) es la versión Current. Node.js 24 (Krypton) es la LTS estable recomendada para producción.

---

## Workspace oficial

```yaml
# pnpm-workspace.yaml
packages:
  - "apps/web"

# pnpm 10: configuración en pnpm-workspace.yaml, no .npmrc
```

---

## package.json (apps/web)

```json
{
  "name": "@redes/web",
  "packageManager": "pnpm@10.27.0",
  "engines": {
    "node": ">=26.0.0"
  }
}
```

---

## Beneficios

| Beneficio | Resultado |
| ------------------ | -------------------- |
| hardlinks | menos espacio |
| instalación rápida | CI más rápido |
| workspaces reales | monorepo limpio |
| lockfile único | builds reproducibles |
| isolated globals | sin conflictos entre paquetes globales |

---

## 5 — lefthook como enforcement local

## Filosofía

La calidad no depende de memoria humana.

Los hooks garantizan:

* formato correcto
* compilación válida
* lint limpio
* tests mínimos
* seguridad (cargo-deny)

ANTES de llegar al repositorio.

---

## Configuración oficial

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
      run: cargo nextest run --profile default -E 'not test(e2e)'

    audit:
      run: cargo deny check
```

---

## Instalación

```bash
lefthook install
```

Incluido automáticamente en:

```bash
just setup
```

---

## 6 — Filosofía de onboarding

Un developer nuevo debe poder hacer:

```bash
git clone ...
cd proyecto
mise install      # instala Rust, Node, pnpm, just
just setup
just dev
```

sin leer documentación extensa.

---

## Resultado esperado

En menos de 5 minutos:

* entorno listo (via mise)
* DB creada
* migraciones aplicadas
* hooks instalados
* frontend funcionando
* backend funcionando

---

## 7 — Integración con IA

El tooling está diseñado para:

* predictibilidad
* automatización
* repetibilidad
* comandos explícitos

---

## Beneficio para IA

Una IA puede inferir workflows fácilmente:

```bash
just test
just lint
just deploy
```

sin conocimiento humano adicional.

---

## 8 — Política de CI

El CI debe ejecutar exactamente los mismos comandos locales.

---

## Regla

Nunca tener:

* comandos "especiales" del CI
* pasos distintos entre local y GitHub Actions

---

## Principio

> "Local = CI = Producción"

---

## Alternativas consideradas

| Herramienta | Motivo de descarte |
| ------------------- | ------------------------- |
| Makefile | sintaxis antigua y frágil |
| npm scripts | limitado a JS |
| Taskfile | dependencia adicional |
| husky | muy centrado en Node |
| pre-commit (Python) | stack extra innecesario |
| asdf / nvm | reemplazados por mise |

---

## Herramientas y Librerías Recomendadas (Edición 2026)

| Herramienta | Propósito | Versión | Estado |
| ---------------- | ---------------------- | -------- | ------ |
| `mise` | gestión de toolchains | 2026.5 | ✅ Activa |
| `just` | task runner | 1.51.0 | ✅ Activa |
| `pnpm` | package manager JS | 10.27 | ✅ Activa |
| `lefthook` | git hooks | 2.1.6 | ✅ Activa |
| `bacon` | feedback loop | 3.22.0 | ✅ Activa |
| `cargo-nextest` | tests paralelos | 0.9.135 | ✅ Activa |
| `cargo-deny` | auditoría supply-chain | 0.19.6 | ✅ Activa |
| `cargo-audit` | CVEs | 0.22.1 | ✅ Activa |
| `typos` | calidad textual | 1.46.2 | ✅ Activa |
| `sqlx-cli` | migraciones | 0.8.6 | ✅ Activa |
| `openapi-typescript` | tipos TS desde OpenAPI | latest | ✅ Activa |

---

## Consecuencias

### ✅ Positivas

* Onboarding extremadamente rápido (mise + just)
* Comandos consistentes
* Calidad automatizada
* Menos errores humanos
* Workflows reproducibles
* Mejor colaboración humano + IA
* CI más simple

### ⚠️ Negativas / Trade-offs

**Más herramientas instaladas**

Requiere:
* mise (Rust, Node, pnpm, just)
* lefthook

Mitigación:
```bash
just setup
```
automatiza todo.

**Hooks pueden sentirse lentos**

Mitigación:
* `nextest`
* paralelismo
* capas rápidas de tests

**Developers pueden saltarse hooks**

```bash
LEFTHOOK=0 git push
```

Mitigación:
* CI vuelve a validar todo
* bypass queda visible en historial

---

## Decisiones derivadas

* `mise` es el gestor de toolchains oficial (reemplaza asdf/nvm)
* `just` es el entrypoint oficial del proyecto
* Todo workflow recurrente debe vivir en `justfile`
* `pnpm` 10.27 es obligatorio — no usar npm/yarn/pnpm 9
* `lefthook install` forma parte de `just setup`
* `just deploy` ejecuta validaciones antes de desplegar (Coolify)
* El CI ejecuta exactamente los mismos comandos locales
* El tooling prioriza simplicidad operacional sobre flexibilidad excesiva
* `cargo-edit` eliminado (no se usa en el proyecto)
* `cargo-watch` eliminado (reemplazado por `bacon`)

---

## Registro de cambios de versiones

| Fecha | Componente | Anterior | Actual | Notas |
|-------|------------|----------|--------|-------|
| 2026-05-16 | mise | 2026.x (genérico) | **2026.5** | Última: v2026.5.9 (15 may). Versionado por fecha. |
| 2026-05-16 | just | 1.40 | **1.51.0** | Nuevas funciones: módulos, `[no-cd]`, path functions |
| 2026-05-16 | pnpm | 11.0 | **10.27** | Última estable (dic 2025). Node.js 26 es Current; 24 es LTS. |
| 2026-05-16 | Node.js | 24 | **26** | Node 26 Current (may 2026). Node 24 LTS estable para prod. |
| 2026-05-16 | lefthook | 2.1.6 | **2.1.6** | Sin cambios. Versión vigente. |
| 2026-05-16 | bacon | 3.22.0 | **3.22.0** | Sin cambios. Versión vigente. |
| 2026-05-16 | cargo-nextest | 0.9 | **0.9.135** | Runner de tests actualizado |
| 2026-05-16 | cargo-deny | 0.18 | **0.19.6** | Fix de segfault (0.19.5), advisory parsing (0.19.4) |
| 2026-05-16 | cargo-audit | 0.21 | **0.22.1** | Última estable (feb 2026). MSRV 1.85.0 |
| 2026-05-16 | typos | 1.46.1 | **1.46.2** | Última estable (16 may 2026) |
| 2026-05-16 | sqlx-cli | 0.8 | **0.8.6** | Patch release con fixes |
