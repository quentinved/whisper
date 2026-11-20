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
| `xcrun swift test --package-path WhisperKit` | Offline, no simulator: the commands against in-memory fakes (burn-after-reading, metadata never consumes, unknown ≠ safe), link parsing from every client, AES-GCM incl. a payload sealed by Rust, and the HTTP adapter's wire format via a stubbed `URLProtocol`. |
| `Tools/interop_check.sh` | The Swift and Rust payload formats agree, in both directions, including Unicode and multiline secrets, and a wrong key is rejected. |
| `Tools/e2e_check.sh [server]` | The app's own commands and adapters (`CreateClientEncryptedSecret`, `GetSecretById`, `AesGcm`, `HTTPSharedSecretRepository`) work against a running server: round trip, Rust can read what iOS wrote, the key never appears in stored bytes, and burn-after-reading really burns. |
| `xcodebuild test …` | The UI itself: compose → link → open → reveal, and a one-time secret refusing a second read. |

Use `xcrun swift`, not a swiftly/Homebrew `swift` on `PATH`: a toolchain that
doesn't match Xcode's SDK fails with `unknown argument: '-target-arch-variant'`.

```bash
xcrun swift test --package-path applications/ios/WhisperKit
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

Hexagonal, like the Rust workspace: a local Swift package whose targets mirror
the crates, and the app target as the application layer.

```
WhisperKit/                       Swift package (≙ the Rust workspace)
  Sources/WhisperCore/            ≙ services-core — Foundation only
    Entities/                     SharedSecret (plain | clientEncrypted)
    ValuesObject/SharedSecret/    SecretId, SecretKey, SealedSecret,
                                  SecretExpiration, SecretMetadata, WhisperLink
    Contracts/Repositories/       SharedSecretRepository (port)
    Services/                     SecretEncryption (port)
    Commands/SharedSecret/        CreateClientEncryptedSecret, GetSecretMetadata,
                                  GetSecretById
  Sources/WhisperCrypto/          ≙ adapters/aes-gcm-crypto — AesGcm (CryptoKit)
  Sources/WhisperHTTP/            ≙ adapters/postgresql-adapter —
                                  HTTPSharedSecretRepository (URLSession)
  Sources/WhisperProbe/           macOS CLI for Tools/*.sh (≙ cargo example)
  Tests/                          Swift Testing suites per target
Whisper/                          ≙ applications/axum/server — the app
  App/          WhisperApp, RootView, AppSettings, Dependencies (composition root)
  Platform/     BiometricGate, Clipboard (UIKit / LocalAuthentication)
  Design/       Theme.swift, the iOS mirror of assets/css/modern.css
  Components/   Button styles, cards, banners
  Features/     Send, Receive, Settings (views + view models)
Config/         Info.plist (URL scheme, Face ID string)
Tools/          interop_check.sh, e2e_check.sh, testflight.sh, sync_brand.swift
WhisperUITests/ End-to-end UI tests
```

Dependencies point inwards and the compiler enforces it: `WhisperCore` imports
nothing but Foundation, the adapters import only `WhisperCore`, and
`App/Dependencies.swift` is the one app file that names a concrete adapter. View
models call commands, never adapters. A link is always opened against its own
server — `Dependencies.repository(link.server)` — not the one in Settings.

## Shipping to TestFlight

```bash
applications/ios/Tools/testflight.sh
```

Archives with a `yymmdd.HHMM` build number (the team's convention) and uploads
with Xcode's signed-in account. Prerequisites: Xcode signed in to team
`7WBNUC3KR9`, and an App Store Connect app record for
`com.quentinvedrenne.whisper`. Internal testers go in the app's `internal`
TestFlight group; turn on automatic distribution there and each upload reaches
them once processed.

## Design

`Design/Theme.swift` maps text, border, status and radius tokens onto the CSS
custom properties in `applications/axum/server/assets/css/modern.css`; change a
token there and change it here. On top of that the app uses the icon's brand
colours (violet, indigo, mint) for actions, toggles, focus and the drifting
`AuroraBackground`, with frosted-glass cards over it.

Keyboard: a window-level tap anywhere outside a text input dismisses it
(`Platform/KeyboardDismissal.swift`), and every screen adds a keyboard **Done**
button, since the composer's Return key only inserts a line.

## Security decisions worth knowing

- **The key is never sent.** It lives in the URL fragment and in
  `WhisperLink.key`; only `id` is ever put in a request.
- **Nothing is fetched on a tap that only meant "I have a link".** Opening a
  burn-after-reading secret is irreversible, so the app checks
  `GET /secret/:id/meta` first and stops at a confirmation screen. Servers
  predating that endpoint answer `nil`, and the warning hedges instead of
  promising.
- **Face ID gates the reveal**, falling open on devices with no passcode or
  biometrics — it is a shoulder-surfing defence, not part of the crypto.
- **Revealed secrets are copied with a 90-second pasteboard expiry**, which also
  bounds how long they sit in Universal Clipboard. Share links are copied
  normally, since the user may need them for longer.
- **ATS allows local networking only**, for development against localhost.
  Remote traffic still requires HTTPS.

## What is not built yet

- **A share extension** — select a password anywhere, share, get a link back.
  This is also the strongest answer to App Store guideline 4.2, which a plain
  form posting to an API can otherwise fall foul of.
- **Received history.** There are no accounts, so it would be local-only; the
  server genuinely cannot help. (Sent history exists — see below.)
- **Managed secrets** (the `whisper-secrets` vault). Feasible — the auth token
  is `hex(PBKDF2-SHA256(passphrase, "{url}:auth", 600_000))`, and CommonCrypto
  covers PBKDF2 since CryptoKit does not — but it serves a `.env`-in-a-repo
  workflow that does not belong on a phone.
- **Swift 6 language mode.** The target builds in Swift 5 mode; the concurrency
  annotations are already in place, so flipping it is a small, separate change.
## Universal links

Share links (`https://whisper.quentinvedrenne.com/get_secret?…#k=…`) open in
the app instead of Safari: `Config/Whisper.entitlements` claims
`applinks:whisper.quentinvedrenne.com`, and the server answers
`GET /.well-known/apple-app-site-association` for only links carrying a `#k=`
key — Slack's server-encrypted links have none and keep opening on the web.
iOS reads that file when the app is installed, so after deploying a server
change to it, reinstall the app. `whisper://` deep links work regardless.

## Sent tab and delivery status

Every secret created here asks the server to track delivery
(`track_delivery: true`); servers from v1.5.0 answer with a status token, which
`SentStore` keeps in `UserDefaults` with the server it came from — never the
link or its key. The **Sent** tab and the **Delivery** card on "Your link is
ready" read `GET /v1/status/:token` through `GetDeliveryStatus` (Waiting,
Opened with the time, or Expired), refreshing on pull and whenever the app
returns to the foreground. Against an older server there is no token, so
nothing is tracked and the Sent tab stays empty. A token the server no longer
knows (purged 7 days after expiry) drops out of the list.

## Multiple Values and custom expiry

Like the web form, the composer has **Free Form** and **Multiple Values**:
key/value rows plus named sections, sent as a compact JSON object
(`StructuredSecret` in WhisperCore, with the web's validation messages). On the
way back, any secret whose trimmed text is a non-empty JSON object — from the
web, this app, or written by hand — is shown as fields, each masked until its
eye is tapped and copyable alone, with a Raw JSON view; key order is kept as
sent. Expiry offers the presets or **Custom…**, any date up to the server's
7-day ceiling, re-checked at send time.

## Brand

`Tools/sync_brand.swift` is the single source of the logo: it draws the mark
from geometry and writes the website's `assets/logo.svg`, the app's
`Mark.imageset` (a template the app tints with the brand gradient) and the app
icon. Run `xcrun swift applications/ios/Tools/sync_brand.swift [style]` from
the repo root; `aurora` (default), `obsidian` and `ivory` are the icon styles.

Export compliance: the app uses only standard AES-GCM via Apple's CryptoKit, so
`ITSAppUsesNonExemptEncryption` is declared `false` in `Config/Info.plist`.
