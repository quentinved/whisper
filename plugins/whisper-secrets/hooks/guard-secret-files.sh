#!/bin/sh
# PreToolUse guard: stop Claude's file tools from opening files that hold
# plaintext secrets (.env, .env.local, ...) or the team passphrase (.whisperrc).
# Still readable: .env.whisper (names + IDs), templates like .env.example, and
# .env files committed to git (already visible to anyone with the repo).
# Reads the tool call as JSON on stdin; exit 2 blocks it and shows stderr to Claude.

input=$(cat)

# `file_path` for Read, `path` for Grep.
target_path() {
  if command -v jq >/dev/null 2>&1; then
    printf '%s' "$input" | jq -r '.tool_input.file_path // .tool_input.path // empty'
  else
    # Without jq: fine for ordinary paths (it can't unescape quotes in a name).
    printf '%s' "$input" |
      sed -n -E 's/.*"(file_path|path)"[[:space:]]*:[[:space:]]*"([^"]*)".*/\2/p' |
      head -n 1
  fi
}

is_committed() {
  git -C "$(dirname "$1")" ls-files --error-unmatch -- "$(basename "$1")" >/dev/null 2>&1
}

target=$(target_path)
name=${target##*/}
name=${name##*\\}

case "$name" in
  .whisperrc) ;;
  .env.whisper | .env.example | .env.sample | .env.template | .env.dist) exit 0 ;;
  .env | .env.*) is_committed "$target" && exit 0 ;;
  *) exit 0 ;;
esac

cat >&2 <<MSG
Blocked by the whisper-secrets plugin: $name holds plaintext secrets or the team passphrase, and anything read here lands in the conversation.
Use \`whisper-secrets status\` or \`cat .env.whisper\` for secret names, and \`whisper-secrets run -- <cmd>\` to run code with the values.
MSG
exit 2
