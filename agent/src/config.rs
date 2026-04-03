use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::error::{AgentError, Result};

/// Top-level configuration loaded from `config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub watch:    WatchConfig,
    pub scoring:  ScoringConfig,
    pub canary:   CanaryConfig,
    pub response: ResponseConfig,
    pub snapshot: SnapshotConfig,
    pub report:   ReportConfig,
    pub log:      LogConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watch:    WatchConfig::default(),
            scoring:  ScoringConfig::default(),
            canary:   CanaryConfig::default(),
            response: ResponseConfig::default(),
            snapshot: SnapshotConfig::default(),
            report:   ReportConfig::default(),
            log:      LogConfig::default(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path).map_err(|e| AgentError::Config {
            path: path.display().to_string(),
            source: Box::new(e),
        })?;
        let cfg: Config = toml::from_str(&raw).map_err(|e| AgentError::Config {
            path: path.display().to_string(),
            source: Box::new(e),
        })?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn load_or_default(path: &Path) -> Self {
        if path.exists() {
            Self::load(path).unwrap_or_else(|e| {
                eprintln!("[warn] Failed to load config ({e}), using defaults");
                Self::default()
            })
        } else {
            Self::default()
        }
    }

    fn validate(&self) -> Result<()> {
        if self.scoring.elevated_threshold >= self.scoring.high_threshold {
            return Err(AgentError::ConfigInvalid(
                "elevated_threshold must be < high_threshold".into(),
            ));
        }
        if self.scoring.high_threshold >= self.scoring.critical_threshold {
            return Err(AgentError::ConfigInvalid(
                "high_threshold must be < critical_threshold".into(),
            ));
        }
        Ok(())
    }

    /// Return the default config file path for the current platform.
    pub fn default_path() -> PathBuf {
        #[cfg(windows)]
        {
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
            PathBuf::from(appdata).join("AegisOne").join("config.toml")
        }
        #[cfg(not(windows))]
        {
            dirs_next::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("aegisone")
                .join("config.toml")
        }
    }
}

// ── Watch ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchConfig {
    /// Directories to watch recursively.
    pub paths: Vec<PathBuf>,
    /// File extensions to skip entirely (no entropy analysis).
    pub skip_extensions: Vec<String>,
    /// Skip entropy analysis for files larger than this (bytes).
    pub entropy_max_file_bytes: u64,
    /// How many bytes to sample from the start of a file for entropy.
    pub entropy_sample_bytes: usize,
    /// Debounce window in milliseconds.
    pub debounce_ms: u64,
}

impl Default for WatchConfig {
    fn default() -> Self {
        let user_home = home_dir();
        let mut paths = vec![
            user_home.join("Documents"),
            user_home.join("Desktop"),
            user_home.join("Pictures"),
            user_home.join("Downloads"),
        ];
        // Only include paths that exist
        paths.retain(|p| p.exists());
        if paths.is_empty() {
            paths.push(user_home);
        }

        Self {
            paths,
            skip_extensions: vec![
                ".mp4".into(), ".mp3".into(), ".mkv".into(), ".avi".into(),
                ".zip".into(), ".7z".into(), ".gz".into(), ".rar".into(),
                ".jpg".into(), ".jpeg".into(), ".png".into(), ".gif".into(),
                ".exe".into(), ".dll".into(), ".sys".into(),
            ],
            entropy_max_file_bytes: 50 * 1024 * 1024, // 50 MB
            entropy_sample_bytes: 65_536,
            debounce_ms: 200,
        }
    }
}

