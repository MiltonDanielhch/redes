# ADR 0013 — Infraestructura: Docker Compose + Distroless + Red Privada

| Campo               | Valor                                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                       |
| **Fecha**           | 2026-05-16                                                                                       |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                 |
| **Relacionado con** | ADR 0012 (Tooling), ADR 0003 (Backend Axum), ADR 0014 (Monitoreo), ADR 0019 (Coolify) |
| **Última revisión** | 2026-05-16 — Agregado Dockerfile multi-stage + cargo-chef + distroless |

---

## Contexto

El sistema de monitoreo de red de la Gobernación del Beni necesita:

* despliegue simple
* mantenimiento sencillo
* aislamiento entre servicios
* facilidad de debugging
* reproducibilidad
* bajo consumo de recursos

La infraestructura objetivo:

* VPS o servidor Linux pequeño/mediano
* equipo técnico reducido
* presupuesto limitado
* despliegue rápido y estable

Kubernetes, Nomad o Swarm introducen complejidad innecesaria para esta fase del proyecto.

---

## Decisión

Usar **Docker Compose** como orquestador oficial de infraestructura.

La plataforma se ejecuta mediante:

* contenedores separados
* red privada interna
* volúmenes persistentes
* variables de entorno
* comunicación interna por DNS de Docker
* imágenes distroless para runtime (~10MB)

---

## Arquitectura General

```text
┌──────────────────────────────────────────────────────┐
│              red privada interna                    │
│                  172.20.0.0/16                      │
├──────────────────────────────────────────────────────┤
│                                                      │
│   ┌─────────────────┐                               │
│   │    Frontend     │                               │
│   │   SvelteKit     │                               │
│   │     :3000       │                               │
│   └────────┬────────┘                               │
│            │ HTTP                                   │
│            ▼                                        │
│   ┌─────────────────┐                               │
│   │     Backend     │                               │
│   │   Rust + Axum   │                               │
│   │     :8080       │                               │
│   └────────┬────────┘                               │
│            │ SQL                                    │
│            ▼                                        │
│   ┌─────────────────┐                               │
│   │   PostgreSQL    │                               │
│   │    17-alpine    │                               │
│   │       :5432     │                               │
│   └─────────────────┘                               │
│                                                      │
└──────────────────────────────────────────────────────┘
```

---

## Estructura de archivos

```text
infra/
├── docker/
│   ├── backend.Dockerfile      # Multi-stage Rust + cargo-chef + distroless
│   ├── frontend.Dockerfile     # SvelteKit + Node 24
│   ├── .dockerignore           # Exclusiones de contexto
│   └── docker-compose.yml      # Orquestación oficial
│
└── coolify/
    └── docker-compose.yml      # Override para Coolify deploy
```

---

## 1 — Backend Dockerfile (Multi-Stage + cargo-chef + Distroless)

```dockerfile
# Ubicación: `infra/docker/backend.Dockerfile`
#
# Descripción: Multi-stage build para backend Rust. Usa cargo-chef para cachear
#              dependencias y distroless/static como runtime final (~10MB).
#
# ADRs: 0013, 0003, 0019

# ─────────────────────────────────────────────────────────────────────────────
# Stage 1: Planner — genera recipe.json de dependencias
# ─────────────────────────────────────────────────────────────────────────────
FROM lukemathwalker/cargo-chef:latest-rust-1.86 AS chef
WORKDIR /app

# ─────────────────────────────────────────────────────────────────────────────
# Stage 2: Planner — computa recipe.json
# ─────────────────────────────────────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ─────────────────────────────────────────────────────────────────────────────
# Stage 3: Builder — compila dependencias + aplicación
# ─────────────────────────────────────────────────────────────────────────────
FROM chef AS builder

# Instalar target musl para static linking
RUN rustup target add x86_64-unknown-linux-musl

# Copiar recipe del planner
COPY --from=planner /app/recipe.json recipe.json

# Cachear dependencias (esta capa se cachea si recipe.json no cambia)
RUN cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

# Copiar código fuente y compilar
COPY . .
RUN cargo build --release --bin api --target x86_64-unknown-linux-musl

# ─────────────────────────────────────────────────────────────────────────────
# Stage 4: Runtime — distroless/static con nonroot
# ─────────────────────────────────────────────────────────────────────────────
FROM gcr.io/distroless/static-debian12:nonroot

# CA certs y tzdata incluidos en distroless/static
WORKDIR /app

# Copiar binario estático
COPY --from=builder --chown=nonroot:nonroot /app/target/x86_64-unknown-linux-musl/release/api /app/api

# Puerto expuesto (documentación, no binding)
EXPOSE 8080

# Healthcheck interno (requiere endpoint /health en Axum)
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3     CMD ["/app/api", "--healthcheck"] || exit 1

USER nonroot:nonroot

ENTRYPOINT ["/app/api"]
```

**Tamaño esperado:** ~10-15MB (vs ~1.6GB con imagen Rust completa)

---

## 2 — Frontend Dockerfile

