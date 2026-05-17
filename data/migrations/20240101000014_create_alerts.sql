-- Ubicación: `data/migrations/014_create_alerts.sql`
--
-- Descripción: Tabla alerts para notificaciones de monitoreo
--
-- ADRs relacionados: 0004, 0020

CREATE TYPE alert_type AS ENUM ('DeviceOffline', 'BandwidthSaturation', 'PacketLoss', 'Intrusion', 'TopologyChange', 'HighTraffic');
CREATE TYPE alert_severity AS ENUM ('Critical', 'High', 'Medium', 'Low');
CREATE TYPE alert_status AS ENUM ('Active', 'Acknowledged', 'Resolved');

CREATE TABLE IF NOT EXISTS alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type alert_type NOT NULL,
    severity alert_severity NOT NULL,
    device_id UUID REFERENCES devices(id) ON DELETE SET NULL,
    message TEXT NOT NULL,
    details JSONB,
    status alert_status NOT NULL DEFAULT 'Active',
    acknowledged_by UUID REFERENCES users(id) ON DELETE SET NULL,
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alerts_status ON alerts (status, created_at DESC);
CREATE INDEX idx_alerts_device ON alerts (device_id) WHERE device_id IS NOT NULL;
CREATE INDEX idx_alerts_severity ON alerts (severity, created_at DESC);
CREATE INDEX idx_alerts_type ON alerts (alert_type, created_at DESC);

CREATE TRIGGER trg_alerts_updated_at
    BEFORE UPDATE ON alerts
    FOR EACH ROW
    EXECUTE FUNCTION trg_set_updated_at();