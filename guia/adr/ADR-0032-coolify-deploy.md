# ADR 0032 — Deploy Alternativo: Coolify + SQLite + Litestream

| Campo               | Valor                                                                                                               |
| ------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **Estado**          | ✅ Aceptado — alternativa válida a Kamal                                                                             |
| **Fecha**           | 2026                                                                                                                |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                                    |
| **Relacionado con** | ADR 0014 (Deploy — Kamal), ADR 0004 (SQLite + Litestream), ADR 0013 (Containerfile Distroless), ADR VPS (5 Pilares) |

---

# Contexto

El stack principal usa **Kamal** como sistema de deploy (ADR 0014).
Kamal opera desde la máquina del desarrollador:

* build local
* push al registry
* deploy vía SSH
* rollback instantáneo
* sin componentes extra en el VPS

Este enfoque es extremadamente eficiente para VPS pequeños.

Sin embargo, algunos equipos prefieren una experiencia tipo PaaS con:

* dashboard web
* deploy automático desde Git
* SSL visual
* manejo centralizado de múltiples aplicaciones
* logs y rollbacks desde interfaz gráfica

Para esto, **Coolify** surge como alternativa self-hosted moderna.

La pregunta clave es:

> ¿Coolify puede ejecutar correctamente este stack basado en SQLite WAL + Litestream sin romper la arquitectura minimalista?

---

# Decisión

Se acepta **Coolify** como alternativa válida a Kamal para despliegue del stack.

Coolify:

* NO reemplaza la arquitectura actual
* NO modifica el Containerfile
* NO altera Litestream
* NO cambia SQLite
* NO introduce Kubernetes
* NO requiere reescribir el sistema

Solo cambia la capa operativa del deploy.

---

# Principio arquitectónico

La aplicación debe seguir siendo:

* un contenedor autónomo
* con SQLite local
* Litestream integrado
* restauración automática
* healthcheck HTTP
* sin dependencias externas obligatorias

Coolify únicamente orquesta el ciclo de vida del contenedor.

---

# Compatibilidad con SQLite + Litestream

## Estado

| Componente                 | Compatibilidad                     |
| -------------------------- | ---------------------------------- |
| SQLite WAL                 | ✅ Compatible                       |
| Litestream sidecar interno | ✅ Compatible                       |
| Distroless                 | ✅ Compatible                       |
| Axum                       | ✅ Compatible                       |
| Healthcheck `/health`      | ✅ Compatible                       |
| Restore automático         | ✅ Compatible                       |
| Volume persistence         | ✅ Requiere configuración explícita |

---

# Arquitectura con Coolify

```text
Git Push
   ↓
Coolify Webhook
   ↓
Build Docker Image
   ↓
Deploy Container
   ↓
Mount /data volume
   ↓
ENTRYPOINT:
litestream replicate -exec /api
   ↓
SQLite restaura desde S3 si no existe
   ↓
Axum inicia
   ↓
/health OK
   ↓
Traffic swap
```

---

# Persistencia de SQLite

## Regla obligatoria

SQLite debe vivir en un volumen persistente Docker.

En Coolify:

```text
Persistent Storage
Destination Path: /data
```

Entonces:

```env
DATABASE_URL=sqlite:/data/boilerplate.db
```

---

# Litestream — sin cambios

El patrón del ADR 0004 sigue intacto.

```dockerfile
ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]
```

Litestream:

1. restaura desde S3 si la DB no existe
2. inicia replicación continua
3. ejecuta la API Rust

Coolify no interfiere.

---

# Configuración recomendada en Coolify

## Variables de entorno

```env
SERVER_PORT=8080
ENVIRONMENT=production

DATABASE_URL=sqlite:/data/boilerplate.db

PASETO_SECRET=...
RESEND_API_KEY=...
MAIL_FROM=noreply@tudominio.com

AWS_ENDPOINT_URL_S3=...
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...

LITESTREAM_BUCKET=...

RUST_LOG=info
```

---

# Healthcheck

```text
Path: /health
Port: 8080
Interval: 10s
Timeout: 5s
Retries: 3
```

Coolify no redirige tráfico hasta que el healthcheck pasa.

---

# Containerfile

El Containerfile del ADR 0013 funciona sin modificaciones.

```dockerfile
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /api /api

COPY --from=ghcr.io/benbjohnson/litestream:latest-amd64 \
     /usr/local/bin/litestream /litestream

COPY infra/litestream/litestream.yml /etc/litestream.yml

EXPOSE 8080

ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]
```

---

# Litestream configuration

```yaml
# infra/litestream/litestream.yml

dbs:
  - path: /data/boilerplate.db

    replicas:
      - type: s3

        bucket: ${LITESTREAM_BUCKET}
        path: boilerplate/db

        endpoint: ${AWS_ENDPOINT_URL_S3}

        sync-interval: 1s
        snapshot-interval: 24h
        retention: 72h
```

---

# Deploy Flow

## Kamal

```text
Laptop → SSH → Deploy
```

