-- Ubicación: `data/migrations/002_create_rbac.sql`
--
-- Descripción: Tablas RBAC - roles + permissions + role_permissions
--
-- ADRs relacionados: 0004, 0006

-- =====================================================
-- Tabla: permissions
-- =====================================================
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- =====================================================
-- Tabla: role_permissions (N:M)
-- =====================================================
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    granted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, permission_id)
);

CREATE INDEX idx_role_permissions_role ON role_permissions (role_id);
CREATE INDEX idx_role_permissions_permission ON role_permissions (permission_id);

-- =====================================================
-- Permissions predefinidas (formato: recurso:accion)
-- =====================================================
INSERT INTO permissions (name, description) VALUES
    ('users:read', 'Ver usuarios'),
    ('users:create', 'Crear usuarios'),
    ('users:update', 'Actualizar usuarios'),
    ('users:delete', 'Eliminar usuarios'),
    ('roles:read', 'Ver roles'),
    ('roles:manage', 'Gestionar roles'),
    ('sedes:read', 'Ver sedes'),
    ('sedes:manage', 'Gestionar sedes'),
    ('devices:read', 'Ver dispositivos'),
    ('devices:manage', 'Gestionar dispositivos'),
    ('alerts:read', 'Ver alertas'),
    ('alerts:manage', 'Gestionar alertas'),
    ('metrics:read', 'Ver métricas'),
    ('audit:read', 'Ver auditoría'),
    ('settings:manage', 'Gestionar configuración')
ON CONFLICT (name) DO NOTHING;

-- =====================================================
-- Roles predefinidos
-- =====================================================
INSERT INTO roles (name, description) VALUES
    ('admin', 'Administrador del sistema'),
    ('operator', 'Operador de monitoreo'),
    ('viewer', 'Solo lectura')
ON CONFLICT (name) DO NOTHING;

-- =====================================================
-- Asignar permissions a admin (todos)
-- =====================================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin'
ON CONFLICT DO NOTHING;

-- =====================================================
-- Asignar permissions a operator
-- =====================================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'operator'
AND p.name IN (
    'users:read', 'sedes:read', 'sedes:manage',
    'devices:read', 'devices:manage',
    'alerts:read', 'alerts:manage',
    'metrics:read'
)
ON CONFLICT DO NOTHING;

-- =====================================================
-- Asignar permissions a viewer
-- =====================================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'viewer'
AND p.name IN (
    'users:read', 'sedes:read',
    'devices:read',
    'alerts:read',
    'metrics:read'
)
ON CONFLICT DO NOTHING;