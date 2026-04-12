use crate::{commands::init::share_passphrase, config::WhisperConfig, error::CliError};
use console::style;

pub async fn run() -> Result<(), CliError> {
    let config = WhisperConfig::load()?;
    let share_url = share_passphrase(&config.url, &config.passphrase).await?;

    println!(
        "{} Share this link with your teammate {}:",
        style("done").green().bold(),
        style("(expires in 24h)").dim()
    );
    println!(
        "  \x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        share_url,
        style(share_url.as_str()).underlined()
    );
    println!();
    println!(
        "  They can join with: {}",
        style("whisper-secrets join <link>").cyan()
    );

    crate::clipboard::prompt_and_copy(share_url.as_str())?;

    Ok(())
}
