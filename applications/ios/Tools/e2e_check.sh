#!/usr/bin/env bash
# End-to-end check against a running Whisper server, exercising the exact Swift
# files the app ships — `EphemeralCrypto`, `WhisperLink` and `WhisperClient` —
# rather than a reimplementation of them.
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
CORE="$REPO_ROOT/applications/ios/Whisper/Core"
BUILD_DIR="$(mktemp -d)"
trap 'rm -rf "$BUILD_DIR"' EXIT

echo "==> Server: $SERVER"
if ! curl -fsS -o /dev/null "$SERVER/health"; then
  echo "No server at $SERVER. Start one first (see applications/ios/README.md)." >&2
  exit 1
fi

echo "==> Building a CLI from the app's own networking and crypto sources"
cat > "$BUILD_DIR/main.swift" <<'EOF'
import Foundation

let args = CommandLine.arguments
let semaphore = DispatchSemaphore(value: 0)
var exitCode: Int32 = 0

Task {
    do {
        switch args[1] {
        case "create":
            let server = URL(string: args[2])!
            let sealed = try EphemeralCrypto.seal(args[3])
            let selfDestruct = args.count > 4 && args[4] == "burn"
            let created = try await WhisperClient(server: server).createEphemeral(
                payloadB64URL: sealed.payloadB64URL,
                expiration: Date().addingTimeInterval(3600),
                selfDestruct: selfDestruct
            )
            print(WhisperLink(id: created.id, key: sealed.keyB64URL, server: server).url.absoluteString)

        case "open":
            guard let link = WhisperLink.parse(args[2]) else {
                throw NSError(domain: "e2e", code: 1,
                              userInfo: [NSLocalizedDescriptionKey: "unparseable link"])
            }
            let client = WhisperClient(server: link.server)
            let fetched = try await client.fetch(id: link.id)
            print(
                fetched.clientEncrypted
                    ? try EphemeralCrypto.open(
                        payloadB64URL: fetched.secret, keyB64URL: link.key)
                    : fetched.secret
            )

        case "payload":
            // The raw stored bytes plus the key, so the Rust side can try them.
            guard let link = WhisperLink.parse(args[2]) else { exit(1) }
            let fetched = try await WhisperClient(server: link.server).fetch(id: link.id)
            print("\(link.key) \(fetched.secret)")

        default:
            exitCode = 2
        }
    } catch {
        FileHandle.standardError.write("\(error.localizedDescription)\n".data(using: .utf8)!)
        exitCode = 1
    }
    semaphore.signal()
}

semaphore.wait()
exit(exitCode)
EOF
swiftc -O -o "$BUILD_DIR/whisper_client" \
  "$BUILD_DIR/main.swift" \
  "$CORE/Base64URL.swift" "$CORE/EphemeralCrypto.swift" \
  "$CORE/WhisperLink.swift" "$CORE/WhisperClient.swift"

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
link="$("$BUILD_DIR/whisper_client" create "$SERVER" "$SECRET")"
echo "  link: ${link:0:60}…"
check "client round-trip" "$SECRET" "$("$BUILD_DIR/whisper_client" open "$link")"

echo "==> 2. Rust decrypts what the client uploaded (link opens in the web UI)"
read -r key payload < <("$BUILD_DIR/whisper_client" payload "$link")
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
burn_link="$("$BUILD_DIR/whisper_client" create "$SERVER" "one-time" burn)"
check "first read" "one-time" "$("$BUILD_DIR/whisper_client" open "$burn_link")"
if "$BUILD_DIR/whisper_client" open "$burn_link" >/dev/null 2>&1; then
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