```dockerfile
# Ubicación: `infra/docker/frontend.Dockerfile`
#
# Descripción: Build de SvelteKit SSR con adapter-node. Runtime en Node 24.
#
# ADRs: 0013, 0017

# ─────────────────────────────────────────────────────────────────────────────
# Stage 1: Builder — build de SvelteKit
# ─────────────────────────────────────────────────────────────────────────────
FROM node:24-alpine AS builder
WORKDIR /app

# Instalar pnpm
RUN npm install -g pnpm@11.0.0

# Copiar dependencias primero (cache de layer)
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml ./
COPY apps/web/package.json ./apps/web/

RUN pnpm install --frozen-lockfile

# Copiar código y build
COPY . .
RUN pnpm --filter web build

# ─────────────────────────────────────────────────────────────────────────────
# Stage 2: Runtime — Node 24 alpine
# ─────────────────────────────────────────────────────────────────────────────
FROM node:24-alpine AS runtime
WORKDIR /app

# Crear usuario no-root
RUN addgroup -g 1001 -S nodejs &&     adduser -S sveltekit -u 1001

# Copiar build output
COPY --from=builder --chown=sveltekit:nodejs /app/apps/web/build ./build
COPY --from=builder --chown=sveltekit:nodejs /app/apps/web/package.json ./
COPY --from=builder --chown=sveltekit:nodejs /app/node_modules ./node_modules

USER sveltekit

EXPOSE 3000

ENV NODE_ENV=production
ENV PORT=3000
ENV HOST=0.0.0.0

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3     CMD node -e "require('http').get('http://localhost:3000/health', (r) => r.statusCode === 200 ? process.exit(0) : process.exit(1))"

CMD ["node", "build"]
```

---

## 3 — .dockerignore

```text
# Ubicación: `infra/docker/.dockerignore`
#
# Descripción: Exclusiones del contexto de build Docker.
#
# ADRs: 0013

# Rust
/target/
**/*.rs.bk
Cargo.lock.bak
.cargo/

# Node
node_modules/
.pnpm-store/
dist/
build/
coverage/

# Git
.git/
.gitignore
.gitattributes

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Env
.env
.env.local
.env.*.local

# Docs
*.md
!README.md
docs/
guia/

# SQLx
.sqlx/

# Test
*.test.*
tests/
__tests__/

# CI
.github/
.gitlab-ci.yml

# Docker
Dockerfile*
docker-compose*.yml
.dockerignore
```

---

## 4 — Docker Compose Oficial

```yaml
# Ubicación: `infra/docker/docker-compose.yml`
#
# Descripción: Orquestación oficial del sistema. PostgreSQL 17, backend distroless,
#              frontend Node. Red privada, healthchecks, restart policies.
#
# ADRs: 0013, 0019

version: "3.9"

services:
  postgres:
    image: postgres:17-alpine
    container_name: redes-postgres
    restart: always
    stop_grace_period: 30s

    environment:
      POSTGRES_DB: ${POSTGRES_DB:-redes}
      POSTGRES_USER: ${POSTGRES_USER:-redes}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-changeme}

    volumes:
      - postgres_data:/var/lib/postgresql/data

    expose:
      - "5432"

    networks:
      - red-interna

    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-redes} -d ${POSTGRES_DB:-redes}"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s

  backend:
    build:
      context: ../..           # Desde infra/docker/ subir 2 niveles a root
      dockerfile: infra/docker/backend.Dockerfile
    image: redes-backend:latest
    container_name: redes-backend
    restart: unless-stopped

    env_file:
      - ../../.env              # Variables desde root

    ports:
      - "8080:8080"

    depends_on:
      postgres:
        condition: service_healthy

    networks:
      - red-interna

    healthcheck:
      test: ["CMD", "/app/api", "--healthcheck"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

  frontend:
    build:
      context: ../..
      dockerfile: infra/docker/frontend.Dockerfile
    image: redes-frontend:latest
    container_name: redes-frontend
    restart: unless-stopped

    ports:
      - "3000:3000"

    depends_on:
      backend:
        condition: service_healthy

    networks:
      - red-interna

    healthcheck:
      test: ["CMD-SHELL", "node -e "require('http').get('http://localhost:3000/health', (r) => r.statusCode === 200 ? process.exit(0) : process.exit(1))""]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

volumes:
  postgres_data:

networks:
  red-interna:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

---

## 5 — Docker Compose Override (Desarrollo)

```yaml
# Ubicación: `infra/docker/docker-compose.override.yml`
#
# Descripción: Override para desarrollo local. Expone DB, monta volúmenes,
#              desactiva healthchecks agresivos.
#
# ADRs: 0013

services:
  postgres:
    ports:
      - "5432:5432"           # Expuesto para debugging local

  backend:
    build:
      target: builder          # Usar stage builder para hot-reload
    volumes:
      - ../../:/app            # Montar código fuente
    environment:
      RUST_LOG: debug
      RUST_BACKTRACE: 1

  frontend:
    volumes:
      - ../../apps/web:/app/apps/web
    environment:
      NODE_ENV: development
