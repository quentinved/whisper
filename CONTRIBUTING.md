# Contributing to Whisper

Thanks for your interest in Whisper. This guide covers how to get a local checkout running, where things live, and how to submit a change that has a good chance of landing.

If you want to chat before opening a PR, join the [Slack community](https://join.slack.com/t/whisper-secrets/shared_invite/zt-3vtg200cj-tlOebOVAceFKeqfPbK9WuQ).

## Before you start

- **Search existing issues and PRs** for the problem you're about to report or fix.
- For **non-trivial changes** (new commands, new endpoints, architecture shifts), open an issue first to align on scope. Small fixes and doc changes don't need one.
- **Security issues go through [SECURITY.md](SECURITY.md), not a public issue.**

## Local setup

```bash
# Prerequisites: Rust 1.70+, Docker, Docker Compose
git clone https://github.com/quentinved/Whisper
cd Whisper

# Database (creates the `whisper` database automatically on first start)
docker-compose up -d

# AES key (one-time)
openssl rand -out aes_key.bin 32

# Run the server
cargo run --bin whisper-server -- --url-posgtresql "postgres://postgres:toto@localhost/whisper"

# In another terminal: run the CLI against your local server
cargo run --bin whisper-secrets -- init --url http://localhost:1212
```

## Architecture

Whisper follows a hexagonal (Ports & Adapters) layout. When adding code, put it in the right layer:

```
services-core/              domain logic, entities, value objects (no I/O)
adapters/
  aes-gcm-crypto/           encryption adapter (implements SecretEncryption trait)
  postgresql-adapter/       DB adapter (implements SharedSecretRepository trait)
applications/
  axum/server/              HTTP server, routes, templates, Slack integration
  cli/                      whisper-secrets CLI
  discord/                  Discord bot
```

Rules of thumb:
- New business logic → `services-core/`
- New storage/encryption backend → `adapters/`
- New HTTP route / CLI command → `applications/`
- Domain types stay pure Rust (no `sqlx`, no `reqwest`) — adapters translate at the boundary

More context: [docs/architecture.md](docs/architecture.md) has the deep walkthrough — crate responsibilities, routes, storage schema, and where to add new functionality. [CLAUDE.md](CLAUDE.md) has the internal project guide for maintainers.

## Coding standards

- `cargo fmt` before committing — CI rejects unformatted code
- `cargo clippy --workspace --all-targets --features test-utils -- -D warnings` must pass
- Keep functions short and focused; top-level `run()` orchestrates, helpers do the work
- Prefer `TryFrom`/`TryInto` over custom `parse()`/`new()` constructors
- Wrap raw strings/primitives in newtypes (e.g. `SecretId`, `BearerToken`)
- Private fields + `new()` constructor on domain structs
- Dev-only imports (e.g. `rstest`) gated under `#[cfg(test)]`

## Tests

```bash
cargo test --workspace                                         # Everything
cargo test -p whisper-core                                     # Domain
cargo test -p whisper-crypto                                   # Encryption
cargo test -p whisper-secrets --test cli -- --test-threads=1   # CLI integration (must be single-threaded)
cargo test -p whisper-core --features test-utils --test cucumber  # BDD
```

Every new feature needs tests. Bug fixes should include a regression test.

## Commit messages

Follow the existing style in `git log`:

- `ADD:` new feature or file
- `FIX:` bug fix
- `UPDATE:` enhancement to existing behavior
- `BUMP:` version bumps
- `REMOVE:` deletions

Keep the subject under ~60 chars. Explain **why** in the body if it's not obvious.

## Pull requests

1. Fork, create a branch off `main`
2. Make your change, add tests
3. Run `cargo fmt` and `cargo clippy` locally
4. Add a `CHANGELOG.md` entry under a new dated section if the change is user-visible
5. Open the PR. Fill out the template. Link related issues.

We review PRs on a best-effort basis — please be patient. A green CI and a clear description go a long way.

## Releasing

Releases are handled by maintainers. See [CLAUDE.md](CLAUDE.md#releasing-the-cli--npm-package) for the internal process if you're curious.

## License

By contributing, you agree that your contributions are licensed under the project's [MIT License](LICENSE).
