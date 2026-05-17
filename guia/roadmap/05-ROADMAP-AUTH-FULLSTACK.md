# Roadmap — Auth Fullstack (Monitoreo de Infraestructura Regional)

> **Pre-requisitos:**
> - Backend Bloques I, II, III completados
> - Frontend FE.I, FE.II, FE.III completados
>
> **ADRs:** 0020 · 0001 · 0003 · 0006 · 0007 · 0008 · 0009 · 0016 · 0017

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
| A.2 | Verificación de email — back + front | [ ] |
| A.3 | Login — back + front | [ ] |
| A.4 | Sesión activa y protección de rutas | [ ] |
| A.5 | Refresh de tokens | [ ] |
| A.6 | Logout — back + front | [ ] |
| A.7 | Password Reset — back + front | [ ] |
| A.8 | RBAC — permisos en acción | [ ] |
| A.9 | Rate Limiting en auth | [ ] |
| **Total Auth** | | [ ] |

---

## A.1 — Registro (POST /auth/register)

> **Referencia:** ADR 0008, ADR 0006, ADR 0009

### Backend

```
[ ] Handler: POST /auth/register
    [ ] Rate limiting: max 5 intentos / 15min por IP (ADR 0009)
    [ ] Valida email: formato RFC 5322 + único (case-insensitive)
    [ ] Valida password: mínimo 12 caracteres, 1 mayúscula, 1 minúscula, 1 número, 1 símbolo
    [ ] PasswordHasher::hash(argon2id, OWASP 2024 params)
    [ ] Guarda user en DB con is_active = false (hasta verificación)
    [ ] Genera token de verificación (opaque, 32 bytes, SHA-256 hash en DB)
    [ ] Envía email de verificación vía Resend (ADR 0016)
    [ ] audit.log(action: "user_registered", resource: "users", resource_id: user.id)
    [ ] Retorna 201 { "id": "usr_xxx", "email": "...", "message": "Verificación enviada" }
    [ ] 409 si email duplicado (incluso soft-deleted)
    [ ] 422 si validación falla

[ ] RegisterUseCase:
    [ ] Email::new() valida formato
    [ ] find_active_by_email() → 409 si existe y no está soft-deleted
    [ ] find_soft_deleted_by_email() → 409 si fue eliminado (no reutilizar emails)
    [ ] hash_password() con argon2id
    [ ] users.save() con is_active = false
    [ ] tokens.create_verification_token(user_id)
    [ ] audit.log()
    [ ] Retorna user_id (no expone datos sensibles)
```

### Frontend

```
[ ] src/routes/(auth)/register/+page.svelte
    [ ] Layout: AuthLayout (centrado, sin sidebar)
    [ ] Componente: components/auth/RegisterForm.svelte
    [ ] Campos: name, email, password, password_confirmation
    [ ] ArkType validation en tiempo real:
        [ ] email: formato válido
        [ ] password: 12+ chars, fuerza visual (zxcvbn opcional)
        [ ] password_confirmation: coincide con password
    [ ] TanStack mutation → POST /auth/register
    [ ] onSuccess: toast "Revisa tu email para verificar" + redirect /login
    [ ] onError: mostrar mensaje específico (409 = email duplicado, 422 = validación)
    [ ] Botón deshabilitado mientras carga
```

---

## A.2 — Verificación de email (GET /auth/verify/:token)

> **Referencia:** ADR 0008, ADR 0016

### Backend

```
[ ] Handler: GET /auth/verify/:token
    [ ] Extrae token raw de URL params
    [ ] hash_token(raw) → SHA-256
    [ ] Busca en tokens WHERE purpose = 'email_verification' AND hash = $1
    [ ] Si no existe o expirado → 400 "Token inválido o expirado"
    [ ] Si válido:
        [ ] UPDATE users SET is_active = true WHERE id = token.user_id
        [ ] DELETE tokens WHERE id = token.id (one-time use)
        [ ] audit.log(action: "email_verified", resource: "users", resource_id: user_id)
        [ ] Retorna 200 { "message": "Email verificado. Puedes iniciar sesión." }
```

### Frontend

