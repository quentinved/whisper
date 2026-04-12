use crate::error::CliError;
use console::style;
use std::io::{IsTerminal, Write};
use std::process::{Command, Stdio};

/// Prompt the user to copy a URL to the clipboard, then do it if they accept.
/// Silently skips if not running in an interactive terminal.
pub fn prompt_and_copy(url: &str) -> Result<(), CliError> {
    if !std::io::stdin().is_terminal() || !has_clipboard_tool() {
        return Ok(());
    }

    let should_copy = dialoguer::Confirm::new()
        .with_prompt("Copy link to clipboard?")
        .default(true)
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))?;

    if should_copy {
        copy_to_clipboard(url)?;
        println!("{} Copied to clipboard", style("done").green().bold());
    }

    Ok(())
}

fn has_clipboard_tool() -> bool {
    if cfg!(target_os = "macos") {
        Command::new("which")
            .arg("pbcopy")
            .stdout(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else if cfg!(target_os = "linux") {
        Command::new("which")
            .arg("xclip")
            .stdout(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else if cfg!(target_os = "windows") {
        true // clip.exe is always available on Windows
    } else {
        false
    }
}

fn copy_to_clipboard(text: &str) -> Result<(), CliError> {
    let mut child = if cfg!(target_os = "macos") {
        Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| CliError::Input(format!("Failed to run pbcopy: {}", e)))?
    } else if cfg!(target_os = "linux") {
        Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| CliError::Input(format!("Failed to run xclip: {}", e)))?
    } else if cfg!(target_os = "windows") {
        Command::new("clip")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| CliError::Input(format!("Failed to run clip: {}", e)))?
    } else {
        return Ok(());
    };

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| CliError::Input(format!("Failed to write to clipboard: {}", e)))?;
    }

    child
        .wait()
        .map_err(|e| CliError::Input(format!("Clipboard command failed: {}", e)))?;

    Ok(())
}
