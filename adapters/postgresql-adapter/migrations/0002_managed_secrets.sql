CREATE TABLE managed_secrets (
    id CHAR(36) NOT NULL PRIMARY KEY,
    payload BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_pulled_at TIMESTAMPTZ,
    auth_token_hash TEXT
);
