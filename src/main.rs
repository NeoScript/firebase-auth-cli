#![deny(clippy::all)]

mod commands;
mod config;
mod errors;
mod firebase;
mod output;
mod prompt;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "fbadmin", version, about = "Firebase Auth administration CLI")]
pub struct Cli {
    #[arg(long, short = 'p', env = "FBADMIN_PROFILE", help = "Use a named profile from config")]
    pub profile: Option<String>,

    #[arg(long, env = "FBADMIN_PROJECT", help = "Firebase project ID (uses ADC)")]
    pub project: Option<String>,

    #[arg(long, short = 'c', env = "FBADMIN_CREDENTIALS", help = "Path to service account JSON")]
    pub credentials: Option<String>,

    #[arg(
        long,
        short = 'e',
        env = "FBADMIN_EMULATOR_HOST",
        help = "Connect to emulator (host:port)"
    )]
    pub emulator_host: Option<String>,

    #[arg(long, short = 'f', value_enum, default_value_t = OutputFormat::Table, help = "Output format")]
    pub format: OutputFormat,

    #[arg(long, help = "Preview destructive operations without executing")]
    pub dry_run: bool,

    #[arg(long, short = 'y', help = "Skip confirmation prompts")]
    pub yes: bool,

    #[arg(long, short = 'v', action = ArgAction::Count, help = "Verbosity: -v info, -vv debug, -vvv trace")]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Profile management
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Custom claims management
    Claims {
        #[command(subcommand)]
        command: ClaimsCommand,
    },
    /// User management
    Users {
        #[command(subcommand)]
        command: UsersCommand,
    },
    /// Auth action links
    Links {
        #[command(subcommand)]
        command: LinksCommand,
    },
    /// Emulator-only utilities
    Emulator {
        #[command(subcommand)]
        command: EmulatorCommand,
    },
    /// Connection info
    Info,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Guided setup wizard
    Init,
    /// Add or overwrite a profile
    Add {
        /// Profile name
        name: Option<String>,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        credentials: Option<String>,
        #[arg(long)]
        emulator_host: Option<String>,
    },
    /// Remove a profile
    Remove {
        /// Profile name
        name: Option<String>,
    },
    /// Set the default profile
    Default {
        /// Profile name
        name: Option<String>,
    },
    /// List all profiles
    List,
    /// Show details of a single profile
    Show {
        /// Profile name
        name: Option<String>,
    },
    /// Show resolved config chain
    Which,
    /// Print config file path(s)
    Path,
}

#[derive(Subcommand)]
pub enum ClaimsCommand {
    /// Show all custom claims for a user
    Get {
        #[arg(long)]
        email: Option<String>,
    },
    /// Add/update a single claim (merges with existing)
    Merge {
        /// Claim key
        key: Option<String>,
        /// Claim value (auto-detects type: bool, int, float, string, JSON)
        value: Option<String>,
        #[arg(long)]
        email: Option<String>,
    },
    /// Remove a specific claim key
    Remove {
        /// Claim key to remove
        key: Option<String>,
        #[arg(long)]
        email: Option<String>,
    },
    /// Remove ALL custom claims
    Clear {
        #[arg(long)]
        email: Option<String>,
    },
    /// Find users with a claim matching key (and optionally value)
    Find {
        /// Claim key to search for
        key: String,
        /// Optional value to match
        value: Option<String>,
        #[arg(long, help = "Only return users where this is the sole value")]
        exclusive: bool,
    },
}

#[derive(Subcommand)]
pub enum UsersCommand {
    /// Show user info
    Get {
        #[arg(long, conflicts_with = "uid")]
        email: Option<String>,
        #[arg(long, conflicts_with = "email")]
        uid: Option<String>,
    },
    /// Create a new user
    Create {
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        display_name: Option<String>,
    },
    /// Disable a user account
    Disable {
        #[arg(long)]
        email: Option<String>,
    },
    /// Re-enable a disabled user account
    Enable {
        #[arg(long)]
        email: Option<String>,
    },
    /// Bulk delete users from CSV
    Remove {
        #[arg(long)]
        csv: Option<String>,
    },
    /// List all users (paginated)
    List {
        #[arg(long)]
        limit: Option<usize>,
    },
    /// List users inactive for N days
    ListInactive {
        #[arg(long, default_value_t = 90)]
        days: u64,
    },
    /// Print total number of users
    Count,
}

#[derive(Subcommand)]
pub enum LinksCommand {
    /// Generate a password reset link
    PasswordReset {
        #[arg(long)]
        email: Option<String>,
    },
    /// Generate an email verification link
    EmailVerify {
        #[arg(long)]
        email: Option<String>,
    },
    /// Generate an email sign-in link
    SignIn {
        #[arg(long)]
        email: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum EmulatorCommand {
    /// Delete ALL users in the emulator
    ClearUsers,
    /// Show emulator configuration
    Config,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli).await {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Config { ref command } => commands::config_cmd::run(&cli, command).await,
        Commands::Claims { ref command } => commands::claims::run(&cli, command).await,
        Commands::Users { ref command } => commands::users::run(&cli, command).await,
        Commands::Links { ref command } => commands::links::run(&cli, command).await,
        Commands::Emulator { ref command } => commands::emulator::run(&cli, command).await,
        Commands::Info => commands::info::run(&cli).await,
    }
}
