# Whisper for iPhone

A SwiftUI prototype of Whisper's zero-knowledge secret sharing. Secrets are
sealed on the device and the key never reaches the server, so a link created
here opens in the web UI, and a link from the CLI, Slack or Discord opens here.

Status: **working prototype.** Sending and opening secrets both work end to end
against a real server. See [What is not built yet](#what-is-not-built-yet).

## The one thing that makes this possible

Whisper's ephemeral payload is `nonce[12] ‖ ciphertext ‖ tag[16]`, base64url
without padding. That is byte-for-byte what CryptoKit calls
`AES.GCM.SealedBox.combined`, so the app needs no bridging to Rust, no
third-party crypto and no manual framing:

```swift
let key = SymmetricKey(size: .bits256)
let box = try AES.GCM.seal(Data(plaintext.utf8), using: key)
let payload = box.combined!   // exactly what POST /v1/ephemeral expects
```

Nothing in the type system enforces that agreement across languages, so
`Tools/interop_check.sh` enforces it instead.

## Running it

Requires Xcode 16 or newer (built against Xcode 26.6) and an iOS 17+ simulator
or device.

Signing matches the other apps in this account — `DEVELOPMENT_TEAM =
7WBNUC3KR9`, automatic signing, bundle id `com.quentinvedrenne.whisper`
(`…​.uitests` for the test bundle), the same convention as Scopa and Who's
First. A device build signs against the existing wildcard "iOS Team
Provisioning Profile: *", so running on hardware needs no new App ID.

```bash
open applications/ios/Whisper.xcodeproj    # then ⌘R
```

From the command line:

```bash
xcodebuild -project applications/ios/Whisper.xcodeproj -scheme Whisper \
  -destination 'id=<simulator udid>' build
```

By default the app talks to `https://whisper.quentinvedrenne.com`. Change it in
Settings, or point a single run at a local server without touching Settings:

```bash
SIMCTL_CHILD_WHISPER_SERVER=http://localhost:1212 \
  xcrun simctl launch <udid> com.quentinvedrenne.whisper
```

### A local server to test against

```bash
docker-compose up -d
openssl rand -out aes_key.bin 32 && chmod 600 aes_key.bin
cargo run --bin whisper-server -- \
  --url-postgresql "postgres://postgres:<password>@localhost/whisper" \
  --base-url "http://localhost:1212"
```

## Tests

| Command | What it proves |
|---|---|
| `Tools/interop_check.sh` | The Swift and Rust payload formats agree, in both directions, including Unicode and multiline secrets, and a wrong key is rejected. |
| `Tools/e2e_check.sh [server]` | The app's own `EphemeralCrypto`, `WhisperLink` and `WhisperClient` work against a running server: round trip, Rust can read what iOS wrote, the key never appears in stored bytes, and burn-after-reading really burns. |
| `xcodebuild test …` | The UI itself: compose → link → open → reveal, a one-time secret refusing a second read, and the Sent tab turning "Waiting" into "Opened" once the link is revealed. |

```bash
applications/ios/Tools/interop_check.sh
applications/ios/Tools/e2e_check.sh http://localhost:1212
xcodebuild test -project applications/ios/Whisper.xcodeproj -scheme Whisper \
  -destination 'id=<udid>'
```

The UI tests create real secrets, so they default to `http://localhost:1212` and
skip themselves when nothing answers there — they never fall back to the app's
own default, which is production. Override only on purpose, with
`TEST_RUNNER_WHISPER_SERVER=…`, and note that this injection does not reach the
runner on every Xcode version; the tests assert in
`testSettingsShowsTheConfiguredServer` that the app really is pointed where you
think it is.

They also set `WHISPER_DISABLE_BIOMETRIC_GATE=1` in the app's launch
environment, because a simulator with biometry enrolled raises a Face ID prompt
that a UI test cannot satisfy. Like `WHISPER_SERVER`, it is read once in
`AppSettings.init` and is absent from any normal run.

## Layout

```
Whisper/
  Core/         Crypto, link parsing, HTTP, settings — no SwiftUI, so the
                test tools can compile these files directly
  Design/       Theme.swift, the iOS mirror of assets/css/modern.css
  Components/   Button styles, cards, banners
  Features/     Send, Sent, Receive, Settings
Config/         Info.plist (URL scheme, Face ID string)
Tools/          interop_check.sh, e2e_check.sh, sync_brand.swift
WhisperUITests/ End-to-end UI tests
```

`Core` deliberately imports nothing from SwiftUI or UIKit except where it must
(`Clipboard`), which is what lets `Tools/e2e_check.sh` build a macOS CLI out of
the very files the app ships rather than a copy of them.

## Design

`Design/Theme.swift` maps one-to-one onto the CSS custom properties in
`applications/axum/server/assets/css/modern.css`: the same monochrome palette,
the same radii, the same reservation of colour for warnings and errors only.
Change a token there and change it here; do not invent new shades.

## Security decisions worth knowing

- **The key is never sent.** It lives in the URL fragment and in
  `WhisperLink.key`; only `id` is ever put in a request.
- **Nothing is fetched on a tap that only meant "I have a link".** Opening a
  burn-after-reading secret is irreversible, so the app checks
  `GET /secret/:id/meta` first and stops at a confirmation screen. Servers
  predating that endpoint answer `nil`, and the warning hedges instead of
  promising.
- **Delivery status keeps no link.** Creating a secret asks the server to track
  delivery (`track_delivery: true`) and gets back a status token that can only
  read whether the secret was opened (`GET /v1/status/:token`). `SentStore`
  keeps that token, the server, the dates and the self-destruct flag in
  `UserDefaults` — never the link, its id or its key. Servers without the
  endpoint return no token and the secret is simply not tracked.
- **Face ID gates the reveal**, falling open on devices with no passcode or
  biometrics — it is a shoulder-surfing defence, not part of the crypto.
- **Revealed secrets are copied with a 90-second pasteboard expiry**, which also
  bounds how long they sit in Universal Clipboard. Share links are copied
  normally, since the user may need them for longer.
- **ATS allows local networking only**, for development against localhost.
  Remote traffic still requires HTTPS.

## What is not built yet

- **Universal links.** `whisper://` works today; `https://` links open Safari.
  Real universal links need an `apple-app-site-association` file served from
  the Whisper domain and a paid team ID. The fragment does survive into
  `NSUserActivity`, so no server-side change is needed beyond the AASA file.
- **A share extension** — select a password anywhere, share, get a link back.
  This is also the strongest answer to App Store guideline 4.2, which a plain
  form posting to an API can otherwise fall foul of.
- **Sent/received history.** There are no accounts, so any history would be
  local-only; the server genuinely cannot help. Worth doing only once the
  per-user keypair model lands.
- **Managed secrets** (the `whisper-secrets` vault). Feasible — the auth token
  is `hex(PBKDF2-SHA256(passphrase, "{url}:auth", 600_000))`, and CommonCrypto
  covers PBKDF2 since CryptoKit does not — but it serves a `.env`-in-a-repo
  workflow that does not belong on a phone.
- **Swift 6 language mode.** The target builds in Swift 5 mode; the concurrency
  annotations are already in place, so flipping it is a small, separate change.
- The app icon and the in-app mark come from the website's own
  `applications/axum/server/assets/logo.svg`. `Tools/sync_brand.swift` copies
  that file into the asset catalog and re-renders `icon-1024.png` from it — run
  it after changing the website logo so the two cannot drift. The logo is drawn
  as a template image so it inverts in dark mode, matching
  `[data-theme="dark"] .logo-image { filter: invert(1) }` on the web.

Export compliance: the app uses only standard AES-GCM via Apple's CryptoKit, so
`ITSAppUsesNonExemptEncryption` is declared `false` in `Config/Info.plist`.