```
[ ] src/routes/(auth)/verify/[token]/+page.svelte
    [ ] On mount: GET /auth/verify/{token} desde URL
    [ ] Estados:
        [ ] Verificando... (spinner)
        [ ] Éxito: ícono check + "Email verificado" + botón "Ir a login"
        [ ] Error: ícono alerta + "Link inválido o expirado" + botón "Reenviar email"
    [ ] Si error: opción para solicitar nuevo email de verificación
```

---

## A.3 — Login (POST /auth/login)

> **Referencia:** ADR 0008, ADR 0006, ADR 0009

### Backend

```
[ ] Handler: POST /auth/login
    [ ] Rate limiting: max 10 intentos / 5min por IP (ADR 0009)
    [ ] Busca user por email (case-insensitive)
    [ ] Si no existe → 401 genérico (no revelar si email existe)
    [ ] Si user no está verificado (is_active = false) → 403 "Verifica tu email primero"
    [ ] Si user está soft-deleted → 401 genérico
    [ ] verify_password(argon2id) con timing-safe comparison
    [ ] Si inválido → 401 genérico + audit.log(action: "login_failed", resource: "users")
    [ ] Si válido:
        [ ] PasetoService::generate_access_token(user_id, roles[], 15min) → "v4.local.xxx"
        [ ] generate_opaque_token() → refresh_token (32 bytes aleatorios)
        [ ] hash_token(refresh_token) → SHA-256
        [ ] sessions.create(user_id, refresh_hash, ip, ua, expires_at = 7 días)
        [ ] audit.log(action: "login_success", resource: "users", resource_id: user_id, ip, ua)
        [ ] Retorna 200 {
            "access_token": "v4.local.xxx",
            "refresh_token": "opaque_xxx",
            "expires_in": 900,
            "user": {
                "id": "usr_xxx",
                "email": "...",
                "name": "...",
                "roles": ["User"],
                "permissions": ["devices:read", "alerts:read", ...]
            }
        }
```

**Nota:** NUNCA retornar JWT (strings que empiecen con "eyJ"). Verificar en tests.

### Frontend

```
[ ] src/routes/(auth)/login/+page.svelte
    [ ] Layout: AuthLayout
    [ ] Componente: components/auth/LoginForm.svelte
    [ ] Campos: email, password
    [ ] ArkType validation: email formato, password no vacío
    [ ] TanStack mutation → POST /auth/login
    [ ] onSuccess:
        [ ] setAuth(user, access_token, refresh_token) en auth.svelte.ts
        [ ] Persistir tokens en localStorage (encriptado con subtle crypto)
        [ ] redirect /dashboard
    [ ] onError:
        [ ] 401 → "Credenciales incorrectas"
        [ ] 403 → "Verifica tu email antes de iniciar sesión"
        [ ] 429 → "Demasiados intentos. Espera 5 minutos."
        [ ] 500 → "Error del servidor. Intenta más tarde."
    [ ] Checkbox "Recordarme" → extender session expiry a 30 días
```

---

## A.4 — Sesión activa y protección de rutas

> **Referencia:** ADR 0008, ADR 0006, ADR 0003

### Backend

```
[ ] auth_middleware en todas las rutas /api/v1/* (excepto /health, /auth/*, /docs, /openapi.json)
    [ ] Extrae Authorization: Bearer <token>
    [ ] Si no hay token → 401
    [ ] Si token empieza con "eyJ" → 401 "JWT no soportado, usa PASETO"
    [ ] Verifica PASETO v4.local via pasetors
    [ ] Extrae claims: user_id, roles[], exp
    [ ] Si expirado → 401 "Token expirado"
    [ ] Busca user en DB (verificar no soft-deleted)
    [ ] Inyecta AuthClaims { user_id, roles, permissions } en request.extensions
    [ ] Si user no is_active → 403 "Cuenta desactivada"

[ ] rbac_middleware en rutas específicas (composición con auth_middleware)
    [ ] require_permission(permission: &str)
    [ ] Verifica que permissions[] de AuthClaims contenga el permiso
    [ ] Si no → 403 "Permiso requerido: {permission}"
    [ ] Cache de permisos en Moka (TTL 5min) para evitar consultas repetidas

[ ] Rutas protegidas con RBAC:
    [ ] GET /api/v1/devices → require_permission("devices:read")
    [ ] POST /api/v1/devices → require_permission("devices:write")
    [ ] PUT /api/v1/devices/:id/archive → require_permission("devices:delete")  ← soft delete
    [ ] GET /api/v1/alerts → require_permission("alerts:read")
    [ ] POST /api/v1/alerts/:id/acknowledge → require_permission("alerts:write")
    [ ] GET /api/v1/audit → require_permission("audit:read")
    [ ] GET /api/v1/admin/users → require_permission("user:read")
    [ ] POST /api/v1/admin/users → require_permission("user:write")
    [ ] GET /api/v1/agents → require_permission("agent:read")
    [ ] POST /api/v1/agents/:id/restart → require_permission("agent:write")
```

