# Verificaciones — Guía de Pruebas por Fase

> Corre estas verificaciones al terminar cada bloque del roadmap.
> Si algo falla, no avances al siguiente bloque.
>
> **Proyecto:** Monitoreo de Infraestructura Regional - Gobernación del Beni

---

## Cómo usar este documento

```
1. Terminas un bloque del ROADMAP
2. Vienes aquí y corres las verificaciones de ese bloque
3. Todo ✅ → avanzas
4. Algo ❌ → resuelves antes de continuar
```

---

## Bloque Génesis — Workspace + Tooling

> **Referencia:** ADR 0012 (Tooling), ADR 0018 (Sintonía CLI)

### 1. El workspace compila

```bash
cargo check --workspace
```
**Esperado:** `"Finished"` sin errores

### 2. No hay JWT en ningún Cargo.toml

```bash
grep -r "jsonwebtoken" . --include="*.toml"
```
**Esperado:** cero resultados  
**Ref:** ADR 0008 — JWT prohibido

### 3. Licencias y CVEs limpios

```bash
cargo deny 0.19.6 check
```
**Esperado:** sin violations

### 4. Herramientas instaladas

```bash
just --version && sqlx --version && cargo nextest --version
```
**Esperado:** versiones impresas sin error

### 5. El hook de lefthook funciona

```bash
git commit --allow-empty -m "test genesis"
```
**Esperado:** lefthook ejecuta fmt antes del commit

---

## Bloque I — Fundación (Dominio + DB + RBAC)

> **Referencia:** ADR 0001 (Hexagonal), ADR 0004 (PostgreSQL), ADR 0006 (RBAC)

### Las migraciones

**Ejecutar migraciones:**
```bash
just migrate
```
**Esperado:** PostgreSQL migraciones aplicadas

**Verificar tablas creadas:**
```bash
psql -U postgres -d monitoreo -c "\dt"
```
**Esperado:** users, roles, permissions, sessions, audit_logs, sedes, devices, etc.

**Verificar que el admin existe:**
```bash
psql -U postgres -d monitoreo -c "SELECT email FROM users;"
```
**Esperado:** `admin@admin.com`

### El dominio no tiene dependencias externas

```bash
cat crates/domain/Cargo.toml
# Esperado: solo thiserror, uuid, time, serde

cargo nextest 0.9.135 run -p domain
# Esperado: todos los tests pasan
```

---

## Bloque II — API (Axum + Middleware)

> **Referencia:** ADR 0003 (Axum), ADR 0009 (Rate Limit), ADR 0016 (OpenAPI)

```bash
# Arrancar el servidor
just dev-api

# Health check
curl http://localhost:3000/health
# Esperado: {"status":"ok","database":"connected"}

# La API responde JSON
curl http://localhost:3000/api/v1/sedes
# Esperado: 401 Unauthorized (ruta protegida)

# Rate limit funciona
for i in {1..35}; do curl -s -o /dev/null -w "%{http_code}\n" http://localhost:3000/api/v1/sedes; done
# Esperado: los últimos retornan 429
```

---

## Bloque III — Seguridad (PASETO + Auth + RBAC)

> **Referencia:** ADR 0006 (RBAC), ADR 0007 (Errores), ADR 0008 (PASETO)

```bash
# 1. Registro de usuario
curl -X POST http://localhost:3000/auth/register   -H "Content-Type: application/json"   -d '{"email":"test@example.com","password":"password123"}'

# 2. Login y obtener token
TOKEN=$(curl -s -X POST http://localhost:3000/auth/login   -H "Content-Type: application/json"   -d '{"email":"test@example.com","password":"password123"}'   | jq -r '.access_token')
echo "Token: $TOKEN"
# Esperado: token PASETO (empieza con "v4.local.")

# 3. Request autenticado
curl -H "Authorization: Bearer $TOKEN"   http://localhost:3000/api/v1/sedes
# Esperado: 200 OK con lista de sedes

# 4. Token inválido es rechazado
curl -H "Authorization: Bearer invalid.token.here"   http://localhost:3000/api/v1/sedes
# Esperado: 401 {"error":"unauthorized"}

# 5. Verificar que PASETO empieza con v4.local
echo $TOKEN | cut -c1-10
# Esperado: "v4.local.X"

# 6. Soft Delete — verificar que no hace DELETE real
curl -X DELETE -H "Authorization: Bearer $ADMIN_TOKEN"   http://localhost:3000/api/v1/users/USER_ID
psql -U postgres -d monitoreo -c "SELECT deleted_at FROM users WHERE email='test@example.com';"
# Esperado: fecha ISO — NO es NULL
```

---

