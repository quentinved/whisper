# Whisper

[![CI](https://github.com/quentinved/Whisper/actions/workflows/whisper-ci.yaml/badge.svg)](https://github.com/quentinved/Whisper/actions/workflows/whisper-ci.yaml)
[![npm version](https://img.shields.io/npm/v/whisper-secrets?logo=npm)](https://www.npmjs.com/package/whisper-secrets)
[![npm downloads](https://img.shields.io/npm/dm/whisper-secrets?logo=npm)](https://www.npmjs.com/package/whisper-secrets)
[![GitHub release](https://img.shields.io/github/v/release/quentinved/Whisper?logo=github)](https://github.com/quentinved/Whisper/releases/latest)
[![License: MIT](https://img.shields.io/github/license/quentinved/Whisper)](LICENSE)
[![Slack Community](https://img.shields.io/badge/Community-Slack-4A154B?logo=slack&logoColor=white)](https://join.slack.com/t/whisper-secrets/shared_invite/zt-3vtg200cj-tlOebOVAceFKeqfPbK9WuQ)

Whisper is a zero-knowledge secret manager for developers and teams. Encrypt your `.env` on your machine and sync it with your team, or send a one-time secret via a short-lived URL — without a server that can read your data. Use the hosted instance at [whisper.quentinvedrenne.com](https://whisper.quentinvedrenne.com), deploy your own, or run the CLI standalone.

## Two ways to use Whisper

### 1. One-time secret sharing

Send a password, API key, or any secret via a URL that auto-expires and optionally self-destructs after first read. Use the hosted [web app](https://whisper.quentinvedrenne.com), the [Slack / Discord integrations](https://whisper.quentinvedrenne.com/integrations), or the CLI:

```bash
whisper-secrets share              # 1h expiration, self-destructs on read
whisper-secrets share -e 24h       # custom expiration (up to 7d)
whisper-secrets get <url-or-id>    # retrieve someone else's secret
```

### 2. Team `.env` management

Manage your project's `.env` across a team without storing plaintext anywhere. Secrets are **encrypted on your machine** before upload — the server only ever sees ciphertext.

```bash
whisper-secrets init                    # set up a project, get a share link for teammates
whisper-secrets import                  # encrypt & upload every entry from .env
whisper-secrets push STRIPE_SECRET_KEY  # add secrets individually
whisper-secrets pull                    # download & decrypt to .env
```

Commit `.env.whisper` (UUIDs only, no plaintext) to git. Teammates run `whisper-secrets join <link>` to clone access and auto-pull.

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

## How It Works

1. `whisper-secrets init` generates a random passphrase and creates `.whisperrc`
2. The passphrase derives an encryption key (PBKDF2-SHA256, 600,000 iterations) and an auth token
3. `push` / `import` encrypt secrets client-side with AES-256-GCM, then upload ciphertext to the server
4. `pull` downloads ciphertext and decrypts locally
5. The server only stores encrypted blobs — zero knowledge of your secrets

**Files:**
- `.whisperrc` — project config (URL + passphrase). Add to `.gitignore`
- `.env.whisper` — maps secret names to server IDs. Commit to git
- `.env` — plaintext secrets from `pull`. Add to `.gitignore`

## Security

- **AES-256-GCM** authenticated encryption with a unique nonce per secret
- **PBKDF2-SHA256** at 600,000 iterations for client-side key derivation (CLI)
- **Server-side encryption** for ephemeral web/Slack/Discord secrets (AES-256-GCM with a server-held key, separate from the application)
- **Automatic cleanup** of expired secrets (every minute)
- **Self-destruct** option deletes a secret immediately after first retrieval
- **Security headers** on all responses: HSTS, CSP, X-Frame-Options, X-Content-Type-Options

Found a vulnerability? **Please report it privately** — see [SECURITY.md](SECURITY.md).

## Self-Hosting

### Prerequisites

- Rust 1.70+
- PostgreSQL 16+
- Docker & Docker Compose (optional)

### Using Docker Compose

```bash
# Start the database (creates the `whisper` database on first start)
docker-compose up -d

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

Whisper uses a hexagonal (Ports & Adapters) layout with a pure Rust domain (`services-core/`), adapters for I/O (`adapters/postgresql-adapter`, `adapters/aes-gcm-crypto`), and applications (`applications/axum/server`, `applications/cli`, `applications/discord`). For a deeper walkthrough — crate responsibilities, routes, storage schema, and where to add new functionality — see [docs/architecture.md](docs/architecture.md).

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for local setup, code standards, tests, and the PR process.

- [Report a bug](https://github.com/quentinved/Whisper/issues/new?template=bug_report.yml)
- [Request a feature](https://github.com/quentinved/Whisper/issues/new?template=feature_request.yml)
- [Report a security issue](SECURITY.md) — please do not open a public issue
- [Join the Slack community](https://join.slack.com/t/whisper-secrets/shared_invite/zt-3vtg200cj-tlOebOVAceFKeqfPbK9WuQ) for questions, ideas, and help

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE).