### Frontend

```
[ ] src/routes/(dashboard)/+layout.server.ts — SSR auth verification:
    [ ] Lee access_token desde cookies (httpOnly, secure, sameSite=strict)
    [ ] Si no hay cookie → set redirect a /login
    [ ] Si token inválido → set redirect a /login
    [ ] Inyecta user básico en locals para SSR rendering

[ ] src/routes/(dashboard)/+layout.svelte — cliente auth:
    [ ] auth.svelte.ts: estado desde localStorage al hidratar
    [ ] Si !isLoggedIn → goto /login (client-side redirect)
    [ ] $effect para verificar token expiry cada 60s
    [ ] Si token expira en < 2min → auto-refresh silencioso

[ ] components/auth/PermissionGate.svelte:
    [ ] Props: permission: string, fallback?: Snippet
    [ ] Lee user.permissions[] del auth store
    [ ] Si tiene permiso → renderiza children
    [ ] Si no → renderiza fallback o nada
    [ ] Uso: <PermissionGate permission="devices:write"><BotonCrear /></PermissionGate>

[ ] components/layout/Sidebar.svelte — filtrado por permisos:
    [ ] Cada NavItem verifica permission antes de renderizar
    [ ] Si no tiene permiso → no muestra el item (no link roto)
    [ ] Items protegidos:
        [ ] /dashboard/audit → "audit:read"
        [ ] /dashboard/admin/users → "user:read"
        [ ] /dashboard/agents → "agent:read"
```

---

## A.5 — Refresh de tokens

> **Referencia:** ADR 0008

### Backend

```
[ ] Handler: POST /auth/refresh
    [ ] Extrae refresh_token del body: { "refresh_token": "opaque_xxx" }
    [ ] hash_token(refresh_token) → SHA-256
    [ ] Busca en sessions WHERE token_hash = $1 AND expires_at > NOW()
    [ ] Si no existe o expirado → 401 "Sesión inválida"
    [ ] Busca user (verificar no soft-deleted, is_active = true)
    [ ] REVOCAR refresh anterior: DELETE sessions WHERE id = session.id
    [ ] Generar nuevo access_token (PASETO v4, 15min)
    [ ] Generar nuevo refresh_token (opaque, 32 bytes)
    [ ] hash nuevo refresh → SHA-256
    [ ] Crear nueva session con nuevo hash, ip, ua, expires_at = 7 días
    [ ] audit.log(action: "token_refreshed", resource: "users", resource_id: user_id)
    [ ] Retorna 200 {
        "access_token": "v4.local.xxx",
        "refresh_token": "nuevo_opaque_xxx",
        "expires_in": 900
    }

[ ] Seguridad:
    [ ] Un refresh_token solo se usa UNA VEZ (one-time rotation)
    [ ] Si se intenta reusar un refresh revocado → 401 + invalidate ALL sessions del user
    [ ] Detecta potencial token theft
```

### Frontend

```
[ ] src/lib/api/client.ts — interceptor de refresh:
    [ ] Wrapper alrededor de fetch:
        [ ] Intenta request normal
        [ ] Si 401 + refresh_token disponible:
            [ ] POST /auth/refresh con refresh_token
            [ ] Si exitoso: actualizar access_token + refresh_token en store y localStorage
            [ ] Reintentar request original con nuevo token
            [ ] Si falla refresh: clearAuth() + redirect /login
        [ ] Si 401 + NO refresh_token: clearAuth() + redirect /login
    [ ] Mutex: solo UNA petición de refresh a la vez (evitar race conditions)
    [ ] Queue: encolar requests que llegan durante refresh, reejecutarlas después

[ ] src/lib/stores/auth.svelte.ts — auto-refresh:
    [ ] $effect que verifica expiry cada 60s
    [ ] Si access_token expira en < 2min → trigger refresh silencioso
    [ ] Si refresh falla (red offline, etc.) → mostrar "Sesión expirará pronto" banner
```

