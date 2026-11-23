import XCTest

/// The first-launch tour. Needs no server: it never touches the network.
final class OnboardingTests: XCTestCase {
    private var app: XCUIApplication!

    override func setUpWithError() throws {
        continueAfterFailure = false
        app = XCUIApplication()
        // Forces the tour on, as on a fresh install, without wiping the app.
        app.launchArguments += ["-onboarding_pending", "YES"]
        app.launch()
    }

    func testTourWalksThroughToTheComposer() {
        let next = app.buttons["onboarding-next"]
        XCTAssertTrue(next.waitForExistence(timeout: 10), "the tour never appeared")
        XCTAssertTrue(app.buttons["onboarding-skip"].isHittable)

        for _ in 0..<3 { next.tap() }
        XCTAssertEqual(next.label, "Get started")
        let skip = app.buttons["onboarding-skip"]
        XCTAssertFalse(skip.exists && skip.isHittable, "Skip is shown on the last page")
        next.tap()

        XCTAssertTrue(
            app.textViews["secret-editor"].waitForExistence(timeout: 10),
            "finishing the tour did not land on the composer")
    }

    func testSkipLeavesTheTour() {
        let skip = app.buttons["onboarding-skip"]
        XCTAssertTrue(skip.waitForExistence(timeout: 10), "the tour never appeared")
        skip.tap()
        XCTAssertTrue(
            app.textViews["secret-editor"].waitForExistence(timeout: 10),
            "skipping the tour did not land on the composer")
    }
}
