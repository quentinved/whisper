-- Delivery receipts: when a tracked secret was first revealed. No foreign key
-- on purpose: the secret row is deleted on a self-destruct read, the receipt
-- must outlive it. Purged 7 days after the secret's expiration.
CREATE TABLE secret_receipts (
    status_token TEXT PRIMARY KEY,
    secret_id TEXT NOT NULL UNIQUE,
    expiration TIMESTAMPTZ NOT NULL,
    opened_at TIMESTAMPTZ NULL
);

CREATE INDEX secret_receipts_expiration_idx ON secret_receipts (expiration);
