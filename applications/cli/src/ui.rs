use indicatif::ProgressBar;
use std::time::Duration;

const SPINNER_TICK_MS: u64 = 80;

pub fn spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(SPINNER_TICK_MS));
    spinner
}
