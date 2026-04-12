use crate::{config::CONFIG_FILE, env_whisper, error::CliError};
use console::style;
use std::path::Path;

pub fn run() -> Result<(), CliError> {
    // Check .whisperrc
    if !Path::new(CONFIG_FILE).exists() {
        eprintln!(
            "{} No .whisperrc found. Run {} to get started.",
            style("error:").red().bold(),
            style("whisper-secrets init").cyan()
        );
        return Ok(());
    }

    let tracked = env_whisper::read()?;
    let local = env_whisper::read_env_file();

    let tracked_keys: std::collections::BTreeSet<&String> = tracked.keys().collect();
    let local_keys: std::collections::BTreeSet<&String> = local.keys().collect();

    // Summary
    println!(
        "{} {} secret(s) tracked in .env.whisper",
        style("tracked:").green().bold(),
        tracked.len()
    );

    // Needs pull: in .env.whisper but not in .env
    let needs_pull: Vec<&&String> = tracked_keys.difference(&local_keys).collect();
    if !needs_pull.is_empty() {
        println!();
        println!(
            "{} {} secret(s) not in .env (run {}):",
            style("missing:").yellow().bold(),
            needs_pull.len(),
            style("whisper-secrets pull").cyan()
        );
        for name in &needs_pull {
            println!("  - {}", style(name).yellow());
        }
    }

    // Untracked: in .env but not in .env.whisper
    let untracked: Vec<&&String> = local_keys.difference(&tracked_keys).collect();
    if !untracked.is_empty() {
        println!();
        println!(
            "{} {} secret(s) in .env but not tracked (run {}):",
            style("untracked:").cyan().bold(),
            untracked.len(),
            style("whisper-secrets push <name>").cyan()
        );
        for name in &untracked {
            println!("  + {}", style(name).cyan());
        }
    }

    // All names match
    if needs_pull.is_empty() && untracked.is_empty() && !tracked.is_empty() {
        println!(
            "{} No missing or untracked entries (values may differ if a teammate rotated a secret — run {} to be sure)",
            style("ok:").green().bold(),
            style("whisper-secrets pull").cyan()
        );
    }

    // No .env at all
    if !Path::new(".env").exists() && !tracked.is_empty() {
        println!();
        println!(
            "{} No .env file found. Run {} to create it.",
            style("info:").blue().bold(),
            style("whisper-secrets pull").cyan()
        );
    }

    Ok(())
}
