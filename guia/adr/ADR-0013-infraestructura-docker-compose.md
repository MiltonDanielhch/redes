# ADR 0013 — Infraestructura: Docker Compose + Red Privada

| Campo               | Valor                                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------ |
| **Estado**          | ✅ Aceptado                                                                                       |
| **Fecha**           | 2026-05-15                                                                                       |
| **Autores**         | Milton Hipamo / Laboratorio 3030                                                                 |
| **Relacionado con** | ADR 0012 (Tooling), ADR 0003 (Backend Axum), ADR 0014 (Monitoreo) |

---

# Contexto

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

# Decisión

Usar **Docker Compose** como orquestador oficial de infraestructura.

La plataforma se ejecuta mediante:

* contenedores separados
* red privada interna
* volúmenes persistentes
* variables de entorno
* comunicación interna por DNS de Docker

---

# Arquitectura General

```text id="iywn4f"
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
│   │       :5432     │                               │
│   └─────────────────┘                               │
│                                                      │
└──────────────────────────────────────────────────────┘
```

---

# Objetivos de Infraestructura

## Simplicidad Operacional

Cualquier operador debe poder levantar el sistema con:

```bash id="zwr6y6"
docker compose up -d
```

---

## Reproducibilidad

Todos los ambientes:

* desarrollo
* staging
* producción

usan exactamente la misma arquitectura.

---

## Aislamiento

Cada servicio:

* tiene su propio contenedor
* dependencias aisladas
* networking privado
* lifecycle independiente

---

## Mantenibilidad

Logs, reinicios y debugging centralizados mediante Docker Compose.

---

# Servicios Oficiales

---

# 1 — Backend

## Stack

* Rust
* Axum
* Tokio
* SQLx

## Responsabilidades

* API REST
* autenticación
* monitoreo
* lógica de negocio
* acceso a DB

## Configuración

| Parámetro   | Valor                  |
| ----------- | ---------------------- |
| Imagen      | `redes-backend:latest` |
| Puerto      | `8080`                 |
| Expuesto    | Sí                     |
| Dependencia | PostgreSQL             |

---

# 2 — Frontend

## Stack

* SvelteKit
* TypeScript

## Responsabilidades

* dashboard
* visualización
* panel administrativo
* monitoreo en tiempo real

## Configuración

| Parámetro   | Valor                   |
| ----------- | ----------------------- |
| Imagen      | `redes-frontend:latest` |
| Puerto      | `3000`                  |
| Expuesto    | Sí                      |
| Dependencia | Backend                 |

---

# 3 — PostgreSQL

## Base oficial

```text id="0mjlwm"
postgres:16-alpine
```

---

## Responsabilidades

* persistencia
* métricas
* usuarios
* auditoría
* eventos de red

## Configuración

| Parámetro        | Valor                |
| ---------------- | -------------------- |
| Imagen           | `postgres:16-alpine` |
| Puerto           | `5432`               |
| Expuesto externo | No                   |
| Persistencia     | `postgres_data`      |

---

# Docker Compose Oficial

```yaml id="udqlw5"
version: "3.9"

services:

  backend:
    image: redes-backend:latest
    container_name: redes-backend

    restart: unless-stopped

    env_file:
      - .env

    ports:
      - "8080:8080"

    depends_on:
      - postgres

    networks:
      - red-interna

  frontend:
    image: redes-frontend:latest
    container_name: redes-frontend

    restart: unless-stopped

    ports:
      - "3000:3000"

    depends_on:
      - backend

    networks:
      - red-interna

  postgres:
    image: postgres:16-alpine
    container_name: redes-postgres

    restart: unless-stopped

    environment:
      POSTGRES_DB: redes
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres

    volumes:
      - postgres_data:/var/lib/postgresql/data

    expose:
      - "5432"

    networks:
      - red-interna

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

# Networking

## Política

La DB jamás se expone públicamente.

---

# Comunicación interna

Docker Compose provee DNS interno automático:

| Servicio           | Host interno    |
| ------------------ | --------------- |
| Backend → DB       | `postgres:5432` |
| Frontend → Backend | `backend:8080`  |

---

# Beneficios

* sin IPs hardcodeadas
* portable
* simple
* reproducible

---

# Variables de Entorno

## `.env`

```env id="h6npxc"
DATABASE_URL=postgres://postgres:postgres@postgres:5432/redes