---

## A.6 — Logout

> **Referencia:** ADR 0008, ADR 0006

### Backend

```
[ ] Handler: POST /auth/logout (requiere auth válido)
    [ ] Extrae access_token del header Authorization
    [ ] Verifica PASETO → extrae user_id
    [ ] Extrae refresh_token del body (opcional, para revocar específico)
    [ ] Si refresh_token proporcionado:
        [ ] hash_token() → buscar session
        [ ] DELETE sessions WHERE token_hash = $1
    [ ] Si no:
        [ ] DELETE sessions WHERE user_id = $1 AND ip_address = $2  ← revocar sesión actual
    [ ] audit.log(action: "logout", resource: "users", resource_id: user_id, ip, ua)
    [ ] Retorna 200 { "message": "Sesión cerrada" }

[ ] Handler opcional: POST /auth/logout-all
    [ ] Requiere auth + re-verificación de password (por seguridad)
    [ ] DELETE ALL sessions WHERE user_id = $1
    [ ] audit.log(action: "logout_all_sessions", resource: "users", resource_id: user_id)
    [ ] Retorna 200
```

### Frontend

```
[ ] components/layout/Topbar.svelte — botón logout:
    [ ] Dropdown de usuario → "Cerrar sesión"
    [ ] TanStack mutation → POST /auth/logout
    [ ] Incluye refresh_token en body para revocar específico
    [ ] onSettled (siempre, incluso si falla):
        [ ] clearAuth() → limpiar localStorage, cookies, store
        [ ] QueryClient.clear() → limpiar cache de TanStack Query
        [ ] redirect /login
    [ ] Si "Cerrar sesión en todos los dispositivos":
        [ ] POST /auth/logout-all
        [ ] Requiere confirmar password en modal
```

---

## A.7 — Password Reset

> **Referencia:** ADR 0008, ADR 0016

### Backend

```
[ ] Handler: POST /auth/forgot-password
    [ ] Rate limiting: max 3 intentos / hora por email (ADR 0009)
    [ ] Recibe { "email": "..." }
    [ ] Busca user por email (case-insensitive)
    [ ] Si no existe → retorna 200 genérico (no revelar si email existe)
    [ ] Si existe:
        [ ] Generar reset_token (opaque, 32 bytes, expira 1h)
        [ ] hash_token() → SHA-256 en tokens table (purpose: password_reset)
        [ ] Enviar email con link: /reset-password?token=xxx
        [ ] audit.log(action: "password_reset_requested", resource: "users", resource_id: user_id)
    [ ] Retorna 200 { "message": "Si el email existe, recibirás instrucciones" } (siempre 200)

[ ] Handler: POST /auth/reset-password
    [ ] Recibe { "token": "...", "new_password": "..." }
    [ ] hash_token(raw) → buscar en tokens WHERE purpose = 'password_reset'
    [ ] Si no existe o expirado → 400 "Token inválido o expirado"
    [ ] Validar new_password: 12+ chars, fuerza
    [ ] hash_password(new_password) con argon2id
    [ ] UPDATE users SET password_hash = $1 WHERE id = user_id
    [ ] DELETE tokens WHERE user_id = $1 AND purpose = 'password_reset'  ← invalidar todos los resets del user
    [ ] DELETE sessions WHERE user_id = $1  ← forzar re-login en todos los dispositivos
    [ ] audit.log(action: "password_reset_completed", resource: "users", resource_id: user_id)
    [ ] Retorna 200 { "message": "Password actualizado. Inicia sesión." }
```

### Frontend

```
[ ] src/routes/(auth)/forgot-password/+page.svelte
    [ ] Campo: email
    [ ] ArkType validation
    [ ] TanStack mutation → POST /auth/forgot-password
    [ ] onSuccess: toast "Revisa tu email" (incluso si email no existe)
    [ ] onError: mensaje genérico

[ ] src/routes/(auth)/reset-password/+page.svelte
    [ ] Lee token desde URL query param: ?token=xxx
    [ ] Si no hay token → redirect /forgot-password
    [ ] Campos: new_password, confirm_password
    [ ] Indicador de fuerza de password
    [ ] TanStack mutation → POST /auth/reset-password
    [ ] onSuccess: "Password actualizado" + redirect /login
    [ ] onError (400): "Link inválido o expirado" + botón para solicitar nuevo
```

