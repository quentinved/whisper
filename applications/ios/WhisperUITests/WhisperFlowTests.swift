import XCTest

/// Drives the real app against a real Whisper server, through the UI, the same
/// way a person would.
///
/// The server defaults to a local instance and the tests skip themselves if it
/// is not running. That default is deliberate: these tests create real secrets,
/// and an accidental run must never write them to production. Override it only
/// on purpose:
///
///   xcodebuild test -project Whisper.xcodeproj -scheme Whisper \
///     -destination 'id=<simulator udid>' \
///     TEST_RUNNER_WHISPER_SERVER=http://localhost:1212
final class WhisperFlowTests: XCTestCase {
    private var app: XCUIApplication!

    /// `TEST_RUNNER_*` injection does not always reach the runner, so a local
    /// server is assumed rather than falling back to the app's own default —
    /// which is production.
    private var server: String {
        ProcessInfo.processInfo.environment["WHISPER_SERVER"] ?? "http://localhost:1212"
    }

    override func setUpWithError() throws {
        continueAfterFailure = false
        try skipUnlessServerIsReachable()

        app = XCUIApplication()
        app.launchEnvironment["WHISPER_SERVER"] = server
        app.launchEnvironment["WHISPER_DISABLE_BIOMETRIC_GATE"] = "1"
        // UserDefaults argument domain: overrides `OnboardingView.pendingKey`
        // for this run only, so the first-launch tour doesn't cover the tabs.
        app.launchArguments += ["-onboarding_pending", "NO"]
        app.launch()

        try prepareApp()
    }

    /// The whole point of the app, end to end: seal a secret here, get a link,
    /// then open that link here and get the same text back. If the crypto,
    /// the link format or the API contract were wrong, this would fail.
    func testRoundTripThroughTheUI() throws {
        let secret = "correct-horse-battery-staple-\(Int.random(in: 1000...9999))"

        let editor = app.textViews["secret-editor"]
        XCTAssertTrue(editor.waitForExistence(timeout: 10), "composer never appeared")
        editor.tap()
        editor.typeText(secret)
        capture("01-compose")

        app.buttons["create-button"].tap()

        let linkLabel = app.staticTexts["share-link"]
        XCTAssertTrue(linkLabel.waitForExistence(timeout: 20), "no link was produced")
        let link = linkLabel.label
        XCTAssertTrue(link.contains("shared_secret_id="), "link is missing the id: \(link)")
        XCTAssertTrue(link.contains("#k="), "link is missing the key fragment: \(link)")
        capture("02-link-ready")

        app.buttons["Open"].tap()

        let field = app.textFields["link-field"]
        XCTAssertTrue(field.waitForExistence(timeout: 10), "link field never appeared")
        field.tap()
        field.typeText(link)
        app.buttons["continue-button"].tap()

        let reveal = app.buttons["reveal-button"]
        XCTAssertTrue(reveal.waitForExistence(timeout: 20), "confirmation step never appeared")
        capture("03-confirm")

        reveal.tap()

        let toggle = app.buttons["reveal-toggle"]
        XCTAssertTrue(toggle.waitForExistence(timeout: 20), "the secret was never decrypted")
        XCTAssertFalse(
            app.staticTexts["revealed-secret"].exists,
            "the secret must stay hidden until the user asks to see it")
        capture("04-hidden")

        toggle.tap()
        let revealed = app.staticTexts["revealed-secret"]
        XCTAssertTrue(revealed.waitForExistence(timeout: 5), "tapping Show did not reveal it")
        XCTAssertEqual(revealed.label, secret, "the decrypted secret does not match")
        capture("05-revealed")
    }

    /// A revealed secret must not sit in the app until the next link: once its
    /// time is up, the screen forgets the plaintext on its own.
    func testRevealedSecretClearsItself() throws {
        app.terminate()
        app.launchEnvironment["WHISPER_REVEAL_LIFETIME"] = "5"
        app.launch()

        let editor = app.textViews["secret-editor"]
        XCTAssertTrue(editor.waitForExistence(timeout: 10), "composer never appeared")
        editor.tap()
        editor.typeText("short-lived-\(Int.random(in: 1000...9999))")
        app.buttons["create-button"].tap()

        let linkLabel = app.staticTexts["share-link"]
        XCTAssertTrue(linkLabel.waitForExistence(timeout: 20), "no link was produced")
        openLink(linkLabel.label)

        let reveal = app.buttons["reveal-button"]
        XCTAssertTrue(reveal.waitForExistence(timeout: 20), "confirmation step never appeared")
        reveal.tap()

        XCTAssertTrue(
            app.buttons["reveal-toggle"].waitForExistence(timeout: 20),
            "the secret was never decrypted")
        XCTAssertTrue(
            app.textFields["link-field"].waitForExistence(timeout: 20),
            "the revealed secret never cleared itself")
        XCTAssertFalse(app.buttons["reveal-toggle"].exists, "the plaintext is still on screen")
    }

