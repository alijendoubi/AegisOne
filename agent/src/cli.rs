use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name    = "aegisone-agent",
    version = env!("CARGO_PKG_VERSION"),
    about   = "AegisOne on-device AI security agent",
    long_about = "Detects ransomware in real time, kills process trees, rolls back encrypted files, \
                  and generates plain-language incident reports — all on your device."
)]
pub struct Cli {
    /// Path to config.toml (defaults to platform config dir)
    #[arg(short, long, value_name = "PATH", env = "AEGISONE_CONFIG")]
    pub config: Option<PathBuf>,

    /// Increase log verbosity (use multiple times for more detail)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Emit logs as newline-delimited JSON (for SIEM ingestion)
    #[arg(long)]
    pub json_logs: bool,

    /// Detect and score but take no response actions
    #[arg(long)]
    pub dry_run: bool,

    /// Disable Ollama hook even if configured
    #[arg(long)]
    pub no_ollama: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the detection and response engine (default)
    Watch {
        /// Additional paths to monitor (overrides config)
        #[arg(short, long, value_name = "PATH")]
        path: Vec<PathBuf>,
    },

    /// Show the current threat level and active score windows
    Status,

    /// Manage VSS snapshots
    Snapshot {
        #[command(subcommand)]
        action: SnapshotAction,
    },

    /// Generate a report for a past incident
    Report {
        /// Incident UUID
        incident_id: String,

        /// Use Ollama to enrich the report with AI narrative
        #[arg(long)]
        use_ollama: bool,
    },

    /// Manage canary files
    Canary {
        #[command(subcommand)]
        action: CanaryAction,
    },

    /// Show or validate the current configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum SnapshotAction {
    /// List all available shadow copies on the configured volume
    List,
    /// Create a new shadow copy now
    Create,
    /// Roll back affected files from a shadow copy
    Rollback {
        /// Shadow copy ID (e.g. `{GUID}`)
        shadow_id: String,
        /// Files to restore (all affected files if omitted)
        #[arg(short, long)]
        file: Vec<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum CanaryAction {
    /// Place canary files at configured paths
    Install,
    /// Verify canary files are intact
    Verify,
    /// Remove canary files
    Remove,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Print the resolved configuration as TOML
    Show,
    /// Validate the config file and exit
    Validate,
    /// Write the default config to the default path
    Init,
}
