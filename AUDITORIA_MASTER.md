# 🛠️ Auditoría de Software — Lab 3030

> Generado: `2026-05-16 01:15`

## Resumen

| Métrica | Valor |
| :--- | :--- |
| **Proyecto** | `redes` |
| **Líneas de Código (Netas)** | 9307 LoC |
| **Peso Total del Proyecto** | 310.24KB |
| **Timestamp** | 2026-05-16 01:15 |
| **Estado** | Activa |

## Breakdown por Capa

| Capa / Archivo | LoC | Peso | % LoC |
| :--- | ---: | ---: | ---: |
| `guia` | 9158 | 304.79KB | 98.4% ███████████████████ |
| `audit.py` | 146 | 5.43KB | 1.6%  |
| `README.md` | 3 | 20.00B | 0.0%  |
| **TOTAL** | **9307** | **310.24KB** | 100% |

## Mapa de Arquitectura

```text
redes/
├── README.md (3 LoC | 20.00B)
├── audit.py (146 LoC | 5.43KB)
└── guia/ [304.79KB]
    ├── PROMPT_MAESTRO.md (244 LoC | 11.32KB)
    ├── adr/ [208.32KB]
    │   ├── ADR-0001-arquitectura-hexagonal-corregido.md (208 LoC | 7.44KB)
    │   ├── ADR-0002-configuracion-tipeada-secretos-corregido.md (195 LoC | 5.42KB)
    │   ├── ADR-0003-stack-backend-rust-axum.md (216 LoC | 5.89KB)
    │   ├── ADR-0004-persistencia-postgresql-docker.md (187 LoC | 5.51KB)
    │   ├── ADR-0005-migraciones-seeding.md (215 LoC | 5.96KB)
    │   ├── ADR-0006-rbac-sessions-audit.md (279 LoC | 7.88KB)
    │   ├── ADR-0007-manejo-errores.md (252 LoC | 6.82KB)
    │   ├── ADR-0008-seguridad-auth-paseto.md (263 LoC | 8.90KB)
    │   ├── ADR-0009-rate-limiting.md (250 LoC | 8.73KB)
    │   ├── ADR-0010-testing-calidad.md (386 LoC | 11.05KB)
    │   ├── ADR-0011-estandares-desarrollo.md (319 LoC | 9.35KB)
    │   ├── ADR-0012-herramientas-desarrollo.md (375 LoC | 13.21KB)
    │   ├── ADR-0013-infraestructura-docker-compose.md (373 LoC | 10.34KB)
    │   ├── ADR-0014-monitoreo-tareas-criticas.md (209 LoC | 7.38KB)
    │   ├── ADR-0015-apalis-jobs.md (87 LoC | 3.66KB)
    │   ├── ADR-0016-documentacion-openapi-utoipa.md (423 LoC | 11.91KB)
    │   ├── ADR-0017-frontend-sveltekit-svelte5.md (374 LoC | 10.88KB)
    │   ├── ADR-0018-sintonia-cli.md (392 LoC | 11.43KB)
    │   ├── ADR-0019-coolify-deploy.md (298 LoC | 9.44KB)
    │   ├── ADR-0020-monitoreo-infraestructura-regional.md (442 LoC | 12.08KB)
    │   └── futura/ [35.03KB]
    │       ├── ADRFT-0001-build-externo-binarios.md (345 LoC | 9.26KB)
    │       ├── ADRFT-0002-mailer-resend.md (420 LoC | 11.33KB)
    │       ├── ADRFT-0003-persistencia-mysql.md (83 LoC | 3.89KB)
    │       └── ADRFT-0004-CONNECTRPC.md (354 LoC | 10.55KB)
    └── roadmap/ [85.15KB]
        ├── 00-ROADMAP-TEMPLATE.md (146 LoC | 4.43KB)
        ├── 01-ROADMAP-MASTER.md (195 LoC | 8.40KB)
        ├── 02-ROADMAP-GENESIS.md (393 LoC | 17.25KB)
        ├── 03-ROADMAP-BACKEND.md (483 LoC | 23.36KB)
        ├── 04-ROADMAP-FRONTEND.md (408 LoC | 21.31KB)
        ├── 05-ROADMAP-AUTH-FULLSTACK.md (173 LoC | 4.99KB)
        ├── 06-ROADMAP-LANDING.md (28 LoC | 1.01KB)
        ├── 07-ROADMAP-INFRA.md (129 LoC | 3.85KB)
        └── 80-ROADMAP-ADMIN.md (14 LoC | 570.00B)
```
