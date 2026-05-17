-- Ubicación: `data/migrations/012_create_device_links.sql`
--
-- Descripción: Tabla device_links para conexiones entre dispositivos
--
-- ADRs relacionados: 0004, 0020

CREATE TYPE link_type AS ENUM ('Ethernet', 'Fiber', 'Wireless', 'Serial');

CREATE TABLE IF NOT EXISTS device_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_device_id UUID NOT NULL REFERENCES devices(id) ON DELETE RESTRICT,
    target_device_id UUID NOT NULL REFERENCES devices(id) ON DELETE RESTRICT,
    link_type link_type NOT NULL DEFAULT 'Ethernet',
    bandwidth_mbps INTEGER,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_different_devices CHECK (source_device_id != target_device_id)
);

CREATE INDEX idx_device_links_source ON device_links (source_device_id);
CREATE INDEX idx_device_links_target ON device_links (target_device_id);