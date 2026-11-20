#!/usr/bin/env bash
# End-to-end check against a running Whisper server, exercising the exact WhisperKit
# commands and adapters the app ships — `CreateClientEncryptedSecret`,
# `GetSecretById`, `AesGcm` and `HTTPSharedSecretRepository` — rather than a
# reimplementation of them.
#
# It proves the three things that have to hold for the app to be useful:
#   1. a secret sealed on the client can be stored and read back by the client;
#   2. the payload the client uploads is decryptable by the Rust implementation,
#      which is what makes an iPhone link openable in the web UI;
#   3. burn-after-reading really does burn.
#
#   applications/ios/Tools/e2e_check.sh [server_url]
set -euo pipefail

SERVER="${1:-http://localhost:1212}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
KIT="$REPO_ROOT/applications/ios/WhisperKit"

echo "==> Server: $SERVER"
if ! curl -fsS -o /dev/null "$SERVER/health"; then
  echo "No server at $SERVER. Start one first (see applications/ios/README.md)." >&2
  exit 1
fi

echo "==> Building whisper-probe from the app's own WhisperKit adapters"
# Xcode's toolchain, not a swiftly/brew one on PATH, so it matches the SDK.
xcrun swift build -q -c release --package-path "$KIT" --product whisper-probe
PROBE="$(xcrun swift build -c release --package-path "$KIT" --show-bin-path)/whisper-probe"

cargo build -q --manifest-path "$REPO_ROOT/Cargo.toml" -p aes-gcm-crypto --example interop_vector
RUST="$REPO_ROOT/target/debug/examples/interop_vector"

failures=0
check() {
  if [[ "$2" == "$3" ]]; then
    printf '  ok   %s\n' "$1"
  else
    printf '  FAIL %s — expected %q, got %q\n' "$1" "$2" "$3"
    failures=$((failures + 1))
  fi
}

SECRET="e2e secret 🔐 $(date +%s)"

echo "==> 1. Create a re-readable secret from the client, then read it back"
link="$("$PROBE" create "$SERVER" "$SECRET")"
echo "  link: ${link:0:60}…"
check "client round-trip" "$SECRET" "$("$PROBE" open "$link")"

echo "==> 2. Rust decrypts what the client uploaded (link opens in the web UI)"
read -r key payload < <("$PROBE" payload "$link")
check "swift -> rust" "$SECRET" "$("$RUST" decrypt "$key" "$payload")"

echo "==> 3. The server never sees the key"
id="$(sed -E 's/.*shared_secret_id=([^#]+).*/\1/' <<<"$link")"
stored="$(curl -fsS "$SERVER/secret/$id" | python3 -c 'import json,sys; print(json.load(sys.stdin)["secret"])')"
if [[ "$stored" == *"$key"* ]]; then
  echo "  FAIL the key appears in the stored payload"
  failures=$((failures + 1))
else
  echo "  ok   stored payload contains no key material"
fi

echo "==> 4. Burn after reading destroys the secret"
burn_link="$("$PROBE" create "$SERVER" "one-time" burn)"
check "first read" "one-time" "$("$PROBE" open "$burn_link")"
if "$PROBE" open "$burn_link" >/dev/null 2>&1; then
  echo "  FAIL the secret survived its first read"
  failures=$((failures + 1))
else
  echo "  ok   second read is gone"
fi

if (( failures > 0 )); then
  echo "FAILED: $failures check(s)"
  exit 1
fi
echo "PASSED: the app's client stack works against a real Whisper server"
