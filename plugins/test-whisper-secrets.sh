#!/bin/sh
# Checks for the whisper-secrets agent skill and Claude Code plugin:
#   - the marketplace, plugin and hook manifests are valid JSON
#   - SKILL.md frontmatter follows the Agent Skills spec (name, description)
#   - the secret-file guard hook blocks and allows the right files,
#     both with jq and with its sed fallback
# Run from the repo root: sh plugins/test-whisper-secrets.sh

set -u

ROOT=$(pwd)
PLUGIN=$ROOT/plugins/whisper-secrets
SKILL=$PLUGIN/skills/whisper-secrets/SKILL.md
HOOK=$PLUGIN/hooks/guard-secret-files.sh
failures=0

pass() { printf '  ok    %s\n' "$1"; }
fail() {
  printf '  FAIL  %s\n' "$1"
  failures=$((failures + 1))
}

check_manifests() {
  echo "manifests"
  for file in .claude-plugin/marketplace.json "$PLUGIN/.claude-plugin/plugin.json" "$PLUGIN/hooks/hooks.json"; do
    if jq empty "$file" 2>/dev/null; then pass "${file#"$ROOT"/} is valid JSON"; else fail "${file#"$ROOT"/} is not valid JSON"; fi
  done
  for source in $(jq -r '.plugins[].source' .claude-plugin/marketplace.json); do
    if [ -f "$source/.claude-plugin/plugin.json" ]; then pass "marketplace source $source exists"; else fail "marketplace source $source has no plugin.json"; fi
  done
  if [ -x "$HOOK" ]; then pass "hook script is executable"; else fail "hook script is not executable"; fi
}

frontmatter_field() {
  sed -n '2,/^---$/p' "$SKILL" | sed -n "s/^$1: //p"
}

check_skill() {
  echo "SKILL.md"
  name=$(frontmatter_field name)
  description=$(frontmatter_field description)
  if [ "$name" = "$(basename "$(dirname "$SKILL")")" ]; then pass "name matches its folder"; else fail "name '$name' does not match its folder"; fi
  if printf '%s' "$name" | grep -Eq '^[a-z0-9]+(-[a-z0-9]+)*$' && [ ${#name} -le 64 ]; then pass "name is lowercase-hyphenated, <= 64 chars"; else fail "invalid name '$name'"; fi
  if [ -n "$description" ] && [ ${#description} -le 1024 ]; then pass "description is 1-1024 chars (${#description})"; else fail "description is empty or longer than 1024 chars (${#description})"; fi
}

# expect_hook <label> <expected exit> <tool input JSON> [PATH]
expect_hook() {
  if [ -n "${4:-}" ]; then
    printf '%s' "$3" | env PATH="$4" sh "$HOOK" >/dev/null 2>&1
  else
    printf '%s' "$3" | sh "$HOOK" >/dev/null 2>&1
  fi
  status=$?
  if [ "$status" -eq "$2" ]; then pass "$1"; else fail "$1 (exit $status, expected $2)"; fi
}

check_hook() {
  mode=$1
  path=${2:-}
  echo "hook ($mode)"
  for case in ".env 2" ".env.local 2" ".whisperrc 2" ".env.development 0" ".env.whisper 0" ".env.example 0" "src/app.ts 0"; do
    file=${case% *}
    expected=${case##* }
    expect_hook "Read $file -> $expected" "$expected" "{\"tool_name\":\"Read\",\"tool_input\":{\"file_path\":\"$PROJECT/$file\"}}" "$path"
  done
  expect_hook "Grep on .env.local -> 2" 2 "{\"tool_name\":\"Grep\",\"tool_input\":{\"pattern\":\"KEY\",\"path\":\"$PROJECT/.env.local\"}}" "$path"
  expect_hook "Grep on a folder -> 0" 0 "{\"tool_name\":\"Grep\",\"tool_input\":{\"pattern\":\"KEY\",\"path\":\"$PROJECT\"}}" "$path"
  expect_hook "Windows path to .env -> 2" 2 '{"tool_name":"Read","tool_input":{"file_path":"C:\\p\\.env"}}' "$path"
}

# A project with committed and uncommitted env files, in a folder with a space.
make_project() {
  PROJECT="$TMP/my project"
  mkdir -p "$PROJECT"
  (
    cd "$PROJECT" &&
      git init -q &&
      echo 'NEXT_PUBLIC_API=https://api.example.com' >.env.development &&
      echo '{}' >.whisperrc &&
      git add -f .env.development .whisperrc &&
      git -c user.name=test -c user.email=test@example.com commit -qm init &&
      echo 'SECRET=1' >.env &&
      echo 'SECRET=1' >.env.local
  )
}

# A PATH with only what the hook needs, so the sed fallback runs without jq.
make_path_without_jq() {
  mkdir -p "$TMP/bin"
  for tool in sh cat sed head git dirname basename printf env; do
    ln -s "$(command -v "$tool")" "$TMP/bin/$tool"
  done
}

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
make_project
make_path_without_jq

check_manifests
check_skill
check_hook "jq"
check_hook "sed fallback, no jq" "$TMP/bin"

if [ "$failures" -gt 0 ]; then
  echo "$failures check(s) failed"
  exit 1
fi
echo "all checks passed"
