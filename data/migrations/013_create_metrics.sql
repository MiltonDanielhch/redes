-- Ubicación: `data/migrations/013_create_metrics.sql`
--
-- Descripción: Tabla metric_readings para métricas de monitoreo
--
-- ADRs relacionados: 0004, 0020

CREATE TABLE IF NOT EXISTS metric_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    bandwidth_rx_bytes BIGINT NOT NULL DEFAULT 0,
    bandwidth_tx_bytes BIGINT NOT NULL DEFAULT 0,
    latency_ms INTEGER,
    packet_loss_percent DOUBLE PRECISION,
    anomaly_detected BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_metrics_device_time ON metric_readings (device_id, created_at DESC);
CREATE INDEX idx_metrics_anomaly ON metric_readings (anomaly_detected, created_at DESC) WHERE anomaly_detected = true;

-- Partitioning por tiempo (opcional para alto volumen)
-- CREATE INDEX idx_metrics_recent ON metric_readings (created_at DESC);