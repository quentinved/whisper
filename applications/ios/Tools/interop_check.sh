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
CORE="$REPO_ROOT/applications/ios/Whisper/Core"
BUILD_DIR="$(mktemp -d)"
trap 'rm -rf "$BUILD_DIR"' EXIT

PLAINTEXTS=(
  "hello"
  "correct horse battery staple"
  "unicode ✅ 🦀 复杂密码"
  "multi
line
secret"
)

echo "==> Building the Swift harness"
cat > "$BUILD_DIR/main.swift" <<'EOF'
import Foundation

// Exits non-zero on failure rather than trapping, so the wrong-key case below
// reads as a rejection instead of a crash.
let args = CommandLine.arguments
do {
    switch args[1] {
    case "encrypt":
        let sealed = try EphemeralCrypto.seal(args[2])
        print("\(sealed.keyB64URL) \(sealed.payloadB64URL)")
    case "decrypt":
        print(try EphemeralCrypto.open(payloadB64URL: args[3], keyB64URL: args[2]))
    default:
        exit(2)
    }
} catch {
    FileHandle.standardError.write("\(error.localizedDescription)\n".data(using: .utf8)!)
    exit(1)
}
EOF
swiftc -O -o "$BUILD_DIR/swift_crypto" \
  "$BUILD_DIR/main.swift" "$CORE/Base64URL.swift" "$CORE/EphemeralCrypto.swift"

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
  check "rust -> swift" "$plaintext" "$("$BUILD_DIR/swift_crypto" decrypt "$key" "$payload")"

  read -r key payload < <("$BUILD_DIR/swift_crypto" encrypt "$plaintext")
  check "swift -> rust" "$plaintext" "$("$RUST" decrypt "$key" "$payload")"
done

echo "==> Rejecting a payload sealed under a different key"
read -r _ payload < <("$BUILD_DIR/swift_crypto" encrypt "secret")
read -r other_key _ < <("$BUILD_DIR/swift_crypto" encrypt "decoy")
if "$BUILD_DIR/swift_crypto" decrypt "$other_key" "$payload" >/dev/null 2>&1; then
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
