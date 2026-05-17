-- Ubicación: `data/migrations/010_create_sedes.sql`
--
-- Descripción: Tabla sedes para estructura regional
--
-- ADRs relacionados: 0004, 0020

CREATE TABLE IF NOT EXISTS sedes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre VARCHAR(255) NOT NULL,
    ubicacion VARCHAR(500) NOT NULL,
    secretaria VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
);

CREATE UNIQUE INDEX idx_sedes_nombre ON sedes (nombre) WHERE deleted_at IS NULL;

CREATE TRIGGER trg_sedes_updated_at
    BEFORE UPDATE ON sedes
    FOR EACH ROW
    EXECUTE FUNCTION trg_set_updated_at();

-- Sedes predefinidas del Beni
INSERT INTO sedes (nombre, ubicacion, secretaria) VALUES
    ('Sede Central', 'Riberalta', 'Gobernación'),
    (' Palacio Beni', 'Riberalta', 'Gobernación'),
    ('Sec. Gestion Territorial', 'Riberalta', 'Secretaría'),
    ('Sec. Administracion', 'Riberalta', 'Secretaría'),
    ('Sec. Servicios Sociales', 'Riberalta', 'Secretaría'),
    ('Sec. Economia', 'Riberalta', 'Secretaría'),
    ('Sec. Desarrollo Productivo', 'Riberalta', 'Secretaría'),
    ('Sec. Infraestructura', 'Riberalta', 'Secretaría')
ON CONFLICT DO NOTHING;