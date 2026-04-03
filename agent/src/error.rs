use thiserror::Error;

pub type Result<T> = std::result::Result<T, AgentError>;

#[derive(Debug, Error)]
pub enum AgentError {
    // ── I/O ──────────────────────────────────────────────────────────────────
    #[error("I/O error on path `{path}`: {source}")]
    Io {
        path:   String,
        #[source]
        source: std::io::Error,
    },

    #[error("I/O error: {0}")]
    IoPlain(#[from] std::io::Error),

    // ── Config ────────────────────────────────────────────────────────────────
    #[error("Failed to load config from `{path}`: {source}")]
    Config {
        path:   String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Invalid config: {0}")]
    ConfigInvalid(String),

    // ── Detection ─────────────────────────────────────────────────────────────
    #[error("Filesystem watcher failed: {0}")]
    Watcher(String),

    #[error("Entropy calculation failed for `{path}`: {reason}")]
    Entropy { path: String, reason: String },

    #[error("Canary setup failed: {0}")]
    Canary(String),

    // ── Response ──────────────────────────────────────────────────────────────
    #[error("Kill-switch failed for PID {pid}: {reason}")]
    KillSwitch { pid: u32, reason: String },

    #[error("Network block failed for PID {pid}: {reason}")]
    NetworkBlock { pid: u32, reason: String },

    // ── Snapshot / rollback ───────────────────────────────────────────────────
    #[error("VSS operation failed: {0}")]
    Vss(String),

    #[error("Rollback failed: {0}")]
    Rollback(String),

    #[error("Insufficient privileges — agent must run as Administrator")]
    NotElevated,

    // ── Report ────────────────────────────────────────────────────────────────
    #[error("Report generation failed: {0}")]
    Report(String),

    #[error("Ollama request failed: {0}")]
    Ollama(String),

    // ── Channels ──────────────────────────────────────────────────────────────
    #[error("Channel send failed: {0}")]
    ChannelSend(String),

    #[error("Channel closed unexpectedly")]
    ChannelClosed,

    // ── Serialization ─────────────────────────────────────────────────────────
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // ── Other ─────────────────────────────────────────────────────────────────
    #[error("Subsystem `{subsystem}` crashed: {reason}")]
    SubsystemCrash { subsystem: &'static str, reason: String },

    #[error("{0}")]
    Other(String),
}

impl AgentError {
    pub fn io(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io { path: path.into(), source }
    }
    pub fn kill(pid: u32, reason: impl Into<String>) -> Self {
        Self::KillSwitch { pid, reason: reason.into() }
    }
    pub fn vss(reason: impl Into<String>) -> Self {
        Self::Vss(reason.into())
    }
    pub fn other(reason: impl Into<String>) -> Self {
        Self::Other(reason.into())
    }
}
