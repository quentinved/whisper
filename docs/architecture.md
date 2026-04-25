# Architecture

Whisper follows a **hexagonal (Ports & Adapters)** layout, splitting domain logic from I/O. Domain code is pure Rust with no knowledge of HTTP, databases, or encryption libraries. Adapters at the boundary translate between external systems and domain types.

## Layout

```
whisper/
├── services-core/                  # Domain: entities, value objects, commands, traits
├── adapters/
│   ├── postgresql-adapter/         # PostgreSQL repository (implements SharedSecretRepository)
│   └── aes-gcm-crypto/             # AES-256-GCM encryption (implements SecretEncryption)
└── applications/
    ├── axum/server/                # Axum HTTP server: routes, templates, Slack integration
    ├── cli/                        # whisper-secrets CLI
    └── discord/                    # Discord bot
```

## Crate responsibilities

### `whisper-core` (`services-core/`)
Pure domain logic. No async runtime, no I/O, no third-party services.

- **Entities** — `SharedSecret` (id + encrypted blob + expiration + self-destruct flag)
- **Value objects** — `SecretId` (UUID), `SecretEncrypted` (cypher + nonce), `SecretExpiration` (validated future timestamp)
- **Contracts (ports)** — `SharedSecretRepository` (persistence trait), `SecretEncryption` (encrypt/decrypt trait)
- **Commands** — `CreateSecret`, `GetSecretById`, `DeleteExpiredSecrets`

### `whisper-crypto` (`adapters/aes-gcm-crypto/`)
Implements `SecretEncryption` using AES-256-GCM with a random 12-byte nonce per secret. The cipher is built once at startup and reused.

### `whisper-postgresql` (`adapters/postgresql-adapter/`)
Implements `SharedSecretRepository` using SQLx (async, compile-time-checked queries). Migrations applied on startup. Atomic operations use `DELETE ... RETURNING` / `UPDATE ... RETURNING` to avoid race conditions.

### `whisper-server` (`applications/axum/server/`)
Axum HTTP application. Wires up the AES key, DB pool, and cron scheduler at startup. Handles:

- Web routes (HTML pages via Askama templates)
- REST API for ephemeral and managed secrets
- Slack `/whisper` slash command (HMAC-SHA256 signature verification)
- Static asset serving (CSS / SVG embedded via rust-embed)
- Security headers middleware

### `whisper-secrets` (`applications/cli/`)
The CLI. Adds **client-side** encryption on top of the server's at-rest encryption: a passphrase derives a key via PBKDF2-SHA256 (600,000 iterations), and secrets are encrypted before they ever touch the network. This is what makes the CLI **zero-knowledge** — the server only sees ciphertext.

### `whisper-discord-bot` (`applications/discord/`)
TypeScript / Bun. Discord slash command counterpart to the Slack `/whisper` command.

## Routes

| Method | Path | Purpose |
|--------|------|---------|
| GET    | `/` | Create-secret form (HTML) |
| POST   | `/secret` | Create ephemeral secret (form submit) |
| GET    | `/secret/:id` | Retrieve ephemeral secret (JSON) |
| GET    | `/get_secret?shared_secret_id=UUID` | Retrieve secret page (HTML) |
| GET    | `/health` | Health check |
| GET    | `/privacy` / `/terms` / `/integrations` | Static pages |
| GET    | `/contact` | Redirects to mailto: |
| GET    | `/assets/*path` | Static assets (rust-embed) |
| POST   | `/slack/whisper` | Slack slash command (signature middleware) |
| PUT    | `/v1/secrets/:id` | Upsert managed secret (CLI) |
| GET    | `/v1/secrets/:id` | Get managed secret (CLI) |
| DELETE | `/v1/secrets/:id` | Delete managed secret (CLI) |

## Storage

Single PostgreSQL table:

```
secrets (
  id            TEXT PRIMARY KEY,   -- UUID v4
  cypher        BYTEA,              -- AES-256-GCM ciphertext
  nonce         BYTEA,              -- 12-byte random nonce
  expiration    TIMESTAMPTZ,
  self_destruct BOOL
)
```

A cron job (`tokio-cron-scheduler`) deletes expired rows once per minute.

## Security model

- **Web app / Slack / Discord** — encryption *at rest* with the server's AES key. The server can decrypt; if the server or DB is compromised, secrets leak. URL secrecy and HTTPS are the access control.
- **CLI managed secrets** — encryption *client-side* before upload. The server never sees the passphrase or plaintext. Compromising the server reveals only ciphertext.

The two layers are independent and stack: a managed-secret upload is encrypted by the CLI, then the ciphertext is encrypted again by the server's AES key.

## Key design decisions

- **AES-256-GCM** for authenticated encryption (cypher + nonce stored together)
- **UUID v4** for secret IDs (unguessable)
- **Parameterized SQL** via SQLx (SQL injection prevention)
- **Askama** templates with auto-escaping (XSS prevention)
- **Generic errors to clients**, real errors logged server-side only
- **No authentication** for ephemeral secrets — URL secrecy + HTTPS are the security boundary
- **PBKDF2-SHA256** at 600,000 iterations for CLI key derivation (matches OWASP 2023 minimum)

## Adding new functionality

- **New CLI command** → `applications/cli/src/commands/<name>.rs` + register in `main.rs`
- **New HTTP route** → `applications/axum/server/src/router/`
- **New domain type** → `services-core/src/values_object/` or `entities/`
- **New persistence backend** → new crate under `adapters/` implementing `SharedSecretRepository`
- **New encryption backend** → new crate under `adapters/` implementing `SecretEncryption`

Domain types stay pure Rust — adapters translate at the boundary.
