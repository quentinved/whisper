-- 0001_init.sql
CREATE TABLE secrets (
    id CHAR(36) NOT NULL PRIMARY KEY,
    cypher BYTEA NOT NULL,
    nonce BYTEA NOT NULL,
    expiration TIMESTAMPTZ NOT NULL,
    self_destruct BOOLEAN NOT NULL
);
--
