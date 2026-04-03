use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Primitives ─────────────────────────────────────────────────────────────────

pub type EventId = u64;

// ── Filesystem events ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RawFsEvent {
    pub id:      EventId,
    pub ts:      DateTime<Utc>,
    pub path:    PathBuf,
    pub kind:    FsEventKind,
    /// Originating PID when determinable (Windows only via handle tracking)
    pub pid:     Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsEventKind {
    Created,
    Modified,
    Renamed { from: PathBuf },
    Deleted,
}

// ── Process info ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid:        u32,
    pub parent_pid: u32,
    pub name:       String,
    pub exe_path:   Option<PathBuf>,
}

// ── Threat signals ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ThreatSignal {
    HighEntropy {
        path:    PathBuf,
        entropy: f64,
        pid:     Option<u32>,
    },
    BurstWrites {
        path:          PathBuf,
        writes_per_sec: f32,
        pid:           Option<u32>,
    },
    ExtensionChange {
        path:    PathBuf,
        old_ext: String,
        new_ext: String,
        pid:     Option<u32>,
    },
    CanaryTouched {
        canary_path: PathBuf,
        pid:         Option<u32>,
    },
    ShadowDeleteAttempt {
        command_line: String,
        pid:          u32,
    },
    SuspiciousRename {
        path: PathBuf,
        pid:  Option<u32>,
    },
    RansomNoteCreated {
        path: PathBuf,
        pid:  Option<u32>,
    },
}

impl ThreatSignal {
    pub fn pid(&self) -> Option<u32> {
        match self {
            Self::HighEntropy { pid, .. }       => *pid,
            Self::BurstWrites { pid, .. }       => *pid,
            Self::ExtensionChange { pid, .. }   => *pid,
            Self::CanaryTouched { pid, .. }     => *pid,
            Self::ShadowDeleteAttempt { pid, .. } => Some(*pid),
            Self::SuspiciousRename { pid, .. }  => *pid,
            Self::RansomNoteCreated { pid, .. } => *pid,
        }
    }

    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::HighEntropy { path, .. }       => Some(path),
            Self::BurstWrites { path, .. }       => Some(path),
            Self::ExtensionChange { path, .. }   => Some(path),
            Self::CanaryTouched { canary_path, .. } => Some(canary_path),
            Self::ShadowDeleteAttempt { .. }     => None,
            Self::SuspiciousRename { path, .. }  => Some(path),
            Self::RansomNoteCreated { path, .. } => Some(path),
        }
    }
}

// ── Scoring ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Score(pub u32);

impl std::fmt::Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add for Score {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self(self.0 + rhs.0) }
}

impl std::ops::AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0; }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    Nominal,   // 0–24
    Elevated,  // 25–49
    High,      // 50–79
    Critical,  // 80+
}

impl ThreatLevel {
    /// Determine the threat level from a raw score and threshold values.
    /// Thresholds are typically sourced from `ScoringConfig`.
    pub fn from_score(score: Score, elevated: u32, high: u32, critical: u32) -> Self {
        let s = score.0;
        if s >= critical      { ThreatLevel::Critical }
        else if s >= high     { ThreatLevel::High }
        else if s >= elevated { ThreatLevel::Elevated }
        else                  { ThreatLevel::Nominal }
    }

    /// Convenience wrapper that reads thresholds from `ScoringConfig`.
    pub fn from_score_cfg(score: Score, cfg: &crate::config::ScoringConfig) -> Self {
        Self::from_score(score, cfg.elevated_threshold, cfg.high_threshold, cfg.critical_threshold)
    }
}

impl std::fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nominal  => write!(f, "NOMINAL"),
            Self::Elevated => write!(f, "ELEVATED"),
            Self::High     => write!(f, "HIGH"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A signal that has been scored and attributed to a window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredEvent {
    pub id:         EventId,
    pub ts:         DateTime<Utc>,
    pub signal:     ThreatSignal,
    pub delta:      u32,
    pub cumulative: Score,
    pub level:      ThreatLevel,
}

// ── Incident ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id:             Uuid,
    pub started_at:     DateTime<Utc>,
    pub closed_at:      DateTime<Utc>,
    pub peak_level:     ThreatLevel,
    pub final_score:    Score,
    pub events:         Vec<ScoredEvent>,
    pub affected_paths: Vec<PathBuf>,
    pub primary_pid:    Option<u32>,
    pub process_info:   Option<ProcessInfo>,
    pub actions_taken:  Vec<ResponseAction>,
    pub snapshot_used:  Option<SnapshotInfo>,
    pub report_path:    Option<PathBuf>,
}

// ── Response actions ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ResponseAction {
    ProcessTreeKilled {
        root_pid:    u32,
        killed_pids: Vec<u32>,
    },
    NetworkBlocked {
        pid:       u32,
        rule_name: String,
    },
    RollbackStarted {
        snapshot_id:  String,
        target_paths: Vec<PathBuf>,
    },
    RollbackCompleted {
        files_restored: usize,
        manifest_path:  PathBuf,
    },
    ReportGenerated {
        path: PathBuf,
    },
    DryRun {
        would_have: String,
    },
}

// ── Snapshot ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub shadow_id:   String,
    pub volume:      String,
    pub device_path: PathBuf,
    pub created_at:  DateTime<Utc>,
    pub size_bytes:  Option<u64>,
}

// ── Channel messages ───────────────────────────────────────────────────────────

/// Sent from scorer → ResponseOrchestrator
#[derive(Debug)]
pub struct AlertMsg {
    pub event:    ScoredEvent,
    /// Populated when level transitions to Critical
    pub incident: Option<Incident>,
}

/// Sent from ResponseOrchestrator → ReportGenerator
#[derive(Debug)]
pub struct ReportRequest {
    pub incident:   Incident,
    pub use_ollama: bool,
}

/// Snapshot creation request (with oneshot reply)
#[derive(Debug)]
pub struct SnapshotCmd {
    pub volume: String,
    pub reply:  tokio::sync::oneshot::Sender<crate::error::Result<SnapshotInfo>>,
}

/// Supervisor control messages
#[derive(Debug)]
pub enum ControlMsg {
    Shutdown { reason: String },
    SubsystemError { subsystem: &'static str, error: String },
    HealthPing,
}
