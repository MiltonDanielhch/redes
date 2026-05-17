-- Ubicación: `data/migrations/20240101000006_seed_system_data.sql`
--
-- Descripción: Seed data - usuario admin inicial
--
-- ADRs relacionados: 0004, 0006, 0008
--
-- ADVERTENCIA: Cambiar password en producción!

-- Admin inicial (password: admin123 - MUST CHANGE)
-- Hash generado con argon2 simple (cambiar en producción)
INSERT INTO users (email, password_hash, name) VALUES
    ('admin@redes.gob.bo', '$argon2id$v=19$m=19456,t=2,p=1$YWRtaW4xMjM0NTY3ODkw$qIljX5KQ9P8Z7H2mN1xT4Q', 'Administrador del Sistema')
ON CONFLICT (email) DO NOTHING;

-- Asignar rol admin al usuario admin
WITH admin_user AS (
    SELECT id FROM users WHERE email = 'admin@redes.gob.bo'
),
admin_role AS (
    SELECT id FROM roles WHERE name = 'admin'
)
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM admin_user u, admin_role r
WHERE NOT EXISTS (
    SELECT 1 FROM user_roles ur WHERE ur.user_id = u.id AND ur.role_id = r.id
);
