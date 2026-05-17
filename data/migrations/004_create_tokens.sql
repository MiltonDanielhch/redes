-- Ubicación: `data/migrations/004_create_tokens.sql`
--
-- Descripción: Tabla tokens para email_verification y password_reset
--
-- ADRs relacionados: 0004, 0008

CREATE TYPE token_purpose AS ENUM ('email_verification', 'password_reset');

CREATE TABLE IF NOT EXISTS tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    purpose token_purpose NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tokens_user_purpose ON tokens (user_id, purpose);
CREATE INDEX idx_tokens_expiry ON tokens (expires_at);