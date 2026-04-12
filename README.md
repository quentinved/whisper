# Whisper

Secure, zero-knowledge secret management. AES-256-GCM encrypted, auto-expiring, self-destruct after first read. CLI + Web app + Integrations. Built with Rust.

## Install

**Shell (macOS Apple Silicon / Linux)**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/quentinved/Whisper/main/install.sh | sh
```

**npm**
```bash
npm install -g whisper-secrets
```

**Binary download**

Pre-built binaries for Linux (x64, arm64), macOS (Apple Silicon only — Intel Mac not supported), and Windows (x64) are available on the [Releases](https://github.com/quentinved/Whisper/releases/latest) page.

## Quick Start

```bash
# 1. Initialize a project (generates passphrase + share link for your team)
whisper-secrets init

# 2. Import your existing .env or push secrets one by one
whisper-secrets import                  # encrypt & upload every entry from .env
whisper-secrets push STRIPE_SECRET_KEY  # or add secrets individually

# 3. Commit .env.whisper to git (contains only UUIDs, no secrets)

# 4. Teammates clone the repo, cd into it, install the CLI, and join:
git clone <repo> && cd <repo>
npm install -g whisper-secrets
whisper-secrets join <link-from-teammate>  # creates .whisperrc + auto-pulls
```

## CLI Commands

### Managed Secrets (`.env` workflow)

```bash
whisper-secrets init                          # set up a project
whisper-secrets init --url https://your.host  # use your own server
whisper-secrets import                        # upload existing .env
whisper-secrets push SECRET_NAME              # encrypt & upload one secret
whisper-secrets push                          # pick untracked .env entries interactively
whisper-secrets pull                          # download & decrypt to .env
whisper-secrets rotate SECRET_NAME            # update a secret in-place
whisper-secrets remove SECRET_NAME            # delete a secret
whisper-secrets status                        # show tracked, missing, and untracked secrets
```

### Team Collaboration

```bash
whisper-secrets invite                        # generate a new share link for a teammate
whisper-secrets join <link>                   # join a project (auto-pulls if .env.whisper is present)
```

### Ephemeral Secrets (one-time sharing)

```bash
whisper-secrets share                                # 1h, self-destruct
whisper-secrets share -e 24h                         # custom expiration
whisper-secrets share -e 7d --no-self-destruct       # keep after first view
whisper-secrets get https://whisper.example.com/...  # retrieve by URL or ID
```

> **Tip:** If installed via npm or the shell installer, `ws` is available as a shortcut for `whisper-secrets`.

## Features

- **Zero-knowledge CLI**: Secrets encrypted on your machine before anything touches the network
- **AES-256-GCM Encryption**: Industry-standard authenticated encryption with unique nonce per secret
- **PBKDF2-SHA256 Key Derivation**: 600,000 iterations — passphrase never leaves your machine
- **Flexible Expiration**: 6 hours, 1 day, 2 days, or custom (up to 7 days)
- **Self-Destruct**: Optional one-time access — secrets deleted after first retrieval
- **Integrations**: Slack, Discord, Raycast
- **Self-hostable**: Deploy the server anywhere, point `--url` at your instance

## How It Works

1. `whisper-secrets init` generates a random passphrase and creates `.whisperrc`
2. The passphrase derives an encryption key (PBKDF2-SHA256) and an auth token
3. `push` / `import` encrypt secrets client-side, then upload ciphertext to the server
4. `pull` downloads ciphertext and decrypts locally
5. The server only stores encrypted blobs — zero knowledge of your secrets

**Files:**
- `.whisperrc` — project config (URL + passphrase). Add to `.gitignore`
- `.env.whisper` — maps secret names to server IDs. Commit to git
- `.env` — plaintext secrets from `pull`. Add to `.gitignore`

## Security

### Encryption
- **AES-256-GCM**: Authenticated encryption for secret storage
- **Unique Nonces**: Random nonce per secret
- **Key Management**: AES key stored separately from the application

### Data Protection
- **No Plaintext Storage**: Secrets are never stored in plaintext
- **Automatic Cleanup**: Expired secrets deleted every minute
- **Self-Destruct**: Optional one-time access for sensitive secrets

### Security Headers
- HSTS (Strict-Transport-Security)
- Content-Security-Policy
- X-Frame-Options: DENY
- X-Content-Type-Options: nosniff

## Self-Hosting

### Prerequisites

- Rust 1.70+
- PostgreSQL 16+
- Docker & Docker Compose (optional)

### Using Docker Compose

```bash
# Start the database
docker-compose up -d

# Create the database
docker exec -it postgres psql -U postgres -c "CREATE DATABASE whisper;"

# Generate AES key (only needed once)
openssl rand -out aes_key.bin 32

# Run the server
cargo run --bin whisper-server -- --url-posgtresql "postgres://postgres:toto@localhost/whisper"
```

> Database migrations are applied automatically on startup.

### Configuration

| Variable | CLI Flag | Default | Description |
|----------|----------|---------|-------------|
| `PORT` | `--port` | `1212` | TCP port |
| `LISTEN_ADDR` | `--listen-addr` | `127.0.0.1` | Bind address |
| `DATABASE_URL` | `--url-posgtresql` | required | PostgreSQL connection URL |
| `AES_KEY_PATH` | `--aes-key-path` | `aes_key.bin` | Path to 32-byte AES key file |
| `BASE_URL` | `--base-url` | — | Public URL for share links |
| `SLACK_SIGNING_SECRET` | `--slack-signing-secret` | — | Enables Slack integration |

## Architecture

Hexagonal Architecture with clear separation between:

- **Core Domain** (`services-core/`): Business logic and entities
- **Adapters** (`adapters/`): External concerns (database, encryption)
- **Applications** (`applications/`): Web interface, CLI, and bots

```
whisper/
├── services-core/              # Core domain (whisper-core)
├── adapters/
│   ├── postgresql-adapter/     # PostgreSQL repository
│   └── aes-gcm-crypto/        # AES-256-GCM encryption
├── applications/
│   ├── axum/server/            # Axum HTTP server
│   ├── cli/                    # CLI (whisper-secrets)
│   └── discord/                # Discord bot
└── docker-compose.yml
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Create secret form (HTML) |
| `POST` | `/secret` | Create a new ephemeral secret |
| `GET` | `/secret/:id` | Retrieve a secret (JSON API) |
| `GET` | `/get_secret?shared_secret_id=UUID` | Retrieve secret page (HTML) |
| `GET` | `/health` | Health check |
| `POST` | `/slack/whisper` | Slack slash command |
| `PUT` | `/v1/secrets/:id` | Upsert managed secret |
| `GET` | `/v1/secrets/:id` | Get managed secret |
| `DELETE` | `/v1/secrets/:id` | Delete managed secret |

## Development

```bash
# Run all tests
cargo test --workspace

# Lint & format
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