// ── Scoring ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScoringConfig {
    pub window_secs:           u64,
    pub critical_threshold:    u32,
    pub high_threshold:        u32,
    pub elevated_threshold:    u32,
    pub entropy_high_bits:     f64,
    pub entropy_medium_bits:   f64,
    pub burst_high_files_sec:  f32,
    pub burst_med_files_sec:   f32,
    pub multi_dir_bonus_dirs:  usize,
    pub multi_dir_bonus_secs:  u64,
    pub multi_dir_bonus_pts:   u32,
    pub known_ransomware_extensions: Vec<String>,
    pub ransom_note_filenames:       Vec<String>,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            window_secs:          30,
            critical_threshold:   80,
            high_threshold:       50,
            elevated_threshold:   25,
            entropy_high_bits:    7.2,
            entropy_medium_bits:  6.5,
            burst_high_files_sec: 50.0,
            burst_med_files_sec:  20.0,
            multi_dir_bonus_dirs: 3,
            multi_dir_bonus_secs: 10,
            multi_dir_bonus_pts:  15,
            known_ransomware_extensions: vec![
                ".locked".into(), ".encrypted".into(), ".crypt".into(),
                ".ryk".into(), ".clop".into(), ".ransom".into(),
                ".pay2me".into(), ".wcry".into(), ".wncry".into(),
                ".lck".into(), ".cerber".into(), ".locky".into(),
                ".zepto".into(), ".odin".into(), ".aesir".into(),
                ".shit".into(), ".thor".into(), ".micro".into(),
                ".cryptolocker".into(), ".cryptowall".into(),
            ],
            ransom_note_filenames: vec![
                "README.txt".into(), "DECRYPT.txt".into(),
                "HOW_TO_RECOVER.txt".into(), "RECOVERY.txt".into(),
                "READ_ME.txt".into(), "HELP_DECRYPT.txt".into(),
                "YOUR_FILES_ARE_ENCRYPTED.txt".into(),
                "HOW_TO_BUY.txt".into(), "RANSOM_NOTE.txt".into(),
                "!!!RESTORE_FILES!!!.txt".into(),
            ],
        }
    }
}

// ── Canary ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CanaryConfig {
    pub paths:               Vec<String>,
    pub check_interval_secs: u64,
}

impl Default for CanaryConfig {
    fn default() -> Self {
        let home = home_dir();
        Self {
            paths: vec![
                home.join("Documents").join(".aegis_canary_001").to_string_lossy().into_owned(),
                home.join("Desktop").join(".aegis_canary_002").to_string_lossy().into_owned(),
                home.join("Pictures").join(".aegis_canary_003").to_string_lossy().into_owned(),
            ],
            check_interval_secs: 5,
        }
    }
}

// ── Response ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ResponseConfig {
    /// When true, detect and score but take no response actions.
    pub dry_run:          bool,
    pub kill_on_critical: bool,
    pub block_network:    bool,
    pub auto_rollback:    bool,
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            dry_run:          false,
            kill_on_critical: true,
            block_network:    true,
            auto_rollback:    true,
        }
    }
}

// ── Snapshot ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SnapshotConfig {
    /// Create a baseline shadow copy at agent startup.
    pub create_on_start: bool,
    /// Volume to snapshot (e.g. "C:").
    pub volume: String,
    /// Maximum number of AegisOne-created shadows to retain.
    pub max_retained: usize,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            create_on_start: true,
            volume: "C:".into(),
            max_retained: 3,
        }
    }
}

// ── Report ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportConfig {
    pub output_dir:    PathBuf,
    pub use_ollama:    bool,
    pub ollama_url:    String,
    pub ollama_model:  String,
}

impl Default for ReportConfig {
    fn default() -> Self {
        #[cfg(windows)]
        let output_dir = PathBuf::from(
            std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".into()),
        )
        .join("AegisOne")
        .join("reports");
        #[cfg(not(windows))]
        let output_dir = {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".aegisone").join("reports")
        };

        Self {
            output_dir,
            use_ollama:   false,
            ollama_url:   "http://localhost:11434".into(),
            ollama_model: "llama3".into(),
        }
    }
}

// ── Log ────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    pub level:     String,
    pub json:      bool,
    pub log_dir:   Option<PathBuf>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level:   "info".into(),
            json:    false,
            log_dir: None,
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default"))
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    }
}
