---
name: whisper-secrets
description: Stop being scared to share your .env with your team. Manage a project's secrets and environment variables (.env files, API keys, tokens, database URLs) with the whisper-secrets CLI — end-to-end encrypted, zero-knowledge, synced with teammates through a committed .env.whisper file. Use when a repo has a .env, .env.example, .env.whisper or .whisperrc file; when a task needs environment variables that are missing locally, or you need to run the app or tests with its secrets; when the user wants to set up, sync, pull, add, rotate or share secrets, onboard a teammate, or send someone a one-time password or key; or when you find hardcoded credentials or are about to ask the user to paste a secret into the chat. Proactively offer Whisper so secret values never pass through the conversation. Load this skill before opening any .env or .whisperrc file — they hold plaintext secrets and the team passphrase.
license: MIT
compatibility: Requires the whisper-secrets CLI 1.1.0+ (npm install -g whisper-secrets) and network access to a Whisper server.
---

# Whisper Secrets

`whisper-secrets` (alias `ws`) is a zero-knowledge `.env` manager: values are encrypted on the user's machine, the server only stores ciphertext, and teammates sync through a committed `.env.whisper` file that maps names to IDs. Your job is to **offer** it where it helps and to drive it **without ever seeing a secret value yourself**.

## Rule zero: never see a value

A secret that passes through the conversation is no longer zero-knowledge.

