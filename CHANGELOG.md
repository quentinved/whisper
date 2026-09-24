# Changelog
## 24/09/2026 - https://github.com/quentinved/whisper/pull/22
- New `whisper-secrets` agent skill: AI coding agents (Claude Code, Cursor, Codex, Copilot, …) offer to manage a project's `.env` with Whisper and never see a secret value. They work from names only (`status`, `.env.whisper`), run the app and tests through `whisper-secrets run`, and hand any command that prompts for a hidden value or prints the team passphrase link back to the user
- When setup is done, the agent offers to commit `.env.whisper` (only if the user wants), add an "Environment setup" section to the project README, generate `.env.example` from the tracked names, and add a note to `CLAUDE.md` / `AGENTS.md`
- Claude Code: `/plugin marketplace add quentinved/Whisper`, then `/plugin install whisper-secrets@whisper` (repo-root `.claude-plugin/marketplace.json`, plugin in `plugins/whisper-secrets/`). The plugin ships a `PreToolUse` hook that blocks the agent's Read/Grep tools on `.env`, `.env.local`, `.whisperrc` and similar files, even before the skill loads (`.env.whisper`, `.env.example` and git-tracked `.env` files stay readable; `.whisperrc` never is). In Cursor, the skill offers the same protection through `.cursorignore`
- Other agents: `npx skills add quentinved/Whisper`
- CLI v1.1.0: new `whisper-secrets run -- <cmd>` runs a command with every tracked secret injected as an environment variable. Nothing is written to disk and multi-line values work. On macOS/Linux the command replaces the whisper-secrets process (`exec`), so Ctrl+C and SIGTERM from Docker/CI reach it directly and its exit code is passed through; Windows waits for the child and forwards its exit code
- CLI v1.1.0: `pull` updates `.env` in place: tracked values are refreshed, missing ones are appended, and local-only entries and comments are kept (they used to be dropped). It only asks when a local value would change, listing the names; `--yes` skips that question (CI, scripts, AI agents), and without a terminal it fails with a clear "re-run with --yes" error instead of "IO error: not a terminal"
- Website: new "AI coding agents" card on `/integrations`, and `/docs/secrets` documents `run`, the new `pull` behavior, agent setup and a `run`-based CI example
- README and npm README gain a "Use with AI coding agents" section
- Server v1.4.1 (website content only)
- New `RELEASING.md` runbook (CLI tag, website deploy, agent skill), linked from `CONTRIBUTING.md`; CI gains an "Agent skill checks" job (`sh plugins/test-whisper-secrets.sh`: manifests, SKILL.md frontmatter, secret-file hook with and without `jq`)

## 18/09/2026 - https://github.com/quentinved/whisper/pull/21
- New non-consuming `GET /secret/:id/meta` endpoint (`{ exists, client_encrypted, self_destruct }`): the reveal page now checks a link before fetching it, so a zero-knowledge self-destruct secret is no longer destroyed when the link arrives without its `#` fragment key
- Reveal page guards the response parsing, can't double-fire, and offers a retry instead of a dead end on a transient failure
- Create page computes the expiration at submit time (not page load) and uses the local timezone for custom dates
- CSS and JS are served with a `?v=` version and an immutable `Cache-Control`, so a CDN can no longer serve stale styles after a deploy
- Server v1.4.0: hosting moved from Scaleway (Paris) to Oracle Cloud (Stockholm) — still in the EU, so data residency is unchanged
- Release binary is built inside an `oraclelinux:9` container and verified to execute there: Oracle Linux 9 ships glibc 2.34 while the CI runner has 2.39, so a runner-built binary will not start on the deploy target
- Deploy runs as `opc` via `sudo` into `/opt/whisper` (not `root` into `/root`), relabels binaries `bin_t` for SELinux, and health-checks after restart so a failed deploy fails the job
- Services run as a dedicated unprivileged `whisper` user with systemd hardening, replacing the drifted run-as-root setup
- Privacy policy now discloses the hosting provider and the EU hosting location