## Coolify

```text
Git Push → Webhook → Auto Deploy
```

---

# Comparativa — Kamal vs Coolify

| Aspecto             | Kamal        | Coolify          |
| ------------------- | ------------ | ---------------- |
| Filosofía           | Minimalista  | PaaS visual      |
| Interfaz            | CLI          | Dashboard web    |
| RAM extra           | ~0MB         | ~500MB-1GB       |
| SSL                 | Caddy manual | Automático       |
| Rollback            | CLI          | Visual           |
| Multi-app           | Manual       | Integrado        |
| SQLite + Litestream | ✅ Excelente  | ✅ Excelente      |
| VPS ideal           | $5           | $10+             |
| Complejidad         | Baja         | Media            |
| Dependencias        | Docker       | Docker + Coolify |

---

# Cuándo usar Coolify

## Elegir Coolify si

* se manejan múltiples proyectos
* se quiere dashboard visual
* el equipo no domina SSH
* se desea auto-deploy desde Git
* el VPS tiene ≥2GB RAM

---

# Cuándo mantener Kamal

## Mantener Kamal si

* el VPS es de 1GB RAM
* se prioriza simplicidad extrema
* se quiere mínimo consumo posible
* el equipo ya domina terminal
* existe solo un proyecto

---

# Requisitos del VPS

| Escenario | Recomendación     |
| --------- | ----------------- |
| Kamal     | VPS $5 — 1GB RAM  |
| Coolify   | VPS $10 — 2GB RAM |

Coolify ejecuta múltiples contenedores internos:

* Traefik
* DB interna
* Redis
* workers
* scheduler
* proxy
* etc.

Esto consume memoria adicional.

---

# Seguridad

## Reglas obligatorias

* Nunca exponer SQLite fuera del contenedor
* `/data` solo accesible internamente
* Secrets únicamente en variables cifradas
* HTTPS obligatorio
* Healthcheck obligatorio
* Litestream siempre activo

---

# Integración con el stack existente

Coolify es compatible con:

| ADR                            | Compatibilidad |
| ------------------------------ | -------------- |
| ADR 0001 — Monolito Modular    | ✅              |
| ADR 0003 — Axum                | ✅              |
| ADR 0004 — SQLite + Litestream | ✅              |
| ADR 0013 — Distroless          | ✅              |
| ADR 0018 — Apalis              | ✅              |
| ADR 0016 — Resend              | ✅              |
| ADR 0022 — Frontend SvelteKit  | ✅              |
| ADR 0029 — Landing SSR         | ✅              |

---

# Alternativas consideradas

| Opción     | Motivo de descarte                |
| ---------- | --------------------------------- |
| Dokploy    | Ecosistema menos maduro           |
| CapRover   | Arquitectura más antigua          |
| Heroku     | Vendor lock-in                    |
| Railway    | No self-hosted                    |
| Fly.io     | Excelente pero no self-hosted     |
| Kubernetes | Overkill absoluto para este stack |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta       | Propósito                                   |
| ----------------- | ------------------------------------------- |
| `Coolify`         | Orquestación visual self-hosted             |
| `Traefik`         | SSL automático y reverse proxy              |
| `Healthchecks.io` | Monitoreo de jobs y backups                 |
| `Watchtower`      | Actualizaciones automáticas de contenedores |
| `Dozzle`          | Visualización ligera de logs Docker         |
| `ctop`            | Monitoreo de recursos en tiempo real        |
| `glances`         | Observabilidad del VPS                      |
| `fail2ban`        | Protección SSH y reverse proxy              |

---

# Consecuencias

## ✅ Positivas

* Dashboard visual para deploys y logs
* SSL automático
* Rollbacks visuales
* Multi-proyecto en un solo VPS
* Deploy automático vía Git
* Litestream y SQLite funcionan sin cambios
* No se rompe la arquitectura minimalista

---

## ⚠️ Negativas / Trade-offs

### Mayor consumo de RAM

Coolify agrega múltiples contenedores internos.

→ Mitigación:

* usar VPS de 2GB RAM
* swap activo según ADR VPS

---

### Mayor complejidad operativa

Más componentes:

* Traefik
* workers
* DB interna
* Redis

→ Mitigación:

* mantener arquitectura del app extremadamente simple
* un solo contenedor de aplicación

---

### Menos minimalista que Kamal

Kamal sigue siendo más eficiente para:

* un solo proyecto
* VPS pequeños
* equipos técnicos

Coolify brilla cuando existen múltiples aplicaciones.

---

# Decisiones derivadas

* `DATABASE_URL` apunta siempre a `/data/boilerplate.db`
* Litestream sigue integrado en el mismo contenedor
* El Containerfile distroless no cambia
* Kamal sigue siendo la opción principal del stack
* Coolify es alternativa oficialmente soportada
* VPS de 1GB → Kamal recomendado
* VPS de 2GB+ → Coolify válido
* El deploy nunca depende de Kubernetes
* SQLite sigue siendo la base oficial del proyecto