    /// The web's "Multiple Values" mode, both ways: build key/value pairs here,
    /// and read them back as separate, individually revealed fields.
    func testMultipleValuesRoundTrip() throws {
        let password = "s3cr3t-\(Int.random(in: 1000...9999))"

        app.buttons["Multiple Values"].tap()
        let key = app.textFields["kv-0-key"]
        XCTAssertTrue(key.waitForExistence(timeout: 10), "the key/value builder never appeared")
        key.tap()
        key.typeText("DB_PASSWORD")
        let value = app.textFields["kv-0-value"]
        value.tap()
        value.typeText(password)
        capture("08-multiple-values")

        app.buttons["create-button"].tap()
        let linkLabel = app.staticTexts["share-link"]
        XCTAssertTrue(linkLabel.waitForExistence(timeout: 20), "no link was produced")

        openLink(linkLabel.label)
        let reveal = app.buttons["reveal-button"]
        XCTAssertTrue(reveal.waitForExistence(timeout: 20))
        reveal.tap()

        let toggle = app.buttons["field-toggle-DB_PASSWORD"]
        XCTAssertTrue(toggle.waitForExistence(timeout: 20), "the secret wasn't shown as fields")
        XCTAssertFalse(app.staticTexts["field-value-DB_PASSWORD"].exists, "fields must start hidden")
        toggle.tap()
        let shown = app.staticTexts["field-value-DB_PASSWORD"]
        XCTAssertTrue(shown.waitForExistence(timeout: 5))
        XCTAssertEqual(shown.label, password)
        capture("09-fields")
    }

    /// The sender's side: a link starts out "Waiting", and once someone reveals
    /// it the Sent tab says "Opened" — without any account. Needs a server with
    /// delivery status (v1.5.0+).
    func testSentTabShowsWhenTheSecretWasOpened() throws {
        let secret = "delivered-\(Int.random(in: 1000...9999))"

        let editor = app.textViews["secret-editor"]
        XCTAssertTrue(editor.waitForExistence(timeout: 10))
        editor.tap()
        editor.typeText(secret)
        app.buttons["create-button"].tap()

        let linkLabel = app.staticTexts["share-link"]
        XCTAssertTrue(linkLabel.waitForExistence(timeout: 20))
        let link = linkLabel.label

        let status = app.descendants(matching: .any)
            .matching(identifier: "delivery-status").firstMatch
        waitForLabel(of: status, toContain: "Waiting")
        capture("10-delivery-waiting")

        openLink(link)
        let reveal = app.buttons["reveal-button"]
        XCTAssertTrue(reveal.waitForExistence(timeout: 20), "confirmation never appeared")
        reveal.tap()
        XCTAssertTrue(app.buttons["reveal-toggle"].waitForExistence(timeout: 20), "the secret was never opened")
        app.buttons["Done"].tap()

        app.buttons["Sent"].tap()
        let newest = app.descendants(matching: .any).matching(identifier: "sent-row").firstMatch
        waitForLabel(of: newest, toContain: "Opened")
        capture("11-sent-opened")
    }

    /// Tapping empty space must put the keyboard away — the composer is a
    /// multi-line editor, so Return only adds a line.
    func testTappingOutsideDismissesTheKeyboard() throws {
        let editor = app.textViews["secret-editor"]
        XCTAssertTrue(editor.waitForExistence(timeout: 10), "composer never appeared")
        editor.tap()
        editor.typeText("typing…")
        XCTAssertTrue(app.keyboards.element.waitForExistence(timeout: 5), "no keyboard to dismiss")

        app.staticTexts["Built to Disappear."].tap()

        let gone = NSPredicate(format: "exists == false")
        expectation(for: gone, evaluatedWith: app.keyboards.element)
        waitForExpectations(timeout: 5)
    }