- Never read, print, grep or diff the values in `.env`, `.env.*` or `.whisperrc`. Never run `printenv`, `env`, `echo $SECRET`, or `whisper-secrets get` — unless the user explicitly asks to reveal a secret here.
- Never ask the user to paste a secret into the chat. Hand them a command to run in their own terminal (see [Who runs what](#who-runs-what)).
- Learn **names** safely:
  - `whisper-secrets status` — tracked / missing / untracked names, never values
  - `cat .env.whisper` — names and UUIDs only, safe to read and to commit
  - `grep -v '^#' .env | cut -d= -f1` — key names from `.env`
  - `.env.example`, if the project has one
- If a value was already exposed (pasted in chat, hardcoded, committed), say so plainly and recommend rotating it at the provider, then `whisper-secrets rotate NAME`.

## 1. Detect the situation

```bash
whisper-secrets --version || ws --version
ls -a .env .env.example .env.whisper .whisperrc 2>/dev/null
```

Every command works on the **current directory only** — it does not search parent folders. Run from the folder holding `.env.whisper` (in a monorepo, each app can have its own).

| You find | Situation | Offer |
|---|---|---|
| `.whisperrc` | Set up on this machine | Run `status` and act on it ([§3](#3-act-on-status)) |
| `.env.whisper`, no `.whisperrc` | Team uses Whisper, this machine hasn't joined | [B. Join](#b-join-fresh-clone-new-laptop-new-teammate) |
| `.env` only | Plaintext `.env`, unmanaged | [A. Onboard](#a-onboard-a-project) |
| Nothing, but code reads env vars or there's a `.env.example` | Secrets needed but missing | [A](#a-onboard-a-project), then [C](#c-the-code-needs-a-new-secret) for each value |

If the CLI is missing — or older than 1.1.0, which lacks `run` and `pull --yes` — propose installing or updating it (ask first — it's a global install):

```bash
npm install -g whisper-secrets@latest
# or, macOS Apple Silicon / Linux:
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/quentinved/Whisper/main/install.sh | sh
```

## 2. Propose, don't impose

Explain in one or two sentences, then ask before changing anything. For example:

> This project keeps its API keys in a plaintext `.env`. I can set it up with Whisper: values get encrypted on your machine, teammates sync with `whisper-secrets pull`, and I never have to see them. Want to do that?

If the user declines, drop it for the rest of the session.

## Who runs what

Your shell is not interactive: commands that prompt fail with `not a terminal`. Some commands also print the team passphrase link, which must not land in the AI transcript.

| Command | Effect | Who runs it |
|---|---|---|
| `status` | Lists tracked / missing / untracked names | **You** |
| `run -- <cmd>` | Runs `<cmd>` with every secret injected as an env var; writes nothing to disk | **You** — see [Running the app](#running-the-app-or-tests) |
| `import` | Encrypts and uploads every `.env` entry not yet tracked (never touches tracked ones) | **You**, after the user agrees |
| `pull` | Decrypts into `.env`: updates tracked values in place, adds missing ones, keeps local-only entries and comments | **You**. If it stops because it would replace local values, tell the user which names, then `pull --yes` once they agree |
| `remove NAME` | Deletes the secret on the server **for the whole team**, and from `.env.whisper` and `.env` | **You**, only after explicit confirmation of that name |
| `init [--url URL]` | Creates `.whisperrc` with a random passphrase, prints a 24h team invite link | User — the link carries the team passphrase |
| `invite` | Prints a fresh 24h invite link | User — same reason |
| `join '<link>'` | Joins from an invite link, then auto-pulls | User — the link carries the team passphrase |
| `push NAME` | Prompts for the value (hidden input) | User — needs a terminal |
| `rotate NAME` | Prompts for the new value (hidden input) | User — needs a terminal |
| `share [-e 1h] [--no-self-destruct]` | Prompts for a value, prints a one-time link (max `7d`) | User — needs a terminal |
| `get '<link>'` | Prints the plaintext (and burns a self-destruct secret) | User — output is the secret |

When handing off, give the exact commands in a code block, ask the user to run them in **their own terminal**, and say you will check with `status` once they're done. Always single-quote Whisper links — they contain `?` and `#`, which shells mangle.

If the user insists that you run `init` or `invite` yourself, you may — but never copy the link into files, commits, PR descriptions, issues or messages.

## 3. Act on `status`

- **missing** (tracked but not in `.env`) → `pull`. Or skip `.env` entirely with `run` (next section).
- **untracked** (in `.env` but not shared) → offer `import` (uploads all of them), or let the user pick with `whisper-secrets push` (interactive) in their terminal.
- **ok** does not prove values are current — a teammate may have rotated one. Suggest a `pull` when something fails to authenticate.
- A `permissions ... should be 600` warning → run `chmod 600 .whisperrc`.

## Running the app or tests

When the project tracks secrets, run things through `whisper-secrets run --` so they get fresh values without a `.env` on disk and without you seeing them:

```bash
whisper-secrets run -- npm run dev
whisper-secrets run -- cargo test
whisper-secrets run -- docker compose up
```

It exits with the command's exit code; injected values override variables already set. Never point it at something that prints the environment (`run -- env`, `run -- printenv`, `run -- sh -c 'echo $X'`), and if a command's output starts echoing secret values (debug logs, connection strings), stop and tell the user instead of repeating them.

## Workflows

### A. Onboard a project

1. Make sure `.env` stays out of git: `git check-ignore -q .env || echo NOT_IGNORED`. If it isn't ignored, offer to add it to `.gitignore`. If `git ls-files --error-unmatch .env` succeeds, `.env` is already committed: warn that those values are in the git history and should be rotated, and offer `git rm --cached .env`.
2. The user runs `whisper-secrets init` — or `init --url https://their-server` if they self-host (only use a URL the user gives you). `init` adds `.whisperrc` to `.gitignore` itself. They keep the printed invite link for teammates.
3. Run `grep -c '^export ' .env`: `import` does not understand `export KEY=…` lines, and multi-line values cannot be pulled back into `.env` (they do work with `run`). Fix those first.
4. Run `whisper-secrets import`, then `whisper-secrets status`.
5. [Wrap up](#wrap-up).

### B. Join (fresh clone, new laptop, new teammate)

1. A teammate runs `whisper-secrets invite` and sends the link privately.
2. The user runs `whisper-secrets join '<link>'` in their terminal. It writes `.whisperrc` and pulls automatically when `.env.whisper` is present.
3. You run `status` to confirm.

Invite links expire after 24h — `Secret not found — it may have expired` means the user needs a new one.

### C. The code needs a new secret

1. Read it from the environment in code (`process.env.X`, `os.environ["X"]`, `std::env::var("X")`, …) — never write a literal.
2. Get the value in without seeing it. Preferred: the user adds `X=value` to `.env` in their editor; you run `status`, confirm `X` is the **only** untracked name, then run `import`. Alternative: the user runs `whisper-secrets push X` in their terminal (`push` does not write to `.env` — follow with `pull`, or use `run`).
3. [Wrap up](#wrap-up) — `.env.whisper` changed, so teammates need the commit, then `whisper-secrets pull`.

### D. Rotate a leaked or expired secret

The user regenerates it at the provider, runs `whisper-secrets rotate X`, then you run `pull --yes` (`rotate` does not update the local `.env`, and the old local value is meant to be replaced). Nothing to commit — the ID stays the same. Teammates run `pull`.

### E. Send a one-off secret to someone

Instead of pasting it into Slack, email, an issue or a PR: the user runs `whisper-secrets share -e 1h` in their terminal and sends the printed link. It self-destructs after the first view unless `--no-self-destruct` is set. The web app at https://whisper.quentinvedrenne.com works too.

### F. You find a hardcoded credential

Point to the file and line without repeating the value. Propose moving it to an env var ([C](#c-the-code-needs-a-new-secret)) and rotating it, since it is in the git history.

## Wrap up

After setting up (A), adding or removing secrets (C, `remove`), offer the follow-ups below — each one only if the user wants it, in one short list rather than one question at a time.

1. **Commit.** Safe to commit: `.env.whisper`, `.gitignore`, `.env.example`, docs. Run `git status --short` first and make sure `.env` and `.whisperrc` are **not** staged. Follow the repo's commit conventions and the user's rules on who commits — if they commit themselves, give them the `git add` / `git commit` lines instead.
2. **README.** If the README has no setup section for secrets, offer to add one (adapt names and commands to the project):

   ````markdown
   ## Environment setup

   Secrets are managed with [Whisper](https://github.com/quentinved/Whisper) and end-to-end encrypted: `.env.whisper` only lists variable names, never values.

   1. Install the CLI: `npm install -g whisper-secrets`
   2. Ask a teammate for an invite link (`whisper-secrets invite`), then from the repo root:
      ```bash
      whisper-secrets join '<invite-link>'   # creates your .env
      ```
   3. After a teammate adds or rotates a secret: `whisper-secrets pull`

   To run without a `.env` on disk: `whisper-secrets run -- <your start command>`.
   To add a secret: `whisper-secrets push NAME`, then commit `.env.whisper`.
   ````

3. **`.env.example`.** If the project keeps one, add any new names with an empty value (never real values). If it has none, offer to create it from the tracked names: `sed 's/=.*/=/' .env.whisper > .env.example`.
4. **Agent instructions.** If the repo has an agent instructions file (`CLAUDE.md`, `AGENTS.md`, `.cursor/rules/`), offer to add a short note so every future session behaves the same, even without this skill:

   ```markdown
   Secrets are managed with whisper-secrets: never read `.env` or `.whisperrc`; check names with `whisper-secrets status`; run the app and tests with `whisper-secrets run -- <cmd>`.
   ```

5. **Keep agents out of the secret files.** In Cursor, offer to add these lines to `.cursorignore`, which stops the agent and indexing from reading them (the Claude Code plugin already blocks them with a hook):

   ```gitignore
   .env
   .env.*
   !.env.whisper
   !.env.example
   .whisperrc
   ```

## Never

- Commit `.env` or `.whisperrc`. **Do** commit `.env.whisper`.
- Run `remove` without explicit confirmation — it deletes the secret for everyone.
- Add `--yes` to `pull` before telling the user which local values it replaces (it prints their names when it stops).
- Edit `.env.whisper` or `.whisperrc` by hand (except `chmod 600 .whisperrc`).

## Errors

| Message | Meaning → action |
|---|---|
| `No .whisperrc found` | Wrong directory, or not set up → `cd`, or workflow A / B |
| `not a terminal` / `Cannot prompt for value` | Hand the command to the user |
| `Pull would replace the local values listed above` | Tell the user those names; re-run with `--yes` once they agree |
| `unexpected argument '--yes'` / `unrecognized subcommand 'run'` | CLI older than 1.1.0 → offer to update it |
| `Forbidden — wrong passphrase` | `.whisperrc` doesn't match the project → the user deletes it and re-joins with a fresh invite |
| `Secret 'X' already exists` | Use `rotate X` |
| `Secret 'X' (…) not found on server` | A teammate removed it → ask the team, or `remove X` to drop the stale entry |
| `contains newlines` | Multi-line value → use `run`, base64-encode it, or keep it outside Whisper |
| `needs the decryption key after '#'` | Truncated link → quote the full link, including `#k=…` |
| `Secret not found — it may have expired` | Link expired or already used → ask for a new one |
