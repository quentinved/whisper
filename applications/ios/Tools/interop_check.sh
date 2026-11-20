#!/usr/bin/env bash
# Proves the iOS AES-GCM implementation is byte-compatible with the Rust one.
#
# The ephemeral payload format (`nonce[12] ‖ ciphertext ‖ tag[16]`, base64url)
# is a cross-language contract: a link created on iPhone must open in the web
# UI, and a link created by the CLI must open in the app. Nothing in the type
# system enforces that, so it is enforced here instead.
#
# Run from anywhere:  applications/ios/Tools/interop_check.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
KIT="$REPO_ROOT/applications/ios/WhisperKit"

PLAINTEXTS=(
  "hello"
  "correct horse battery staple"
  "unicode ✅ 🦀 复杂密码"
  "multi
line
secret"
)

echo "==> Building whisper-probe from the app's own WhisperKit adapters"
# Xcode's toolchain, not a swiftly/brew one on PATH, so it matches the SDK.
xcrun swift build -q -c release --package-path "$KIT" --product whisper-probe
PROBE="$(xcrun swift build -c release --package-path "$KIT" --show-bin-path)/whisper-probe"

echo "==> Building the Rust harness"
cargo build -q --manifest-path "$REPO_ROOT/Cargo.toml" -p aes-gcm-crypto --example interop_vector
RUST="$REPO_ROOT/target/debug/examples/interop_vector"

failures=0
check() {
  local direction="$1" expected="$2" actual="$3"
  if [[ "$expected" == "$actual" ]]; then
    printf '  ok   %-16s %s\n' "$direction" "$(head -1 <<<"$expected")"
  else
    printf '  FAIL %-16s expected %q, got %q\n' "$direction" "$expected" "$actual"
    failures=$((failures + 1))
  fi
}

echo "==> Round-tripping ${#PLAINTEXTS[@]} secrets in both directions"
for plaintext in "${PLAINTEXTS[@]}"; do
  read -r key payload < <("$RUST" encrypt "$plaintext")
  check "rust -> swift" "$plaintext" "$("$PROBE" decrypt "$key" "$payload")"

  read -r key payload < <("$PROBE" encrypt "$plaintext")
  check "swift -> rust" "$plaintext" "$("$RUST" decrypt "$key" "$payload")"
done

echo "==> Rejecting a payload sealed under a different key"
read -r _ payload < <("$PROBE" encrypt "secret")
read -r other_key _ < <("$PROBE" encrypt "decoy")
if "$PROBE" decrypt "$other_key" "$payload" >/dev/null 2>&1; then
  echo "  FAIL wrong key was accepted"
  failures=$((failures + 1))
else
  echo "  ok   wrong-key     rejected"
fi

if (( failures > 0 )); then
  echo "FAILED: $failures check(s)"
  exit 1
fi
echo "PASSED: the iOS and Rust payload formats agree"
