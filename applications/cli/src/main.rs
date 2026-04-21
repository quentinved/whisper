use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use console::style;
use whisper_secrets::commands;
use whisper_secrets::commands::get::ShareTarget;
use whisper_secrets::telemetry;

#[derive(Parser)]
#[command(
    name = "whisper-secrets",
    about = "Zero-knowledge .env secret manager.\nEncrypt, store, and share secrets with your team — no signup, no accounts.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug logging
    #[arg(long, short, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Set up a new project: generates a .whisperrc config with a random passphrase and shares it via a Whisper link
    Init {
        /// Whisper server URL - Optional, default to https://whisper.quentinvedrenne.com
        #[arg(long)]
        url: Option<String>,
        /// Choose your own passphrase instead of generating one - Optional, default to false
        #[arg(long)]
        manual_passphrase: bool,
    },
    /// Retrieve a shared secret by its URL or ID
    Get {
        /// Whisper share URL or secret ID
        target: ShareTarget,
    },
    /// Join a project using a Whisper share link from a teammate
    Join {
        /// Whisper share URL (from `whisper-secrets init`)
        link: ShareTarget,
    },
    /// Generate a new share link to invite a teammate (re-shares the passphrase)
    Invite,
    /// Read an existing .env file and upload every entry as an encrypted secret
    Import,
    /// Encrypt and upload a single secret, or pick from untracked .env entries
    Push {
        /// Environment variable name (e.g. DATABASE_URL). Omit to pick interactively.
        name: Option<String>,
    },
    /// Download and decrypt all secrets into a .env file
    Pull,
    /// Delete a secret from the server and remove it from .env.whisper
    Remove {
        /// Environment variable name to delete
        name: String,
    },
    /// Update a secret's value in-place (prompts for the new value)
    Rotate {
        /// Environment variable name to rotate
        name: String,
    },
    /// Create a one-time secret and get a share link (like the web UI)
    Share {
        /// How long before the secret expires (e.g. 30m, 1h, 24h, 7d)
        #[arg(long, short, default_value = "1h")]
        expiration: String,
        /// Keep the secret accessible after it has been viewed
        #[arg(long)]
        no_self_destruct: bool,
    },
    /// Show the current state of tracked and local secrets
    Status,
    /// Generate shell completions
    #[command(hide = true)]
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_timer(tracing_subscriber::fmt::time::uptime())
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    let command_name = command_name(&cli.command);
    let result = match cli.command {
        Commands::Get { target } => commands::get::run(&target).await,
        Commands::Join { link } => commands::join::run(&link).await,
        Commands::Invite => commands::invite::run().await,
        Commands::Import => commands::import::run().await,
        Commands::Init {
            url,
            manual_passphrase,
        } => commands::init::run(url.as_deref(), manual_passphrase).await,
        Commands::Push { name } => commands::push::run(name.as_deref()).await,
        Commands::Pull => commands::pull::run().await,
        Commands::Remove { name } => commands::remove::run(&name).await,
        Commands::Rotate { name } => commands::rotate::run(&name).await,
        Commands::Share {
            expiration,
            no_self_destruct,
        } => commands::share::run(&expiration, no_self_destruct).await,
        Commands::Status => commands::status::run(),
        Commands::Completions { shell } => {
            commands::completions::run(shell, &mut Cli::command());
            Ok(())
        }
    };

    if let Some(handle) = telemetry::track_command(command_name, result.is_ok()) {
        let _ = tokio::time::timeout(std::time::Duration::from_millis(100), handle).await;
    }

    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

fn command_name(cmd: &Commands) -> &'static str {
    match cmd {
        Commands::Init { .. } => "init",
        Commands::Get { .. } => "get",
        Commands::Join { .. } => "join",
        Commands::Invite => "invite",
        Commands::Import => "import",
        Commands::Push { .. } => "push",
        Commands::Pull => "pull",
        Commands::Remove { .. } => "remove",
        Commands::Rotate { .. } => "rotate",
        Commands::Share { .. } => "share",
        Commands::Status => "status",
        Commands::Completions { .. } => "completions",
    }
}
