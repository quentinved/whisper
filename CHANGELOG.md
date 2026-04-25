# Changelog
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