    /// A burnt secret must not come back a second time. This is the guarantee
    /// users are relying on, so it is asserted rather than assumed.
    func testSelfDestructSecretCannotBeOpenedTwice() throws {
        let secret = "one-shot-\(Int.random(in: 1000...9999))"

        let editor = app.textViews["secret-editor"]
        XCTAssertTrue(editor.waitForExistence(timeout: 10))
        editor.tap()
        editor.typeText(secret)
        app.buttons["create-button"].tap()

        let linkLabel = app.staticTexts["share-link"]
        XCTAssertTrue(linkLabel.waitForExistence(timeout: 20))
        let link = linkLabel.label

        openLink(link)
        let firstReveal = app.buttons["reveal-button"]
        XCTAssertTrue(firstReveal.waitForExistence(timeout: 20), "confirmation never appeared")
        firstReveal.tap()

        let toggle = app.buttons["reveal-toggle"]
        XCTAssertTrue(toggle.waitForExistence(timeout: 20), "first read failed")
        toggle.tap()
        let revealed = app.staticTexts["revealed-secret"]
        XCTAssertTrue(revealed.waitForExistence(timeout: 5))
        XCTAssertEqual(revealed.label, secret)

        app.buttons["Done"].tap()
        openLink(link)

        // The second attempt must not reach a secret. Either the server reports
        // it gone during the non-consuming check, or the fetch itself fails —
        // both are correct, so assert on the outcome rather than the route.
        let secondReveal = app.buttons["reveal-button"]
        if secondReveal.waitForExistence(timeout: 5) {
            secondReveal.tap()
        }
        XCTAssertTrue(
            app.staticTexts["secret-gone"].waitForExistence(timeout: 10),
            "a burnt secret should land on the gone screen"
        )
        XCTAssertFalse(
            app.buttons["reveal-toggle"].exists,
            "a burn-after-reading secret was served twice"
        )
        capture("06-burned")
    }

    func testSettingsShowsTheConfiguredServer() throws {
        app.buttons["Settings"].tap()
        let field = app.textFields["server-field"]
        XCTAssertTrue(field.waitForExistence(timeout: 10), "settings never appeared")
        XCTAssertEqual(field.value as? String, server, "the app is pointed at the wrong server")
        capture("07-settings")
    }

    // MARK: - Helpers

    /// Refuses to run against an unreachable server instead of silently
    /// exercising whatever the app defaults to.
    private func skipUnlessServerIsReachable() throws {
        guard let url = URL(string: "\(server)/health") else {
            throw XCTSkip("WHISPER_SERVER is not a valid URL: \(server)")
        }

        var request = URLRequest(url: url)
        request.timeoutInterval = 5

        var reachable = false
        let done = DispatchSemaphore(value: 0)
        URLSession.shared.dataTask(with: request) { _, response, _ in
            reachable = (response as? HTTPURLResponse)?.statusCode == 200
            done.signal()
        }.resume()
        _ = done.wait(timeout: .now() + 10)

        guard reachable else {
            throw XCTSkip(
                "No Whisper server at \(server) — start one before running the UI tests.")
        }
    }

    /// Confirms the app came up pointed where the test intends, before any
    /// secret is created.
    private func prepareApp() throws {
        app.buttons["Settings"].tap()
        let field = app.textFields["server-field"]
        XCTAssertTrue(field.waitForExistence(timeout: 10), "settings never appeared")
        XCTAssertEqual(field.value as? String, server, "the app is pointed at the wrong server")
        app.buttons["Send"].tap()
    }

    /// Statuses load asynchronously ("Checking" first), so poll the label.
    private func waitForLabel(of element: XCUIElement, toContain text: String) {
        let predicate = NSPredicate(format: "label CONTAINS %@", text)
        let expectation = XCTNSPredicateExpectation(predicate: predicate, object: element)
        XCTAssertEqual(
            XCTWaiter.wait(for: [expectation], timeout: 20), .completed,
            "expected “\(text)”, got “\(element.exists ? element.label : "nothing")”")
    }

    private func openLink(_ link: String) {
        app.buttons["Open"].tap()
        let field = app.textFields["link-field"]
        XCTAssertTrue(field.waitForExistence(timeout: 10))
        field.tap()
        XCTAssertTrue(
            app.keyboards.element.waitForExistence(timeout: 5), "the keyboard never appeared")

        // The field may still hold a previous attempt. Clear it with deletes:
        // a press-and-hold "Select All" drops keyboard focus, and the following
        // typeText then fails to dispatch.
        if let existing = field.value as? String, existing.contains("shared_secret_id") {
            field.typeText(
                String(repeating: XCUIKeyboardKey.delete.rawValue, count: existing.count + 4))
        }

        field.typeText(link)
        app.buttons["continue-button"].tap()
    }

    /// Saves a screenshot both as a test attachment and, when a destination is
    /// configured, as a plain PNG so the design can be reviewed outside Xcode.
    private func capture(_ name: String) {
        let screenshot = XCUIScreen.main.screenshot()

        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)

        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("whisper-screenshots", isDirectory: true)
        try? FileManager.default.createDirectory(
            at: directory, withIntermediateDirectories: true)
        try? screenshot.pngRepresentation.write(
            to: directory.appendingPathComponent("\(name).png"))
    }
}
