# whisper-secrets agent skill

Teaches AI coding agents (Claude Code, Cursor, Codex, Copilot, …) to manage your project's `.env` with the [`whisper-secrets`](https://www.npmjs.com/package/whisper-secrets) CLI.

When the agent sees a `.env`, a `.env.whisper`, missing environment variables or a hardcoded API key, it offers to handle them with Whisper. It follows one rule throughout: **the agent never sees a secret value.** It works from names only (`whisper-secrets status`, `.env.whisper`). Any command that would expose a value or needs a hidden prompt (`init`, `push`, `rotate`, `share`, `join`) is handed to you to run in your own terminal.

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

You also need the CLI itself: `npm install -g whisper-secrets`.

## Try it

Open a project that has a `.env` and ask:

- *"Set up the env for this project."*
- *"I just cloned this repo, what do I need to run it?"*
- *"Add a Stripe key to the project."*
- *"How do I send this API key to a teammate?"*

## What the agent runs vs. what you run

| The agent runs | You run in your terminal |
|---|---|
| `status`, `import`, `pull` (when there's no `.env` yet), `remove` (after you confirm) | `init`, `invite`, `join`, `push`, `rotate`, `share`, `get` |

The commands on the right either prompt for a hidden value or print a link that carries your team's passphrase. Neither should pass through an AI conversation.
