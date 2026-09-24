use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use console::style;
use whisper_secrets::commands;
use whisper_secrets::commands::get::ShareTarget;
use whisper_secrets::error::CliError;
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
    Pull {
        /// Replace changed local values without asking (for scripts, CI and AI agents)
        #[arg(long, short)]
        yes: bool,
    },
    /// Run a command with your secrets as environment variables, without writing .env
    Run {
        /// The command to run, after `--` (e.g. `whisper-secrets run -- npm start`)
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
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
    let outcome = execute(cli.command).await;

    if let Some(handle) = telemetry::track_command(command_name, outcome.is_ok()) {
        let _ = tokio::time::timeout(std::time::Duration::from_millis(100), handle).await;
    }

    let error = match outcome {
        Ok(Outcome::Done) => return,
        Ok(Outcome::HandOver(child)) => commands::run::hand_over(child).await,
        Err(e) => e,
    };
    eprintln!("{} {}", style("error:").red().bold(), error);
    std::process::exit(1);
}

/// What `main` does once a command has finished and telemetry is sent.
enum Outcome {
    Done,
    /// `run` replaces this process with the user's command, so it is the last step.
    HandOver(std::process::Command),
}

async fn execute(command: Commands) -> Result<Outcome, CliError> {
    let result = match command {
        Commands::Run { command } => {
            return commands::run::prepare(&command)
                .await
                .map(Outcome::HandOver)
        }
        Commands::Get { target } => commands::get::run(&target).await,
        Commands::Join { link } => commands::join::run(&link).await,
        Commands::Invite => commands::invite::run().await,
        Commands::Import => commands::import::run().await,
        Commands::Init {
            url,
            manual_passphrase,
        } => commands::init::run(url.as_deref(), manual_passphrase).await,
        Commands::Push { name } => commands::push::run(name.as_deref()).await,
        Commands::Pull { yes } => commands::pull::run(yes).await,
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
    result.map(|()| Outcome::Done)
}

fn command_name(cmd: &Commands) -> &'static str {
    match cmd {
        Commands::Init { .. } => "init",
        Commands::Get { .. } => "get",
        Commands::Join { .. } => "join",
        Commands::Invite => "invite",
        Commands::Import => "import",
        Commands::Push { .. } => "push",
        Commands::Pull { .. } => "pull",
        Commands::Run { .. } => "run",
        Commands::Remove { .. } => "remove",
        Commands::Rotate { .. } => "rotate",
        Commands::Share { .. } => "share",
        Commands::Status => "status",
        Commands::Completions { .. } => "completions",
    }
}
