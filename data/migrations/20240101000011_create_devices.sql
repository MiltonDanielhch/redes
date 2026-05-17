-- Ubicación: `data/migrations/011_create_devices.sql`
--
-- Descripción: Tabla devices para inventario de red
--
-- ADRs relacionados: 0004, 0020

CREATE TYPE device_type AS ENUM ('Switch', 'AccessPoint', 'Router', 'Firewall', 'Server', 'Ups', 'Camera', 'WirelessLink');
CREATE TYPE device_status AS ENUM ('Active', 'Offline', 'Maintenance');

CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hostname VARCHAR(255) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    mac_address VARCHAR(17),
    device_type device_type NOT NULL,
    status device_status NOT NULL DEFAULT 'Offline',
    sede_id UUID NOT NULL REFERENCES sedes(id) ON DELETE RESTRICT,
    last_seen_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
);

CREATE UNIQUE INDEX idx_devices_hostname ON devices (hostname) WHERE deleted_at IS NULL;
CREATE INDEX idx_devices_ip ON devices (ip_address) WHERE deleted_at IS NULL;
CREATE INDEX idx_devices_sede ON devices (sede_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_devices_type ON devices (device_type) WHERE deleted_at IS NULL;

CREATE TRIGGER trg_devices_updated_at
    BEFORE UPDATE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION trg_set_updated_at();