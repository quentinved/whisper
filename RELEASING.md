# Releasing

Whisper ships three things, and each one is released on its own:

| What | Released by | How to trigger it |
|---|---|---|
| **CLI**: `whisper-secrets` on npm, plus binaries on GitHub Releases | [`whisper-cli-release.yaml`](.github/workflows/whisper-cli-release.yaml) | Push a `vX.Y.Z` tag |
| **Website and server** (and the Discord bot) | [`whisper-release.yaml`](.github/workflows/whisper-release.yaml), shown as "Release & Deploy" | Run it by hand from GitHub Actions |
| **Agent skill**: the Claude Code plugin in [`plugins/whisper-secrets/`](plugins/whisper-secrets/) | Nothing to run | Goes live as soon as it's merged to `main` |

Merging a PR releases nothing except the agent skill. After merging, trigger whichever of the other two the PR changed.

## CLI

1. In the PR, bump the version. `applications/cli/Cargo.toml` is the single source of truth:
   ```bash
   cargo set-version -p whisper-secrets X.Y.Z   # needs cargo-edit
   ```
2. Merge the PR.
3. Tag `main` and push the tag:
   ```bash
   git fetch origin
   git tag vX.Y.Z origin/main && git push origin vX.Y.Z
   ```
4. Watch the **CLI Release** run (`gh run watch`, or the Actions tab). It:
   - runs fmt, clippy and the unit and CLI tests;
   - checks that the tag matches `Cargo.toml`;
   - builds linux-x64/arm64, darwin-arm64 and win32-x64;
   - creates the GitHub release;
   - publishes `whisper-secrets` and the 4 `@whisper-secrets/*` packages to npm (Trusted Publishing, no token).
5. Check it worked:
   ```bash
   npm view whisper-secrets version
   npm install -g whisper-secrets@latest && whisper-secrets --version
   ```

If the run fails, first check npm (`npm view whisper-secrets@X.Y.Z version`):

- **Nothing was published** (the usual case: it failed before or during the builds):
  1. Fix the problem in a PR and merge it. Keep the same version.
  2. Delete the draft release and the tag.
  3. Tag the fixed `main` again.

  "Re-run jobs" doesn't work here: it reuses the workflow file from the old tagged commit. "Run workflow" doesn't either, because the draft release already exists.
  ```bash
  gh release delete vX.Y.Z --yes
  git push origin :refs/tags/vX.Y.Z && git tag -d vX.Y.Z
  git fetch origin && git tag vX.Y.Z origin/main && git push origin vX.Y.Z
  ```
- **Some packages were published**: npm never lets you reuse a version. Bump to the next patch version and release that instead.

You don't need a clean checkout to tag: `git tag vX.Y.Z origin/main` tags the remote `main` directly, so uncommitted work stays where it is.

## Website and server

1. In the PR, bump the server version:
   ```bash
   cargo set-version -p whisper-server X.Y.Z
   ```
   You must bump it when CSS or JS changes: it's the `?v=` cache-buster on static assets. It is also shown in the site footer, which is how you can tell a deploy went out.
2. Merge the PR.
3. Start the deploy. The workflow needs a repo admin, so run it from the owner's account:
   ```bash
   gh workflow run whisper-release.yaml --repo quentinved/Whisper --ref main
   ```
   Or use **Actions → Release & Deploy → Run workflow** on `main`. It:
   - runs the tests;
   - builds the server inside an Oracle Linux 9 container (glibc 2.34) and builds the Discord bot;
   - copies both to the Oracle Cloud VM, then restarts and health-checks them.
4. Check it worked:
   ```bash
   curl https://whisper.quentinvedrenne.com/version   # prints the new version
   ```

## Agent skill (Claude Code / Cursor plugin)

- It is live as soon as it's merged: the marketplace and `npx skills add quentinved/Whisper` both read `main`.
- When `SKILL.md` or the hook changes, bump `version` in `plugins/whisper-secrets/.claude-plugin/plugin.json`. Claude Code uses it to offer the update to people who already installed the plugin.
- Existing users pick up changes with `/plugin marketplace update whisper` in Claude Code, or by re-running `npx skills add quentinved/Whisper`.
- Before merging, run:
  ```bash
  sh plugins/test-whisper-secrets.sh    # also runs in CI
  claude plugin validate . && claude plugin validate plugins/whisper-secrets
  ```
- The skill tells agents which CLI commands they can run. When a CLI command changes, update `SKILL.md` in the same PR.

## When a PR changes several of these

Release in this order, so the skill never points agents at a CLI that isn't published yet:

1. Merge the PR. The skill is now live.
2. Push the CLI tag right away and wait for npm to publish.
3. Deploy the website.