## Bloque IV — Documentación API

> **Referencia:** ADR 0016 (OpenAPI)

```bash
# Scalar UI disponible
curl http://localhost:3000/docs

# OpenAPI spec válido
curl http://localhost:3000/openapi.json | jq '.info.title'
# Esperado: "Monitoreo API"

# El spec tiene los endpoints de monitoreo
curl http://localhost:3000/openapi.json | jq '.paths | keys[]'
```

---

## Bloque V — Async (Jobs + Cache)

> **Referencia:** ADR 0015 (Jobs), ADR 0014 (Monitoreo)

```bash
# 1. Jobs se encolan
curl -X POST http://localhost:3000/auth/register   -H "Content-Type: application/json"   -d '{"email":"job-test@example.com","password":"password123"}'

# Verificar jobs en PostgreSQL
psql -U postgres -d monitoreo -c "SELECT job_type, status FROM jobs ORDER BY created_at DESC LIMIT 5;"

# 2. Verificar logs
just dev-api 2>&1 | grep -i "job"
```

---

## Bloque FE.I — Frontend Foundation

> **Referencia:** ADR 0017 (Frontend)

```bash
# 1. pnpm dev arranca sin errores
cd apps/web && pnpm dev

# 2. Dashboard carga
curl http://localhost:3000
# Esperado: HTML del dashboard
```

---

## Verificaciones de código — Estándares ADR 0011

```bash
# 1. domain NO importa sqlx
grep -r "sqlx" crates/domain/Cargo.toml

# 2. domain NO importa axum
grep -r "axum" crates/domain/Cargo.toml

# 3. JWT prohibido en todo el workspace
grep -r "jsonwebtoken" . --include="*.toml"

# 4. No hay DELETE real en users
grep -rn "DELETE FROM users" . --include="*.rs"
# Esperado: cero resultados — solo UPDATE deleted_at

# 5. Todos los tests pasan
cargo nextest 0.9.135 run --all-targets

# 6. Sin warnings en el código
cargo clippy --all-targets -- -D warnings
```

---

## Checklist de "listo para producción"

```
[ ] cargo nextest 0.9.135 run --all-targets → todos pasan
[ ] cargo clippy --all-targets -D warnings → cero warnings
[ ] just audit → cargo deny 0.19.6 + cargo audit sin issues
[ ] grep "jsonwebtoken" → cero resultados
[ ] grep "DELETE FROM users" → cero resultados
[ ] PostgreSQL conectividad verificada
[ ] Healthchecks.io pings configurados
[ ] curl /health → 200 OK
```

---

## Notas de corrección (v2.0 → v2.1)

**Cambios aplicados al 2026-05-16:**

1. **cargo deny:** Se especifica versión `0.19.6` (latest estable al 11 may 2026) citeweb_search:19#0
2. **cargo nextest:** Se especifica versión `0.9.135` (latest estable al 14 may 2026) citeweb_search:21#0
3. **just:** Se mantiene sin versión específica en comandos (el documento usa `just --version` para verificar instalación). `just` es un command runner muy activo con releases frecuentes; la última minor es 1.43.x (sep 2025) con patches continuos citeweb_search:21#3
4. **sqlx:** Se mantiene sin versión específica en comandos (el documento usa `sqlx --version` para verificar instalación). sqlx CLI se instala vía `cargo install sqlx-cli` y su versión sigue la del crate (0.8.6 current stable)
5. **cargo clippy:** Se mantiene como componente de rustup (instalado vía `rustup component add clippy`). La versión de clippy está ligada a la versión de Rust toolchain (1.95.0 en este proyecto) citeweb_search:21#5
6. **lefthook:** Se mantiene sin versión específica en comandos (el documento verifica que el hook funcione en git commit). Versión latest estable: 2.1.6 (16 abr 2026) citeweb_search:19#1web_search:19#7
7. **Healthchecks.io:** Se mantiene como servicio SaaS externo para monitoreo de jobs y backups (ADR 0014)
8. **No se requiere cambio de puerto:** El documento usa puerto 3000 para API y frontend en desarrollo, consistente con el stack SvelteKit/Vite

---

## Historial de cambios

| Versión | Fecha       | Cambios realizados |
| ------- | ----------- | ------------------ |
| 1.0     | 2026 (orig) | Versión inicial con versiones genéricas de herramientas |
| 2.0     | 2026-05-16  | Revisión general de estructura y verificaciones |
| 2.1     | 2026-05-16  | Fija versiones exactas: cargo deny `0.19.6`, cargo nextest `0.9.135`; documenta versiones de referencia para just, lefthook, sqlx, cargo clippy; actualiza checklist de producción |
