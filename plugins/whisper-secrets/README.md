# whisper-secrets agent skill

**Stop being scared to share your .env with your team.** Your AI coding agent sets it up, syncs it and runs your project with it — end-to-end encrypted, and the agent never sees a value.

Teaches AI coding agents (Claude Code, Cursor, Codex, Copilot, …) to manage your project's `.env` with the [`whisper-secrets`](https://www.npmjs.com/package/whisper-secrets) CLI.

When the agent sees a `.env`, a `.env.whisper`, missing environment variables or a hardcoded API key, it offers to handle them with Whisper. It follows one rule throughout: **the agent never sees a secret value.**

- It works from names only (`whisper-secrets status`, `.env.whisper`).
- It runs your app and tests with `whisper-secrets run -- <cmd>`, which passes secrets to the process without writing them anywhere.
- Any command that would expose a value or needs a hidden prompt (`init`, `push`, `rotate`, `share`, `join`) is handed to you to run in your own terminal.

With the Claude Code plugin, a hook also blocks the agent's file tools from opening `.env`, `.env.local`, `.whisperrc` and similar files. It stays active even before the skill loads. `.env.whisper`, `.env.example` and `.env` files you've committed to git stay readable; `.whisperrc` never is. In Cursor, the skill offers to add the same files to `.cursorignore` instead.

Once setup is done, it offers to commit the safe files (`.env.whisper`, `.gitignore`, `.env.example`) if you want. It can also add an "Environment setup" section to your README and a note to your `CLAUDE.md` / `AGENTS.md`.

## Install

**Claude Code**

```
/plugin marketplace add quentinved/Whisper
/plugin install whisper-secrets@whisper
```

**Cursor, Codex, Copilot and other agents** (via [skills.sh](https://skills.sh))

```bash
npx skills add quentinved/Whisper              # pick your agents interactively
npx skills add quentinved/Whisper -a cursor    # or target one
```

**Manual:** copy [`skills/whisper-secrets/`](skills/whisper-secrets/) into one of these folders:

- For one project: `.claude/skills/` or `.cursor/skills/`
- For all your projects: `~/.claude/skills/` or `~/.cursor/skills/`

Cursor also reads `.claude/skills/`, so a single copy there works for both.

You also need the CLI itself, version 1.1.0 or later: `npm install -g whisper-secrets`.

## Try it

Open a project that has a `.env` and ask:

- *"Set up the env for this project."*
- *"I just cloned this repo, what do I need to run it?"*
- *"Add a Stripe key to the project."*
- *"How do I send this API key to a teammate?"*

## What the agent runs vs. what you run

| The agent runs | You run in your terminal |
|---|---|
| `status`, `run -- <cmd>`, `import`, `pull` (asks you before replacing a local value), `remove` (after you confirm) | `init`, `invite`, `join`, `push`, `rotate`, `share`, `get` |

The commands on the right either prompt for a hidden value or print a link that carries your team's passphrase. Neither should pass through an AI conversation.
