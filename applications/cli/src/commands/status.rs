use crate::{config, env_whisper, error::CliError};
use console::style;
use std::collections::BTreeMap;
use std::path::Path;

pub fn run() -> Result<(), CliError> {
    config::ensure_exists()?;

    let tracked = env_whisper::read()?;
    let local = env_whisper::read_env_file();
    let env_exists = Path::new(".env").exists();

    print!("{}", render_status(&tracked, &local, env_exists));
    Ok(())
}

fn render_status(
    tracked: &BTreeMap<String, String>,
    local: &BTreeMap<String, String>,
    env_exists: bool,
) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();

    writeln!(
        out,
        "{} {} secret(s) tracked in .env.whisper",
        style("tracked:").green().bold(),
        tracked.len()
    )
    .unwrap();

    if tracked.is_empty() {
        writeln!(out).unwrap();
        writeln!(
            out,
            "  Run {} to upload your existing .env, or {} to add one.",
            style("whisper-secrets import").cyan(),
            style("whisper-secrets push <NAME>").cyan()
        )
        .unwrap();
        return out;
    }

    let tracked_keys: std::collections::BTreeSet<&String> = tracked.keys().collect();
    let local_keys: std::collections::BTreeSet<&String> = local.keys().collect();

    let needs_pull: Vec<&&String> = tracked_keys.difference(&local_keys).collect();
    if !needs_pull.is_empty() {
        writeln!(out).unwrap();
        writeln!(
            out,
            "{} {} secret(s) not in .env (run {}):",
            style("missing:").yellow().bold(),
            needs_pull.len(),
            style("whisper-secrets pull").cyan()
        )
        .unwrap();
        for name in &needs_pull {
            writeln!(out, "  - {}", style(name).yellow()).unwrap();
        }
    }

    let untracked: Vec<&&String> = local_keys.difference(&tracked_keys).collect();
    if !untracked.is_empty() {
        writeln!(out).unwrap();
        writeln!(
            out,
            "{} {} secret(s) in .env but not tracked (run {}):",
            style("untracked:").cyan().bold(),
            untracked.len(),
            style("whisper-secrets push <name>").cyan()
        )
        .unwrap();
        for name in &untracked {
            writeln!(out, "  + {}", style(name).cyan()).unwrap();
        }
    }

    if needs_pull.is_empty() && untracked.is_empty() {
        writeln!(
            out,
            "{} No missing or untracked entries (values may differ if a teammate rotated a secret — run {} to be sure)",
            style("ok:").green().bold(),
            style("whisper-secrets pull").cyan()
        )
        .unwrap();
    }

    if !env_exists {
        writeln!(out).unwrap();
        writeln!(
            out,
            "{} No .env file found. Run {} to create it.",
            style("info:").blue().bold(),
            style("whisper-secrets pull").cyan()
        )
        .unwrap();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn with_tempdir<F: FnOnce()>(f: F) {
        let _g = crate::config::CWD_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        f();
        std::env::set_current_dir(prev).unwrap();
    }

    #[test]
    fn status_errors_when_no_whisperrc() {
        with_tempdir(|| {
            let err = run().unwrap_err();
            assert!(matches!(err, CliError::MissingConfig(_)));
        });
    }

    #[test]
    fn status_zero_tracked_message_contains_next_steps() {
        with_tempdir(|| {
            fs::write(".whisperrc", "{}").unwrap();
            let msg = render_status(&BTreeMap::new(), &BTreeMap::new(), false);
            assert!(msg.contains("whisper-secrets import"), "got: {msg}");
            assert!(msg.contains("whisper-secrets push"), "got: {msg}");
        });
    }
}
