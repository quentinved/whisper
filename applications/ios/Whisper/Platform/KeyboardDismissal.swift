import SwiftUI
import UIKit

/// Tapping anywhere outside a text field or editor puts the keyboard away, app
/// wide. Installed once on the window, and it never cancels the tap, so the
/// button or toggle that was tapped still works.
struct KeyboardDismissal: UIViewRepresentable {
    func makeUIView(context: Context) -> Installer { Installer() }
    func updateUIView(_ uiView: Installer, context: Context) {}

    final class Installer: UIView, UIGestureRecognizerDelegate {
        private var installed = false

        override func didMoveToWindow() {
            super.didMoveToWindow()
            guard let window, !installed else { return }
            installed = true
            let tap = UITapGestureRecognizer(
                target: window, action: #selector(UIView.endEditing(_:)))
            tap.cancelsTouchesInView = false
            tap.delegate = self
            window.addGestureRecognizer(tap)
        }

        /// Leave taps inside a text input alone — that's the user placing the
        /// cursor, not leaving the field.
        func gestureRecognizer(
            _ gestureRecognizer: UIGestureRecognizer, shouldReceive touch: UITouch
        ) -> Bool {
            var view = touch.view
            while let current = view {
                if current is UITextView || current is UITextField { return false }
                // System alerts handle their own taps: ending editing under one
                // swallowed taps on its buttons (Save in the "Name this secret"
                // prompt never fired).
                if current.next is UIAlertController { return false }
                view = current.superview
            }
            return true
        }

        func gestureRecognizer(
            _ gestureRecognizer: UIGestureRecognizer,
            shouldRecognizeSimultaneouslyWith other: UIGestureRecognizer
        ) -> Bool { true }
    }
}
