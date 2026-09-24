use crate::{commands::pull, env_whisper, error::CliError, session::Session, ui};
use std::process::{Command, ExitStatus};
use tracing::debug;

/// Build `command` with every tracked secret injected as an environment variable.
/// Nothing is written to disk. Pass the result to [`hand_over`] to run it.
pub async fn prepare(command: &[String]) -> Result<Command, CliError> {
    let (program, args) = command.split_first().ok_or(CliError::MissingCommand)?;
    crate::config::ensure_exists()?;

    let secrets = load_secrets().await?;
    debug!(
        "Running {} with {} injected secrets",
        program,
        secrets.len()
    );

    let mut child = Command::new(program);
    child.args(args).envs(secrets);
    Ok(child)
}

/// Replace this process with `command`, so it receives signals (Ctrl+C,
/// SIGTERM from Docker or CI) directly and its exit code becomes ours.
/// Only returns if the command could not be started.
pub async fn hand_over(command: Command) -> CliError {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let mut command = command;
        let error = command.exec();
        spawn_error(&command, error)
    }
    #[cfg(not(unix))]
    {
        wait_and_exit(command).await
    }
}

async fn load_secrets() -> Result<Vec<(String, String)>, CliError> {
    let entries = env_whisper::read()?;
    debug!("Found {} entries in .env.whisper", entries.len());
    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let spinner = ui::spinner("Deriving key...");
    let session = Session::load()?;
    let secrets = pull::fetch_and_decrypt(&entries, &session, &spinner).await?;
    spinner.finish_and_clear();
    Ok(secrets)
}

/// Windows has no `exec`: run the command as a child and exit with its code.
/// Compiled everywhere so CI type-checks it, but only used off Unix.
#[cfg_attr(unix, allow(dead_code))]
async fn wait_and_exit(command: Command) -> CliError {
    let mut command = tokio::process::Command::from(command);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => return spawn_error(command.as_std(), e),
    };

    // The console delivers Ctrl+C to the child as well: stay alive so the
    // child can shut down cleanly and we can forward its exit code.
    loop {
        tokio::select! {
            status = child.wait() => match status {
                Ok(status) => std::process::exit(exit_code(status)),
                Err(e) => return spawn_error(command.as_std(), e),
            },
            _ = tokio::signal::ctrl_c() => debug!("Ctrl+C received, waiting for child to exit"),
        }
    }
}

fn spawn_error(command: &Command, source: std::io::Error) -> CliError {
    CliError::CommandFailed {
        program: command.get_program().to_string_lossy().into_owned(),
        source,
    }
}

/// Mirror the shell convention: a child killed by a signal exits with 128 + signal.
#[cfg_attr(unix, allow(dead_code))]
fn exit_code(status: ExitStatus) -> i32 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return 128 + signal;
        }
    }
    status.code().unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn prepare_without_a_command_errors() {
        let err = prepare(&[]).await.unwrap_err();
        assert!(matches!(err, CliError::MissingCommand), "got {err:?}");
    }

    #[cfg(unix)]
    #[test]
    fn exit_code_follows_the_shell_convention() {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(exit_code(ExitStatus::from_raw(0)), 0);
        assert_eq!(exit_code(ExitStatus::from_raw(7 << 8)), 7);
        // Killed by SIGTERM (15) → 128 + 15.
        assert_eq!(exit_code(ExitStatus::from_raw(15)), 143);
    }
}