```

---

## Build de imágenes

```bash
# Desde root del proyecto
docker compose -f infra/docker/docker-compose.yml build

# O individualmente
docker build -f infra/docker/backend.Dockerfile -t redes-backend:latest .
docker build -f infra/docker/frontend.Dockerfile -t redes-frontend:latest .
```

---

## Comandos Operativos Oficiales

```bash
# Levantar infraestructura
docker compose -f infra/docker/docker-compose.yml up -d

# Ver logs
docker compose -f infra/docker/docker-compose.yml logs -f

# Detener
docker compose -f infra/docker/docker-compose.yml down

# Reiniciar servicio específico
docker compose -f infra/docker/docker-compose.yml restart backend

# Ver estado
docker compose -f infra/docker/docker-compose.yml ps

# Build con cache de cargo-chef (rápido si deps no cambian)
docker compose -f infra/docker/docker-compose.yml build --no-cache backend
```

---

## Política de Persistencia

### PostgreSQL

La data vive en:

```text
postgres_data
```

Aunque el contenedor muera:

* la DB sobrevive
* reinicio seguro
* upgrades simples

---

## Seguridad

### Red privada

PostgreSQL no expone puertos públicos.

### Aislamiento

Cada servicio:

* filesystem separado
* procesos separados
* dependencias aisladas

### Superficie mínima

* `postgres:17-alpine` — minimal
* `gcr.io/distroless/static-debian12:nonroot` — sin shell, sin root
* imágenes específicas
* sin herramientas innecesarias

### Non-root

* Backend: usuario `nonroot` (distroless)
* Frontend: usuario `sveltekit` (uid 1001)
* PostgreSQL: usuario `postgres` (imagen oficial)

---

## Integración con Coolify (ADR 0019)

Coolify gestiona el deploy leyendo el `docker-compose.yml` del repositorio.

Configuración en Coolify:

1. **Source**: Git repository → `infra/docker/docker-compose.yml`
2. **Build**: Automático via Docker Compose
3. **Environment**: Variables desde `.env` (Coolify UI)
4. **Volumes**: `postgres_data` persistente (Coolify managed)
5. **Domains**: Coolify configura reverse proxy (Caddy) automáticamente

**Nota:** Coolify no necesita `docker-compose.override.yml`. El override es solo para desarrollo local.

---

## Alternativas Consideradas

| Opción | Motivo de descarte |
| ------------ | --------------------------------------- |
| Kubernetes | Complejidad excesiva para esta fase |
| Docker Swarm | Ecosistema reducido |
| Nomad | Overkill para MVP |
| Systemd puro | Sin aislamiento ni networking integrado |
| PM2 | Solo Node.js |

---

## Herramientas y Librerías Recomendadas (Edición 2026)

| Herramienta | Propósito |
| -------------- | --------------------------------- |
| `lazydocker` | TUI para administrar contenedores |
| `ctop` | Monitoreo live de recursos Docker |
| `watchtower` | Auto-update de imágenes |
| `dive` | Analizar tamaño de imágenes |
| `docker scout` | Escaneo de vulnerabilidades |
| `cargo-chef` | Cachear dependencias Rust en Docker |
| `distroless` | Imágenes mínimas sin shell |

---

## Consecuencias

### ✅ Positivas

* Infraestructura reproducible
* Operación simple (`docker compose up -d`)
* Aislamiento real
* Fácil debugging
* Bajo costo operacional
* Imágenes ultra-pequeñas (~10MB backend)
* Builds rápidos con cargo-chef cache
* Non-root por defecto

### ⚠️ Negativas / Trade-offs

**No hay alta disponibilidad horizontal**

Mitigación futura:
* Kubernetes
* Nomad
* Docker Swarm

**Compose no es autoscaling**

Aceptable para:
* MVP
* staging
* producción pequeña/media

**PostgreSQL vive en un solo nodo**

Mitigación futura:
* backups automáticos
* replicas
* WAL shipping

**Distroless dificulta debugging**

Mitigación:
* Logs estructurados (tracing)
* Healthchecks
* `docker run --entrypoint=/busybox/sh gcr.io/distroless/static:debug` para emergencias

---

## Decisiones derivadas

* Toda comunicación interna usa red privada Docker
* PostgreSQL 17-alpine es la base oficial
* PostgreSQL nunca expone puertos públicos
* El deployment oficial usa Docker Compose
* Los contenedores reinician automáticamente
* La persistencia se maneja con volúmenes Docker
* El backend depende explícitamente de PostgreSQL (condition: healthy)
* El frontend depende explícitamente del backend (condition: healthy)
* Backend: multi-stage build con cargo-chef + distroless/static:nonroot
* Frontend: multi-stage build con Node 24 alpine
* Target de build: `x86_64-unknown-linux-musl` (static linking)
* `.dockerignore` obligatorio en `infra/docker/`
* `docker-compose.override.yml` para desarrollo local
* Coolify lee `infra/docker/docker-compose.yml` para deploy
