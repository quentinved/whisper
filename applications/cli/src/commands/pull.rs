use crate::{
    env_whisper::{self, ENV_FILE},
    error::CliError,
    session::Session,
    ui,
};
use console::style;
use indicatif::ProgressBar;
use std::collections::{BTreeMap, BTreeSet};
use std::io::IsTerminal;
use tracing::debug;

/// Download and decrypt every tracked secret into `.env`, updating tracked
/// entries in place and keeping local-only entries and comments.
/// Asks before replacing a local value, unless `yes` (scripts, CI, AI agents).
pub async fn run(yes: bool) -> Result<(), CliError> {
    crate::config::ensure_exists()?;

    let entries = env_whisper::read()?;
    debug!("Found {} entries in .env.whisper", entries.len());

    if entries.is_empty() {
        println!(
            "{} No secrets in .env.whisper.",
            style("skip:").yellow().bold()
        );
        return Ok(());
    }

    let spinner = ui::spinner("Deriving key...");
    let session = Session::load()?;
    let secrets = fetch_and_decrypt(&entries, &session, &spinner).await?;
    spinner.finish_and_clear();

    let merged = merge_env(&read_existing_env()?, &secrets)?;
    if !confirm_replacements(&merged.replaced, yes)? {
        println!("{} Aborted.", style("skip:").yellow().bold());
        return Ok(());
    }
    std::fs::write(ENV_FILE, &merged.content).map_err(CliError::EnvWrite)?;

    print_summary(entries.len(), &merged);
    Ok(())
}

fn read_existing_env() -> Result<String, CliError> {
    match std::fs::read_to_string(ENV_FILE) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(CliError::EnvRead(e)),
    }
}

fn confirm_replacements(replaced: &[String], yes: bool) -> Result<bool, CliError> {
    if replaced.is_empty() || yes {
        return Ok(true);
    }

    eprintln!(
        "{} Pull will replace your local value of:",
        style("warn:").yellow().bold()
    );
    for name in replaced {
        eprintln!("  - {}", style(name).yellow());
    }
    eprintln!();

    if !std::io::stdin().is_terminal() {
        return Err(CliError::PullNeedsConfirmation);
    }
    dialoguer::Confirm::new()
        .with_prompt("Replace them with the team's values?")
        .default(true)
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))
}

fn print_summary(pulled: usize, merged: &MergedEnv) {
    println!(
        "{} Pulled {} secrets to {}",
        style("done").green().bold(),
        style(pulled).cyan(),
        style(ENV_FILE).cyan()
    );
    if !merged.replaced.is_empty() {
        println!("     Updated {}.", merged.replaced.join(", "));
    }
    if merged.kept_local > 0 {
        println!(
            "     {} local-only {} kept.",
            style(merged.kept_local).dim(),
            if merged.kept_local == 1 {
                "entry"
            } else {
                "entries"
            }
        );
    }
}

/// Fetch and decrypt every tracked secret, returning `(name, value)` pairs.
pub(crate) async fn fetch_and_decrypt(
    entries: &BTreeMap<String, String>,
    session: &Session,
    spinner: &ProgressBar,
) -> Result<Vec<(String, String)>, CliError> {
    let mut secrets = Vec::new();

    for (name, uuid) in entries {
        spinner.set_message(format!("Pulling {}...", name));

        debug!("Fetching {} (id={})", name, uuid);
        let payload_b64 =
            session
                .client()
                .get_secret(uuid)
                .await?
                .ok_or_else(|| CliError::NotFoundOnServer {
                    name: name.clone(),
                    id: uuid.clone(),
                })?;

        let payload = base64_url::decode(&payload_b64)
            .map_err(|_| CliError::Base64(format!("Invalid base64 from server for {}", name)))?;

        let decrypted =
            session
                .crypto()
                .decrypt(&payload)
                .map_err(|e| CliError::DecryptionFailed {
                    name: name.clone(),
                    reason: e.to_string(),
                })?;
        debug!("Decrypted {}", name);

        secrets.push((name.clone(), decrypted));
    }

    Ok(secrets)
}

