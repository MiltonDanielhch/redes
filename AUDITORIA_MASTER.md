# 🛠️ Auditoría de Software — Lab 3030

> Generado: `2026-05-15 01:20`

## Resumen

| Métrica | Valor |
| :--- | :--- |
| **Proyecto** | `redes` |
| **Líneas de Código (Netas)** | 13084 LoC |
| **Peso Total del Proyecto** | 606.40KB |
| **Timestamp** | 2026-05-15 01:20 |
| **Estado** | Activa |

## Breakdown por Capa

| Capa / Archivo | LoC | Peso | % LoC |
| :--- | ---: | ---: | ---: |
| `guia` | 12938 | 600.97KB | 98.9% ███████████████████ |
| `audit.py` | 146 | 5.43KB | 1.1%  |
| **TOTAL** | **13084** | **606.40KB** | 100% |

## Mapa de Arquitectura

```text
redes/
├── audit.py (146 LoC | 5.43KB)
└── guia/ [600.97KB]
    ├── PROMPT_MAESTRO.md (413 LoC | 20.78KB)
    ├── adr/ [164.46KB]
    │   ├── ADR-0001-arquitectura-hexagonal.md (172 LoC | 10.55KB)
    │   ├── ADR-0002-configuracion-tipeada-secretos.md (169 LoC | 7.68KB)
    │   ├── ADR-0003-stack-backend-rust-axum.md (146 LoC | 6.96KB)
    │   ├── ADR-0004-persistencia-postgresql-docker.md (22 LoC | 1.09KB)
    │   ├── ADR-0005-migraciones-seeding.md (155 LoC | 7.02KB)
    │   ├── ADR-0006-rbac-sessions-audit.md (276 LoC | 12.62KB)
    │   ├── ADR-0007-manejo-errores.md (194 LoC | 8.73KB)
    │   ├── ADR-0008-seguridad-auth-paseto.md (176 LoC | 8.33KB)
    │   ├── ADR-0009-rate-limiting.md (141 LoC | 6.14KB)
    │   ├── ADR-0010-testing-calidad.md (322 LoC | 12.74KB)
    │   ├── ADR-0011-estandares-desarrollo.md (95 LoC | 4.85KB)
    │   ├── ADR-0012-herramientas-desarrollo.md (190 LoC | 8.21KB)
    │   ├── ADR-0013-build-externo-binarios.md (125 LoC | 6.19KB)
    │   ├── ADR-0014-infraestructura-docker-compose.md (48 LoC | 2.33KB)
    │   ├── ADR-0015-monitoreo-tareas-criticas.md (112 LoC | 5.61KB)
    │   ├── ADR-0019-mailer-resend.md (188 LoC | 7.46KB)
    │   ├── ADR-0021-documentacion-openapi-utoipa.md (166 LoC | 7.61KB)
    │   ├── ADR-0022-frontend-sveltekit-svelte5.md (25 LoC | 1.12KB)
    │   ├── ADR-0023-i18n-adaptacion-regional.md (135 LoC | 6.48KB)
    │   ├── ADR-0024-filosofia-local-first.md (129 LoC | 6.61KB)
    │   ├── ADR-0027-connectrpc-protobuf.md (120 LoC | 5.92KB)
    │   ├── ADR-0028-sintonia-cli.md (188 LoC | 8.48KB)
    │   ├── ADR-0029-landing-page-leads.md (244 LoC | 9.51KB)
    │   └── ADR-0035-monitoreo-infraestructura-regional.md (52 LoC | 2.23KB)
    ├── docs/ [212.11KB]
    │   ├── 00-GUIA-USO.md (182 LoC | 9.58KB)
    │   ├── 01-ARCHITECTURE.md (255 LoC | 11.15KB)
    │   ├── 02-STACK.md (436 LoC | 17.46KB)
    │   ├── 03-STRUCTURE.md (452 LoC | 19.17KB)
    │   ├── 04-VERIFICATION.md (928 LoC | 39.19KB)
    │   ├── 05-MODULES.md (201 LoC | 10.50KB)
    │   ├── ADR-VPS-5-PILARES.md (356 LoC | 15.47KB)
    │   ├── BRUJULA-COMPLETA.md (244 LoC | 14.75KB)
    │   ├── DASHBOARD-DISENO.md (292 LoC | 11.10KB)
    │   ├── INICIO.md (1450 LoC | 50.50KB)
    │   └── SINTONIA-CLI.md (332 LoC | 13.23KB)
    └── roadmap/ [203.62KB]
        ├── 00-ROADMAP-TEMPLATE.md (146 LoC | 4.43KB)
        ├── 01-ROADMAP-MASTER.md (248 LoC | 11.11KB)
        ├── 02-ROADMAP-GENESIS.md (422 LoC | 25.00KB)
        ├── 03-ROADMAP-BACKEND.md (760 LoC | 42.00KB)
        ├── 04-ROADMAP-FRONTEND.md (463 LoC | 27.92KB)
        ├── 05-ROADMAP-AUTH-FULLSTACK.md (492 LoC | 28.42KB)
        ├── 06-ROADMAP-LANDING.md (391 LoC | 21.97KB)
        ├── 07-ROADMAP-INFRA.md (454 LoC | 23.96KB)
        └── 80-ROADMAP-ADMIN.md (431 LoC | 18.79KB)
```