## 23/06/2026 - https://github.com/quentinved/whisper/pull/19
- Zero-knowledge one-time secrets: web and CLI now encrypt in the browser/terminal (AES-256-GCM), the key rides only in the link's `#` fragment, and the server stores ciphertext it can't read — new `POST /v1/ephemeral` endpoint and `client_encrypted` column
- Click-to-reveal page: secrets are fetched and decrypted only on user action, so link previews and crawlers can't burn a self-destruct secret (drops the user-agent bot sniffing)
- Slack and Discord encrypt before storing; Raycast extension goes fully zero-knowledge with a legacy fallback and a new Multiple Values form
- SEO: per-page canonical/OpenGraph/Twitter tags built from the configured base URL, `SoftwareApplication` JSON-LD, a 1200×630 share banner with large card, and zero-knowledge copy
- Wider secret card on desktop, inline page JS moved into asset files, dead assets/CSS removed, `aes_key.bin` gitignored
- Fix the `--url-postgresql` flag typo across README/CONTRIBUTING/CLAUDE docs

## 02/05/2026 - https://github.com/quentinved/whisper/pull/13
- CLI v0.5.0: smoother first-run and CI experience
- Every command now fails fast with a clear "run `whisper-secrets init` or `join`" message when `.whisperrc` is missing (no more cryptic config-parse errors)
- `init` prints which server it is targeting (default vs `--url`), auto-appends `.whisperrc` to `.gitignore`, warns that the share link carries the passphrase, and tips the `ws` shortcut + import/push/share workflows
- `push` and `rotate` detect non-interactive shells and surface a clean `NotATerminal` error instead of a generic dialoguer "IO error: not a terminal" — no more confusing CI failures
- `status` shares the same missing-config error and shows next-step hints when zero secrets are tracked
- Centralize the test CWD lock in `config` so unit tests across command modules don't race on `set_current_dir`

## 25/04/2026 - https://github.com/quentinved/whisper/pull/9
- Add OSS contributor essentials: `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, issue/PR templates
- Restructure README around two products (one-time sharing + team `.env`), add CI/npm/release/license badges, move deep architecture to `docs/architecture.md`
- Gate the CLI npm release on tests (fmt, clippy, unit + CLI integration); scope the Scaleway deploy tests to server-only
- `docker-compose.yml` auto-creates the `whisper` database on first start, add platform README for `@whisper-secrets/*` npm packages

## 21/04/2026 - https://github.com/quentinved/whisper/pull/8
- Add anonymous CLI telemetry via Mixpanel (command, success, version, os, arch); opt out with `DO_NOT_TRACK=1`
- Switch npm publish to OIDC Trusted Publishing with `--provenance` (no more `NPM_TOKEN`)
- Add Slack community link to footer, integrations page, and README

## 12/04/2026 - https://github.com/quentinved/whisper/pull/5
- Add `join`, `invite`, `status`, `completions` commands and `ws` alias
- Smart `push` (interactive picker), smarter `pull` (warns about local-only entries), clipboard prompt on share links
- Fix CI: macOS runner, Linux arm64 strip, add checkout to finalize job, drop macOS Intel target
- Add install script (`curl | sh`) and npm distribution with platform-specific binaries
- Update README, npm README, web docs, and integrations page with all new commands and install methods


## 11/04/2026 - https://github.com/quentinved/whisper/pull/4
- Implement CLI distribution via npm and GitHub Releases (Linux x64/arm64, macOS x64/arm64, Windows x64)
- GitHub Actions workflow to build and publish platform-specific npm packages on new release tags

## 11/04/2026 - https://github.com/quentinved/whisper/pull/2
- Implement CLI and api test 
- Implement Cucumber BDD tests for CLI and API

## 05/04/2026 - https://github.com/quentinved/whisper/pull/1
- Add BDD tests for managed secrets upsert and shared secrets creation.