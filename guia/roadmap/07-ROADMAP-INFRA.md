# Roadmap — Infraestructura y Deploy (Monitoreo de Infraestructura Regional)

> **Stack:** Podman rootless · Caddy · Kamal · Litestream · Tigris S3 · Healthchecks.io
>
> **ADRs:** 0035 (Monitoreo) · 0013 (Build) · 0014 (Deploy) · 0004 (Litestream) · 0015 (Monitoreo)

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| INF.I | Containerfile distroless | [ ] |
| INF.II | Caddy — TLS + seguridad | [ ] |
| INF.III | Litestream + S3 | [ ] |
| INF.IV | Kamal — deploy zero-downtime | [ ] |
| INF.V | Seguridad del VPS | [ ] |
| INF.VI | Monitoreo y alertas | [ ] |
| **Infra** | | [ ] |

---

## INF.I — Containerfile distroless (ADR 0013)

```
[ ] infra/docker/Containerfile:
    [ ] Stage 1: rust:1.85-slim (builder)
    [ ] musl-tools para target x86_64-unknown-linux-musl
    [ ] cargo build --release

    [ ] Stage 2: gcr.io/distroless/cc-debian12:nonroot
    [ ] Usuario no-root
    [ ] COPY binario
    [ ] HEALTHCHECK cmd wget localhost:3000/health
    [ ] EXPOSE 3000

[ ] Verificar imagen < 50MB
```

---

## INF.II — Caddy (ADR 0014)

```
[ ] infra/caddy/Caddyfile:
    [ ] tudominio.com → reverse_proxy localhost:3000 (API)
    [ ] tudominio.com → reverse_proxy localhost:4321 (Web)
    [ ] Security headers: HSTS, X-Frame-Options, CSP
    [ ] encode gzip zstd

[ ] TLS automático Let's Encrypt
```

---

## INF.III — Litestream + S3 (ADR 0004, ADR 0035)

```
[ ] Credenciales Tigris en .env:
    [ ] AWS_ENDPOINT_URL_S3
    [ ] AWS_ACCESS_KEY_ID
    [ ] AWS_SECRET_ACCESS_KEY
    [ ] LITESTREAM_BUCKET

[ ] infra/litestream/litestream.yml:
    [ ] path: /data/boilerplate.db
    [ ] type: s3
    [ ] retention: 168h

[ ] Verificar replicación a S3
```

---

## INF.IV — Kamal zero-downtime (ADR 0014)

```
[ ] infra/kamal/.kamal:
    [ ] service: monitoreo
    [ ] server: IP del VPS
    [ ] registry: Docker Hub o similar
    [ ] env.secret: PASETO_SECRET, RESEND_API_KEY, etc.
    [ ] volumes: ["/data/boilerplate:/data"]
    [ ] healthcheck: /health

[ ] Comandos:
    [ ] kamal setup
    [ ] just deploy
    [ ] kamal rollback

[ ] Verificar zero-downtime
```

---

## INF.V — Seguridad del VPS (ADR 0014)

```
[ ] Usuario deploy sin root
[ ] SSH solo con clave pública
[ ] UFW: solo 22, 80, 443
[ ] Unattended-upgrades

[ ] Verificar:
    [ ] ssh root@vps → rechazado
    [ ] ssh deploy@vps → funciona
```

---

## INF.VI — Monitoreo y alertas (ADR 0015, ADR 0035)

```
[ ] Healthchecks.io — crear checks:
    [ ] HC_WORKER_UUID → Apalis worker (5min)
    [ ] HC_LITESTREAM_UUID → Litestream (1h)
    [ ] HC_TLS_UUID → TLS (24h)
    [ ] HC_DEPLOY_UUID → Deploy (manual)

[ ] Variables en .env:
    [ ] HC_WORKER_UUID, HC_LITESTREAM_UUID, HC_TLS_UUID, HC_DEPLOY_UUID

[ ] Integración en código:
    [ ] Worker hace ping a HC_WORKER_UUID
    [ ] Litestream script hace ping a HC_LITESTREAM_UUID
    [ ] just deploy hace ping a HC_DEPLOY_UUID
```

---

## Verificación final — MVP en producción

```bash
# 1. Deploy exitoso
just deploy
# → audit ✓, tests ✓, kamal deploy ✓, HC ping ✓

# 2. HTTPS funcionando
curl -I https://tudominio.com/health
# → 200, Strict-Transport-Security presente

# 3. Zero-downtime
# while loop + kamal redeploy → ningún 502

# 4. Rollback
kamal rollback
# → < 10 segundos

# 5. Backup
litestream snapshots s3://bucket/boilerplate/db
# → entradas de hoy
```

---

## Troubleshooting

| Síntoma | Solución |
|---------|----------|
| Build falla "sqlx not found" | just prepare + commit |
| Imagen > 15MB | Usar distroless |
| Litestream no replica | Verificar credenciales AWS |
| Deploy falla healthcheck | Verificar /health endpoint |
| SSH rechazado | Verificar clave pública |

---

**Nota:** Este roadmap está basado en el ADR 0035 (Monitoreo de Infraestructura Regional) para la Gobernación del Beni.