RUST_LOG=info

APP_PORT=8080

FRONTEND_URL=http://localhost:3000
```

---

# Comandos Operativos Oficiales

## Levantar infraestructura

```bash id="aqbqbh"
docker compose up -d
```

---

## Ver logs

```bash id="fkwlyh"
docker compose logs -f
```

---

## Detener

```bash id="jgpt6x"
docker compose down
```

---

## Reiniciar servicio específico

```bash id="ahkzsu"
docker compose restart backend
```

---

## Ver estado

```bash id="o7wqz2"
docker compose ps
```

---

# Política de Persistencia

## PostgreSQL

La data vive en:

```text id="9n7g8o"
postgres_data
```

Aunque el contenedor muera:

* la DB sobrevive
* reinicio seguro
* upgrades simples

---

# Seguridad

## Red privada

PostgreSQL no expone puertos públicos.

---

## Aislamiento

Cada servicio:

* filesystem separado
* procesos separados
* dependencias aisladas

---

## Superficie mínima

* alpine en PostgreSQL
* imágenes específicas
* sin herramientas innecesarias

---

# Healthchecks Recomendados

## Backend

```yaml id="8z5l0o"
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 30s
  timeout: 5s
  retries: 3
```

---

## PostgreSQL

```yaml id="jlwmjlwm"
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U postgres"]
  interval: 10s
  timeout: 5s
  retries: 5
```

---

# Logs

## Estrategia oficial

Logs centralizados vía:

```bash id="4znq7y"
docker compose logs
```

---

# Futuro

Preparado para:

* Loki
* Grafana
* Promtail
* OpenTelemetry

---

# Alternativas Consideradas

| Opción       | Motivo de descarte                      |
| ------------ | --------------------------------------- |
| Kubernetes   | Complejidad excesiva para esta fase     |
| Docker Swarm | Ecosistema reducido                     |
| Nomad        | Overkill para MVP                       |
| Systemd puro | Sin aislamiento ni networking integrado |
| PM2          | Solo Node.js                            |

---

# Herramientas y Librerías para Optimizar (Edición 2026)

| Herramienta    | Propósito                         |
| -------------- | --------------------------------- |
| `lazydocker`   | TUI para administrar contenedores |
| `ctop`         | Monitoreo live de recursos Docker |
| `watchtower`   | Auto-update de imágenes           |
| `dive`         | Analizar tamaño de imágenes       |
| `docker scout` | Escaneo de vulnerabilidades       |

---

# Consecuencias

## ✅ Positivas

### Infraestructura reproducible

Mismo entorno en todos lados.

---

### Operación simple

Todo el sistema:

```bash id="4gc1ll"
docker compose up -d
```

---

### Aislamiento real

Fallos de un servicio no contaminan otros.

---

### Fácil debugging

Logs y estado centralizados.

---

### Bajo costo operacional

Perfecto para:

* VPS pequeños
* equipos reducidos
* proyectos gubernamentales

---

## ⚠️ Negativas / Trade-offs

### No hay alta disponibilidad horizontal

Mitigación futura:

* Kubernetes
* Nomad
* Docker Swarm

---

### Compose no es autoscaling

Aceptable para:

* MVP
* staging
* producción pequeña/media

---

### PostgreSQL vive en un solo nodo

Mitigación futura:

* backups automáticos
* replicas
* WAL shipping

---

# Decisiones derivadas

* Toda comunicación interna usa red privada Docker
* PostgreSQL nunca expone puertos públicos
* El deployment oficial usa Docker Compose
* Los contenedores reinician automáticamente
* La persistencia se maneja con volúmenes Docker
* El backend depende explícitamente de PostgreSQL
* El frontend depende explícitamente del backend