---

## A.8 — RBAC en acción

> **Referencia:** ADR 0006, ADR 0008

### Backend

```
[ ] Middleware compuesto: auth + rbac
    [ ] auth_middleware: verifica PASETO, inyecta AuthClaims
    [ ] rbac_middleware(permission): verifica permiso en claims

[ ] Estrategia de permisos:
    [ ] Permissions almacenados como "recurso:acción"
    [ ] Recursos: users, devices, sedes, alerts, intrusions, agents, audit, topology, metrics
    [ ] Acciones: read, write, delete (soft), export, admin
    [ ] Roles predefinidos:
        [ ] Admin: todos los permisos
        [ ] Operator: read all + write alerts + write intrusions + read agents
        [ ] Viewer: read all (solo lectura)
        [ ] Agent: solo write metrics + read devices (para apps/agent)

[ ] Cache de permisos:
    [ ] Moka cache en rbac_middleware: key = "perms:{user_id}", TTL 5min
    [ ] Invalidar cache cuando:
        [ ] Se actualizan roles del user
        [ ] Se actualizan permisos de un rol
```

### Frontend

```
[ ] auth.svelte.ts — permisos:
    [ ] user.permissions[] cargado en login/refresh
    [ ] hasPermission(permission: string): boolean
    [ ] hasAnyPermission(permissions: string[]): boolean
    [ ] hasAllPermissions(permissions: string[]): boolean

[ ] PermissionGate.svelte:
    [ ] <PermissionGate permission="devices:write">
    [ ] <PermissionGate anyOf={["alerts:write", "intrusions:write"]}>
    [ ] <PermissionGate allOf={["user:read", "user:write"]}>

[ ] UI condicional:
    [ ] Botón "Crear dispositivo" → oculto si no "devices:write"
    [ ] Botón "Archivar" → oculto si no "devices:delete"
    [ ] Tab "Auditoría" en sidebar → oculto si no "audit:read"
    [ ] Botón "Exportar CSV" → oculto si no "audit:export"
    [ ] Panel Admin → oculto si no "admin:access"

[ ] Protección de rutas:
    [ ] +page.ts (load function): verificar permiso antes de renderizar
    [ ] Si no tiene permiso → redirect /dashboard + toast "Sin permiso"
```

---

## A.9 — Rate Limiting en auth

> **Referencia:** ADR 0009

```
[ ] Configuración de rate limiting por endpoint:
    [ ] POST /auth/register: 5 / 15min por IP
    [ ] POST /auth/login: 10 / 5min por IP
    [ ] POST /auth/forgot-password: 3 / 60min por email
    [ ] POST /auth/reset-password: 5 / 15min por IP
    [ ] POST /auth/refresh: 20 / 1min por IP (alto, es legítimo)
    [ ] POST /auth/logout: 10 / 1min por IP

[ ] Implementación:
    [ ] Tower middleware: tower_governor o custom Redis/Moka based
    [ ] Key: IP address (X-Forwarded-For si detrás de proxy)
    [ ] Headers de respuesta:
        [ ] X-RateLimit-Limit
        [ ] X-RateLimit-Remaining
        [ ] X-RateLimit-Reset
    [ ] 429 Too Many Requests con Retry-After header

[ ] Frontend:
    [ ] Mostrar countdown cuando recibe 429
    [ ] Deshabilitar botón de submit mientras en cooldown
    [ ] Mensaje: "Demasiados intentos. Espera {seconds} segundos."
```

---

## Verificación final

