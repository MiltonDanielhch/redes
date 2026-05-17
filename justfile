# Monitoreo de Infraestructura Regional - Commands
# Using just 1.51+
#
# Ubicación: `justfile`
#
# Descripción: Comandos de desarrollo, build, test y deploy.
#
# ADRs relacionados: 0012 (Herramientas)

# Development
dev:
    @echo "Starting development environment..."
    pnpm dev

dev-api:
    @echo "Starting API server..."
    cd apps/api && cargo watch -x check -x run

dev-web:
    @echo "Starting web dev server..."
    cd apps/web && pnpm dev

# Build
build:
    cargo build --release

build-agent:
    cargo build --release --bin agent --profile release-size

# Testing
test:
    cargo nextest run

test-watch:
    cargo nextest run --watch

# Linting & Formatting
fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace -- -D warnings

check:
    cargo check --workspace

# Code Quality
audit:
    cargo deny check
    cargo audit

# Database
migrate:
    sqlx migrate run

migrate-reset:
    sqlx migrate reset

db-status:
    sqlx migrate status

prepare:
    cargo sqlx prepare --workspace

# Utilities
doctor:
    @echo "Checking toolchain..."
    rustc --version
    cargo --version
    @echo "Checking Node..."
    node --version
    pnpm --version
    @echo "Checking just..."
    just --version

setup:
    @echo "Installing dependencies..."
    cargo build --workspace
    cd apps/web && pnpm install
    lefthook install

# Deployment
deploy:
    @echo "Deploy not configured yet"

# Clean
clean:
    cargo clean
    cd apps/web && rm -rf node_modules .svelte-kit