/// The `.env` content after a pull, and what changed.
#[derive(Debug)]
struct MergedEnv {
    content: String,
    /// Tracked names whose existing local value is replaced.
    replaced: Vec<String>,
    /// Entries that aren't tracked, kept as they are.
    kept_local: usize,
}

/// Rewrite `existing` with the pulled `secrets`: tracked entries are updated
/// in place, everything else (local-only entries, comments, blank lines) is
/// kept, and tracked names missing from `existing` are appended.
fn merge_env(existing: &str, secrets: &[(String, String)]) -> Result<MergedEnv, CliError> {
    let values = single_line_values(secrets)?;
    let mut lines = Vec::new();
    let mut written = BTreeSet::new();
    let mut replaced = BTreeSet::new();
    let mut kept_local = 0;

    for line in existing.lines() {
        let Some((name, old_value)) = parse_entry(line) else {
            lines.push(line.to_string());
            continue;
        };
        let Some((&name, &value)) = values.get_key_value(name) else {
            kept_local += 1;
            lines.push(line.to_string());
            continue;
        };
        if old_value != value {
            replaced.insert(name);
        }
        written.insert(name);
        lines.push(format!("{}={}", name, value));
    }

    for (name, value) in &values {
        if !written.contains(name) {
            lines.push(format!("{}={}", name, value));
        }
    }

    Ok(MergedEnv {
        content: lines.join("\n") + "\n",
        replaced: replaced.into_iter().map(String::from).collect(),
        kept_local,
    })
}

fn single_line_values(secrets: &[(String, String)]) -> Result<BTreeMap<&str, &str>, CliError> {
    secrets
        .iter()
        .map(|(name, value)| {
            if value.contains('\n') {
                return Err(CliError::MultilineValue { name: name.clone() });
            }
            Ok((name.as_str(), value.as_str()))
        })
        .collect()
}

/// Split a `.env` line into `(name, value)`; `None` for blank lines and comments.
fn parse_entry(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    trimmed
        .split_once('=')
        .map(|(name, value)| (name.trim(), value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secrets(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn merge_into_empty_env_writes_every_secret() {
        let merged = merge_env("", &secrets(&[("A", "1"), ("B", "2")])).unwrap();
        assert_eq!(merged.content, "A=1\nB=2\n");
        assert!(merged.replaced.is_empty());
        assert_eq!(merged.kept_local, 0);
    }

    #[test]
    fn merge_updates_in_place_and_keeps_local_entries_and_comments() {
        let existing = "# database\nDB_URL=old\n\nLOCAL_ONLY=1\nAPI_KEY=same\n";
        let pulled = secrets(&[("API_KEY", "same"), ("DB_URL", "new"), ("NEW_ONE", "x")]);

        let merged = merge_env(existing, &pulled).unwrap();

        assert_eq!(
            merged.content,
            "# database\nDB_URL=new\n\nLOCAL_ONLY=1\nAPI_KEY=same\nNEW_ONE=x\n"
        );
        assert_eq!(merged.replaced, vec!["DB_URL"]);
        assert_eq!(merged.kept_local, 1);
    }

    #[test]
    fn merge_with_identical_values_replaces_nothing() {
        let merged = merge_env("A=1\n", &secrets(&[("A", "1")])).unwrap();
        assert_eq!(merged.content, "A=1\n");
        assert!(merged.replaced.is_empty());
    }

    #[test]
    fn merge_matches_names_written_with_spaces_and_keeps_indented_comments() {
        let merged =
            merge_env("  # note\nAPI_KEY = old\n", &secrets(&[("API_KEY", "new")])).unwrap();
        assert_eq!(merged.content, "  # note\nAPI_KEY=new\n");
        assert_eq!(merged.replaced, vec!["API_KEY"]);
    }

    #[test]
    fn merge_rejects_multiline_values() {
        let err = merge_env("", &secrets(&[("CERT", "a\nb")])).unwrap_err();
        assert!(matches!(err, CliError::MultilineValue { name } if name == "CERT"));
    }
}