```bash
# === REGISTRO ===
curl -X POST http://localhost:8080/auth/register   -H "Content-Type: application/json"   -H "X-Request-Id: test-register-001"   -d '{"name":"Demo User","email":"demo@test.com","password":"Segura123!@#"}'
# → 201 { "id": "usr_xxx", "email": "demo@test.com", "message": "Verificación enviada" }

# === LOGIN (antes de verificar email) ===
curl -X POST http://localhost:8080/auth/login   -H "Content-Type: application/json"   -d '{"email":"demo@test.com","password":"Segura123!@#"}'
# → 403 { "error": "Verifica tu email primero" }

# === VERIFICAR EMAIL (simulado, token desde DB o email) ===
# curl GET /auth/verify/xxx
# → 200 { "message": "Email verificado" }

# === LOGIN (después de verificar) ===
TOKEN_RESPONSE=$(curl -s -X POST http://localhost:8080/auth/login   -H "Content-Type: application/json"   -d '{"email":"demo@test.com","password":"Segura123!@#"}')
ACCESS_TOKEN=$(echo $TOKEN_RESPONSE | jq -r '.access_token')
REFRESH_TOKEN=$(echo $TOKEN_RESPONSE | jq -r '.refresh_token')
echo $ACCESS_TOKEN | cut -c1-10
# → "v4.local.."  ← VERIFICAR: NO "eyJ" (JWT)

# === REQUEST AUTENTICADA ===
curl http://localhost:8080/api/v1/devices   -H "Authorization: Bearer $ACCESS_TOKEN"   -H "X-Request-Id: test-devices-001"
# → 200 [...devices]

# === SIN PERMISO ===
curl -X POST http://localhost:8080/api/v1/devices   -H "Authorization: Bearer $ACCESS_TOKEN"   -H "Content-Type: application/json"   -d '{"hostname":"test","ip":"192.168.1.1","mac":"00:11:22:33:44:55","type":"switch","sede_id":"sede_xxx"}'
# → 403 { "error": "Permiso requerido: devices:write" }

# === JWT RECHAZADO ===
curl http://localhost:8080/api/v1/devices   -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..."
# → 401 { "error": "JWT no soportado, usa PASETO" }

# === REFRESH ===
NEW_TOKENS=$(curl -s -X POST http://localhost:8080/auth/refresh   -H "Content-Type: application/json"   -d "{"refresh_token":"$REFRESH_TOKEN"}")
echo $NEW_TOKENS | jq -r '.access_token' | cut -c1-10
# → "v4.local.."

# === REUSAR REFRESH ANTIGUO (debe fallar) ===
curl -X POST http://localhost:8080/auth/refresh   -H "Content-Type: application/json"   -d "{"refresh_token":"$REFRESH_TOKEN"}"
# → 401 { "error": "Sesión inválida" }

# === LOGOUT ===
curl -X POST http://localhost:8080/auth/logout   -H "Authorization: Bearer $(echo $NEW_TOKENS | jq -r '.access_token')"   -H "Content-Type: application/json"   -d "{"refresh_token":"$(echo $NEW_TOKENS | jq -r '.refresh_token')"}"
# → 200

# === RATE LIMITING ===
for i in {1..6}; do
  curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:8080/auth/register     -H "Content-Type: application/json"     -d '{"email":"test'$i'@test.com","password":"Segura123!@#"}'
done
# → 201 201 201 201 201 429
```

---

## Troubleshooting

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Login siempre 401 | Password hash incorrecto | Verificar argon2id params, verificar verify_password |
| access_token empieza con "eyJ" | Usar PASETO, no JWT | Verificar PasetoService::generate, rechazar JWT en middleware |
| Permisos no cargados | Login response no incluye permissions | Verificar backend incluye permissions[] en response |
| RBAC no funciona | Cache Moka desincronizado | Invalidar cache al actualizar roles/permisos |
| Refresh token reusado funciona | No se revocó el anterior | Verificar DELETE sessions en refresh handler |
| Rate limiting no aplica | Middleware no configurado | Verificar Tower middleware en router |
| Email de verificación no llega | Resend no configurado | Verificar RESEND_API_KEY y MAIL_FROM en .env |
| Password reset link no funciona | Token expirado o mal hasheado | Verificar hash_token() usa SHA-256 consistente |
| Sesión no persiste entre tabs | localStorage no compartido | Verificar auth store usa localStorage (no sessionStorage) |
| Auto-refresh no funciona | $effect no verifica expiry | Verificar lógica de refresh silencioso en auth.svelte.ts |
| Logout en un tab no cierra otros | No se usa logout-all | Implementar logout-all + broadcast channel |

---

**Nota:** Este roadmap está basado en el ADR 0020 (Módulo de Monitoreo de Infraestructura Regional) para la Gobernación del Beni. La seguridad auth (ADR 0008) es crítica: PASETO v4 obligatorio, JWT prohibido, argon2id para passwords, soft delete para usuarios.
