-- Ubicación: `data/migrations/015_create_intrusions.sql`
--
-- Descripción: Tabla intrusion_events para detección de intrusiones
--
-- ADRs relacionados: 0004, 0020

CREATE TYPE intrusion_status AS ENUM ('Detected', 'Investigating', 'Resolved', 'FalsePositive');

CREATE TABLE IF NOT EXISTS intrusion_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mac_address VARCHAR(17) NOT NULL,
    ip_address VARCHAR(45),
    device_id UUID REFERENCES devices(id) ON DELETE SET NULL,
    status intrusion_status NOT NULL DEFAULT 'Detected',
    detected_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP WITH TIME ZONE,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_intrusions_mac ON intrusion_events (mac_address);
CREATE INDEX idx_intrusions_device ON intrusion_events (device_id) WHERE device_id IS NOT NULL;
CREATE INDEX idx_intrusions_status ON intrusion_events (status, detected_at DESC);

CREATE TRIGGER trg_intrusions_updated_at
    BEFORE UPDATE ON intrusion_events
    FOR EACH ROW
    EXECUTE FUNCTION trg_set_updated_at();