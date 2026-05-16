# Roadmap — Auth Fullstack (Monitoreo de Infraestructura Regional)

> **Pre-requisitos:**
> - Backend Bloques I, II, III completados
> - Frontend FE.I, FE.II, FE.III completados
>
> **ADRs:** 0020 · 0001 · 0003 · 0006 · 0007 · 0008 · 0016 · 0017

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Sección | Nombre | Progreso |
|---------|--------|----------|
| A.1 | Registro — back + front | [ ] |
| A.2 | Login — back + front | [ ] |
| A.3 | Sesión activa y protección de rutas | [ ] |
| A.4 | Refresh de tokens | [ ] |
| A.5 | Logout | [ ] |
| A.6 | RBAC — permisos en acción | [ ] |
| **Total Auth** | | [ ] |

---

## A.1 — Registro (POST /auth/register)

### Backend

```
[ ] Handler: POST /auth/register
    [ ] Valida email único
    [ ] Valida password: min 8 chars
    [ ] PasswordHasher::hash(argon2id)
    [ ] Guarda user en DB
    [ ] Retorna 201 { "id": "uuid", "email": "..." }
    [ ] 409 si email duplicado

[ ] RegisterUseCase:
    [ ] Email::new() valida
    [ ] find_active_by_email() → 409 si existe
    [ ] hash_password()
    [ ] users.save()
    [ ] audit.log()
```

### Frontend

```
[ ] pages/register.astro + components/auth/RegisterForm.svelte
    [ ] Campos: email, password, confirmación
    [ ] ArkType validation
    [ ] TanStack mutation → POST /auth/register
    [ ] onSuccess: redirect /login
    [ ] onError: mostrar mensaje
```

---

## A.2 — Login (POST /auth/login)

### Backend

```
[ ] Handler: POST /auth/login
    [ ] Busca user por email
    [ ] verify_password(argon2id)
    [ ] Si inválido → 401
    [ ] PasetoService::generate_access_token(15min) → "v4.local.xxx"
    [ ] generate_opaque_token() → refresh
    [ ] audit.log(login_success)
    [ ] Retorna 200 { access_token, refresh_token }
```

### Frontend

```
[ ] pages/login.astro + components/auth/LoginForm.svelte
    [ ] Campos: email, password
    [ ] ArkType validation
    [ ] TanStack mutation → POST /auth/login
    [ ] onSuccess: setAuth(user, token) + redirect /dashboard
    [ ] onError: "Credenciales incorrectas"
```

---

## A.3 — Sesión activa y protección de rutas

### Backend

```
[ ] auth_middleware en todas las rutas /api/v1/*
[ ] require_permission() en rutas específicas
```

### Frontend

```
[ ] DashboardLayout.astro — verificación SSR:
    [ ] Lee access_token
    [ ] Si no hay sesión → redirect /login

[ ] auth.svelte.ts — estado global:
    [ ] Estado desde localStorage
    [ ] isLoggedIn derivado
    [ ] user.permissions[] disponible
```

---

## A.4 — Refresh de tokens

### Backend

```
[ ] POST /auth/refresh:
    [ ] extrae refresh_token
    [ ] hash_token() y buscar en DB
    [ ] REVOCAR refresh anterior
    [ ] generar nuevo access + refresh
    [ ] retorna { access_token, refresh_token }
```

### Frontend

```
[ ] Interceptor en lib/api/client.ts:
    [ ] Si 401 + refresh_token → POST /auth/refresh
    [ ] Si exitoso → actualizar token + reintentar
    [ ] Si falla → clearAuth() + redirect /login
```

---

## A.5 — Logout

### Backend

```
[ ] POST /auth/logout (requiere auth)
    [ ] Extrae user_id del token
    [ ] sessions.revoke_all_sessions(user_id)
    [ ] retorna 200
```

### Frontend

```
[ ] Logout en Topbar.svelte:
    [ ] TanStack mutation → POST /auth/logout
    [ ] onSettled: clearAuth() + redirect /login
```

---

## A.6 — RBAC en acción

### Backend

```
[ ] require_permission("devices:read") en GET /api/v1/devices
[ ] require_permission("devices:write") en POST/PUT/DELETE
[ ] require_permission("alerts:read") en GET /api/v1/alerts
[ ] require_permission("audit:read") en GET /api/v1/audit
```

### Frontend

```
[ ] user.permissions[] cargado en login
[ ] PermissionGate para componentes:
    [ ] devices:write → crear, editar, eliminar
    [ ] alerts:read → ver alertas
    [ ] audit:read → ver auditoría
[ ] Sidebar filtra items por permiso
```

---

## Verificación final

```bash
# Registro
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@test.com","password":"password_segura_123"}'
# → 201

# Login
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@test.com","password":"password_segura_123"}' \
  | jq -r '.access_token')
echo $TOKEN | cut -c1-10  # → "v4.local.."

# Request autenticada
curl http://localhost:8080/api/v1/devices \
  -H "Authorization: Bearer $TOKEN"
# → 200

# Sin permiso
curl -X POST http://localhost:8080/api/v1/devices \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"hostname":"test"}'
# → 403
```

---

## Troubleshooting

| Síntoma | Solución |
|---------|----------|
| Login siempre 401 | Verificar password hash |
| access_token empieza con "eyJ" | Usar PASETO, no JWT |
| Permisos no cargados | Verificar login response incluye permissions |
| RBAC no funciona | Verificar backend + frontend |

---

**Nota:** Este roadmap está basado en el ADR 0020 (Monitoreo de Infraestructura Regional) para la Gobernación del Beni.