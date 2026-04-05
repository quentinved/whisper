# Whisper

Secure, ephemeral secret sharing. AES-256-GCM encrypted, auto-expiring, self-destruct after first read. Web app + CLI. Built with Rust/Axum.

## Features

- **AES-256-GCM Encryption**: Secrets encrypted at rest with unique nonce per secret
- **Flexible Expiration**: 6 hours, 1 day, 2 days, or custom (up to 7 days)
- **Self-Destruct**: Optional one-time access — secrets deleted after first retrieval
- **CLI**: Create and retrieve secrets from the command line
- **Slack Integration**: `/whisper` slash command with HMAC-SHA256 signature verification
- **Managed Secrets API**: Persistent encrypted secrets with bearer token authentication
- **PostgreSQL Backend**: Reliable storage with automatic migrations
- **Auto-Cleanup**: Cron job removes expired secrets every minute

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 16+
- Docker & Docker Compose (optional)

### Using Docker Compose (Recommended)

1. **Start the database**
   ```bash
   docker-compose up -d
   ```

2. **Create the database**
   ```bash
   docker exec -it postgres psql -U postgres -c "CREATE DATABASE whisper;"
   ```

3. **Generate AES key** (only needed once)
   ```bash
   openssl rand -out aes_key.bin 32
   ```

4. **Run the application**
   ```bash
   cargo run --bin whisper-server -- --url-posgtresql "postgres://postgres:toto@localhost/whisper"
   ```
   > Database migrations are applied automatically on startup.

5. **Access the application**
   Open [http://localhost:1212](http://localhost:1212) in your browser

### Manual Setup

1. **Setup PostgreSQL**
   ```bash
   createdb whisper
   ```

2. **Generate AES key** (only needed once)
   ```bash
   openssl rand -out aes_key.bin 32
   ```

3. **Run the application**
   ```bash
   cargo run --bin whisper-server -- --url-posgtresql "postgres://user:pass@localhost/whisper"
   ```

## Architecture

This project follows **Hexagonal Architecture** principles with clear separation between:

- **Core Domain** (`services-core/`): Business logic and entities
- **Adapters** (`adapters/`): External concerns (database, encryption)
- **Applications** (`applications/`): Web interface and API

### Project Structure

```
whisper/
├── services-core/              # Core domain (whisper-core)
│   ├── entities/               # Domain entities
│   ├── values_object/          # Value objects (SecretId, SecretExpiration, etc.)
│   ├── commands/               # Use cases
│   ├── contracts/              # Repository & service traits
│   └── services/               # Domain services
├── adapters/
│   ├── postgresql-adapter/     # PostgreSQL repository (whisper-postgresql)
│   └── aes-gcm-crypto/        # AES-256-GCM encryption (whisper-crypto)
├── applications/
│   ├── axum/server/            # Axum HTTP server (whisper-server)
│   ├── cli/                    # CLI client (whisper-secrets)
│   └── discord/                # Discord bot (TypeScript/Bun)
└── docker-compose.yml
```

## Configuration

```bash
cargo run --bin whisper-server --help
```

| Variable | CLI Flag | Default | Description |
|----------|----------|---------|-------------|
| `PORT` | `--port` | `1212` | TCP port |
| `LISTEN_ADDR` | `--listen-addr` | `127.0.0.1` | Bind address |
| `DATABASE_URL` | `--url-posgtresql` | required | PostgreSQL connection URL |
| `AES_KEY_PATH` | `--aes-key-path` | `aes_key.bin` | Path to 32-byte AES key file |
| `BASE_URL` | `--base-url` | — | Public URL for share links |
| `SLACK_SIGNING_SECRET` | `--slack-signing-secret` | — | Enables Slack integration |

## Security Features

### Encryption
- **AES-256-GCM**: Industry-standard authenticated encryption for secret storage
- **Unique Nonces**: Each secret uses a unique random nonce for encryption
- **Key Management**: AES key stored separately from the application

### Data Protection
- **No Plaintext Storage**: Secrets are never stored in plaintext
- **Automatic Cleanup**: Expired secrets are automatically deleted
- **Self-Destruct**: Optional one-time access for sensitive secrets

### Security Headers
- HSTS (Strict-Transport-Security)
- Content-Security-Policy
- X-Frame-Options: DENY
- X-Content-Type-Options: nosniff

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Create secret form (HTML) |
| `POST` | `/secret` | Create a new ephemeral secret |
| `GET` | `/secret/:id` | Retrieve a secret (JSON API) |
| `GET` | `/get_secret?shared_secret_id=UUID` | Retrieve secret page (HTML) |
| `GET` | `/health` | Health check |
| `GET` | `/privacy` | Privacy policy |
| `GET` | `/terms` | Terms of service |
| `GET` | `/integrations` | Integrations page |
| `POST` | `/slack/whisper` | Slack slash command |
| `PUT` | `/v1/secrets/:id` | Upsert managed secret |
| `GET` | `/v1/secrets/:id` | Get managed secret |
| `DELETE` | `/v1/secrets/:id` | Delete managed secret |

## CLI

Zero-knowledge `.env` secret manager. Encrypt, store, and share secrets with your team — no signup, no accounts.

### Managed Secrets (`.env` workflow)

```bash
# Initialize a project (generates passphrase + .whisperrc config)
whisper-secrets init
whisper-secrets init --url https://your-server.com

# Import an existing .env file (encrypts & uploads every entry)
whisper-secrets import

# Push a single secret
whisper-secrets push DATABASE_URL

# Pull all secrets into .env
whisper-secrets pull

# Rotate a secret's value
whisper-secrets rotate DATABASE_URL

# Delete a secret
whisper-secrets remove DATABASE_URL
```

### Ephemeral Secrets (one-time sharing)

```bash
# Share a secret (default: 1h expiration, self-destruct on)
whisper-secrets share --expiration 1h

# Share without self-destruct
whisper-secrets share --expiration 24h --no-self-destruct

# Retrieve a secret by URL or ID
whisper-secrets get https://whisper.example.com/get_secret?shared_secret_id=UUID
```

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific component
cargo test -p whisper-core
cargo test -p whisper-crypto

# Lint & format
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

### Database Migrations

Migrations are automatically applied on startup. Migration files are located in `adapters/postgresql-adapter/migrations/`.

## Dependencies

### Core
- **Axum**: Modern web framework for Rust
- **SQLx**: Async SQL toolkit with compile-time checked queries
- **AES-GCM**: Authenticated encryption
- **Askama**: Type-safe template engine
- **tower-http**: Security headers middleware

### Development
- **Clap**: Command-line argument parsing with env var support
- **Tracing**: Structured logging
- **Tokio**: Async runtime
- **Chrono**: Date and time handling

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.