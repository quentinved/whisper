use clap_complete::{generate, Shell};
use std::io;

/// Generate shell completions. This function takes the shell type
/// and the Clap Command (from CommandFactory).
pub fn run(shell: Shell, cmd: &mut clap::Command) {
    generate(shell, cmd, "whisper-secrets", &mut io::stdout());
}
