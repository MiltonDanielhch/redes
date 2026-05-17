# Monitoreo de Infraestructura Regional - Gobernación del Beni

Sistema centralizado de monitoreo de infraestructura de red institucional para la Gobernación del Beni, Bolivia.

---

## 🎯 Objetivo

Monitorear la infraestructura de red de las sedes institucionales del Beni:
- Detectar fallas rápidamente
- Identificar dispositivos no autorizados
- Analizar consumo de ancho de banda
- Visualizar topología de red
- Generar auditorías y reportes técnicos

---

## 🏗️ Arquitectura

```
Monitoreo de Infraestructura Regional
├── Backend:    Rust + Axum + SQLx + PostgreSQL
├── Frontend:   SvelteKit + Svelte 5 + TypeScript
├── Auth:       PASETO v4 (nunca JWT)
├── Jobs:       Apalis (async processing)
├── Deploy:     Coolify
└── Monitoreo:  Healthchecks.io
```

### Stack Tecnológico

| Componente | Tecnología | ADR |
|------------|------------|-----|
| Backend | Rust 2024 + Axum 0.8 | ADR 0003 |
| Frontend | SvelteKit + Svelte 5 | ADR 0017 |
| DB | PostgreSQL | ADR 0004 |
| Deploy | Coolify | ADR 0019 |
| Auth | PASETO v4 Local | ADR 0008 |
| Jobs | Apalis | ADR 0015 |
| API Docs | OpenAPI + Utoipa | ADR 0016 |
| Monitoreo | Healthchecks.io | ADR 0014 |

---

## 📁 Estructura del Proyecto

```
├── crates/                  # Crates de Rust (arquitectura hexagonal)
│   ├── domain/            # Entidades de negocio (sin deps externas)
│   ├── application/        # Casos de uso
│   ├── database/          # Repositorios SQLx
│   ├── auth/              # PASETO + argon2
│   ├── infrastructure/    # Axum + config + utoipa
│   └── ...
├── apps/                   # Aplicaciones
│   ├── api/               # API REST Axum
│   ├── web/               # Dashboard SvelteKit
│   └── agent/             # Agente de monitoreo
├── guia/                   # Documentación
│   ├── adr/               # Architecture Decision Records
│   ├── roadmap/           # Roadmaps por fase
│   └── PROMPT_MAESTRO.md  # Contexto del proyecto
└── data/                  # Datos
    └── migrations/        # Migraciones PostgreSQL
```

---

## 🚀 Primeros Pasos

### Requisitos

- Rust 2024+
- Node.js 24+
- PostgreSQL
- just, pnpm, lefthook

### Instalación

```bash
# 1. Instalar herramientas
just setup

# 2. Verificar que compila
cargo check --workspace

# 3. Configurar variables de entorno
cp .env.example .env.local
# Editar DATABASE_URL, PASETO_SECRET, etc.

# 4. Ejecutar migraciones
just migrate

# 5. Arrancar desarrollo
just dev
```

### Comandos

```bash
just dev           # Desarrollo completo
just dev-api       # Solo backend
just test          # Tests con nextest
just lint          # Clippy
just fmt           # Formateo
just audit         # Security audit
just deploy         # Deploy (Coolify)
```

---

## 📚 Documentación

| Documento | Descripción |
|-----------|-------------|
| `guia/PROMPT_MAESTRO.md` | Contexto global del proyecto |
| `guia/roadmap/01-ROADMAP-MASTER.md` | Mapa general de fases |
| `guia/roadmap/02-ROADMAP-GENESIS.md` | Fase de arranque |
| `guia/adr/` | ADRs (20 decisiones arquitectónicas) |
| `guia/verification_comando.md` | Verificaciones por fase |

---

## 🔐 Reglas de Arquitectura

1. **domain sin dependencias** - El crate domain NO puede importar sqlx, axum, etc.
2. **JWT prohibido** - Solo PASETO v4 Local (tokens empiezan con `v4.local.`)
3. **Soft Delete** - Nunca DELETE real, solo `UPDATE deleted_at`
4. **Fail-fast config** - Si falta variable de entorno, el proceso no arranca
5. **Todo autenticado se audita** - audit_logs automático

---

## 📋 Estado del Proyecto

| Fase | Estado |
|------|--------|
| Génesis | Pendiente |
| Backend | Pendiente |
| Frontend | Pendiente |
| Auth | Pendiente |
| Infra | Pendiente |
| **MVP** | **Pendiente** |

---

## 📄 Licencia

MIT

---

## 👤 Autores

Milton Hipamo / Laboratorio 3030

---

> **Nota:** Este proyecto está basado en los ADRs en `guia/adr/` como fuente de verdad.