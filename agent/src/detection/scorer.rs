use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::ScoringConfig;
use crate::events::types::*;
use crate::{analyze, observe};

/// Per-PID scoring window.
struct ScoreWindow {
    pid:         u32,
    score:       Score,
    started_at:  Instant,
    events:      Vec<ScoredEvent>,
    dir_set:     HashSet<PathBuf>, // distinct directories with events
    last_signal: Instant,
    peak_level:  ThreatLevel,
}

impl ScoreWindow {
    fn new(pid: u32) -> Self {
        let now = Instant::now();
        Self {
            pid,
            score: Score(0),
            started_at: now,
            events: Vec::new(),
            dir_set: HashSet::new(),
            last_signal: now,
            peak_level: ThreatLevel::Nominal,
        }
    }

    fn is_expired(&self, window_secs: u64) -> bool {
        self.started_at.elapsed().as_secs() >= window_secs
    }

    fn reset_with_decay(&mut self) {
        self.score = Score(self.score.0 / 2);
        self.events.clear();
        self.dir_set.clear();
        self.started_at = Instant::now();
        self.last_signal = Instant::now();
    }
}

/// The main scoring engine.
/// Reads `RawFsEvent`s from `raw_rx`, analyzes them, emits `AlertMsg`s.
pub async fn run(
    cfg:      ScoringConfig,
    mut raw_rx: mpsc::Receiver<RawFsEvent>,
    alert_tx:   mpsc::Sender<AlertMsg>,
    id_seq:     Arc<AtomicU64>,
    canary_paths: Vec<PathBuf>,
    dry_run:    bool,
    watch_cfg:  crate::config::WatchConfig,
) {
    let mut windows: HashMap<u32, ScoreWindow> = HashMap::new();
    // Burst tracking: pid → (count, window_start)
    let mut burst: HashMap<u32, (u32, Instant)> = HashMap::new();

    // Window expiry ticker
    let mut gc_ticker = tokio::time::interval(std::time::Duration::from_secs(5));
    gc_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            Some(event) = raw_rx.recv() => {
                process_event(
                    event, &cfg, &mut windows, &mut burst, &alert_tx,
                    &id_seq, &canary_paths, dry_run, &watch_cfg,
                ).await;
            }
            _ = gc_ticker.tick() => {
                gc_windows(&cfg, &mut windows);
            }
            else => break,
        }
    }
}

async fn process_event(
    event:        RawFsEvent,
    cfg:          &ScoringConfig,
    windows:      &mut HashMap<u32, ScoreWindow>,
    burst:        &mut HashMap<u32, (u32, Instant)>,
    alert_tx:     &mpsc::Sender<AlertMsg>,
    id_seq:       &Arc<AtomicU64>,
    canary_paths: &[PathBuf],
    dry_run:      bool,
    watch_cfg:    &crate::config::WatchConfig,
) {
    let pid = event.pid.unwrap_or(0);
    let path = &event.path;

    // ── Canary check (highest priority) ─────────────────────────────────────
    if canary_paths.iter().any(|c| c == path) {
        let signal = ThreatSignal::CanaryTouched {
            canary_path: path.clone(),
            pid: event.pid,
        };
        score_and_emit(pid, signal, 50, cfg, windows, alert_tx, id_seq, &event, dry_run).await;
        return;
    }

    // ── Ransomware extension check ───────────────────────────────────────────
    if let FsEventKind::Renamed { ref from } = event.kind {
        let new_ext = path.extension()
            .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
            .unwrap_or_default();
        let old_ext = from.extension()
            .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
            .unwrap_or_default();

        if crate::detection::patterns::is_ransomware_extension(&new_ext, &cfg.known_ransomware_extensions) {
            observe!("extension change: {} → {} (pid={pid})", from.display(), path.display());
            let signal = ThreatSignal::ExtensionChange {
                path: path.clone(),
                old_ext,
                new_ext,
                pid: event.pid,
            };
            score_and_emit(pid, signal, 40, cfg, windows, alert_tx, id_seq, &event, dry_run).await;
            return;
        }

        if crate::detection::patterns::looks_random_extension(&new_ext) {
            let signal = ThreatSignal::SuspiciousRename {
                path: path.clone(),
                pid: event.pid,
            };
            score_and_emit(pid, signal, 25, cfg, windows, alert_tx, id_seq, &event, dry_run).await;
        }
    }

    // ── Ransom note check ────────────────────────────────────────────────────
    if matches!(event.kind, FsEventKind::Created) {
        if let Some(fname) = path.file_name().map(|n| n.to_string_lossy()) {
            if crate::detection::patterns::is_ransom_note(&fname, &cfg.ransom_note_filenames) {
                observe!("ransom note created: {} (pid={pid})", path.display());
                let signal = ThreatSignal::RansomNoteCreated {
                    path: path.clone(),
                    pid: event.pid,
                };
                score_and_emit(pid, signal, 50, cfg, windows, alert_tx, id_seq, &event, dry_run).await;
                return;
            }
        }
    }

    // ── Burst write tracking ─────────────────────────────────────────────────
    if matches!(event.kind, FsEventKind::Created | FsEventKind::Modified) {
        let entry = burst.entry(pid).or_insert((0, Instant::now()));
        entry.0 += 1;
        let elapsed = entry.1.elapsed().as_secs_f32().max(0.001);
        let rate = entry.0 as f32 / elapsed;

        if elapsed > 1.0 {
            // Reset burst window every second
            *entry = (1, Instant::now());
        }

        let pts: u32 = if rate >= cfg.burst_high_files_sec {
            25
        } else if rate >= cfg.burst_med_files_sec {
            10
        } else {
            0
        };

        if pts > 0 {
            observe!("burst writes {:.0} files/s in {} (pid={pid})", rate, path.display());
            let signal = ThreatSignal::BurstWrites {
                path: path.clone(),
                writes_per_sec: rate,
                pid: event.pid,
            };
            score_and_emit(pid, signal, pts, cfg, windows, alert_tx, id_seq, &event, dry_run).await;
        }

        // Track distinct directories for multi-dir bonus
        if let Some(dir) = path.parent() {
            let win = windows.entry(pid).or_insert_with(|| ScoreWindow::new(pid));
            win.dir_set.insert(dir.to_path_buf());
            if win.dir_set.len() >= cfg.multi_dir_bonus_dirs
                && win.last_signal.elapsed().as_secs() <= cfg.multi_dir_bonus_secs
            {
                let bonus = cfg.multi_dir_bonus_pts;
                if bonus > 0 {
                    analyze!("multi-dir spread bonus +{bonus}pts (pid={pid}, dirs={})", win.dir_set.len());
                    // Apply bonus directly without emitting a new signal
                    win.score += Score(bonus);
                    win.dir_set.clear(); // reset to avoid repeated bonus
                }
            }
        }
    }

    // ── Entropy analysis (deferred to avoid blocking this hot path) ──────────
    // Entropy is computed in a spawned task so it doesn't stall event processing
    if matches!(event.kind, FsEventKind::Created | FsEventKind::Modified) {
        let path_clone = path.clone();
        let sample = watch_cfg.entropy_sample_bytes;
        let max = watch_cfg.entropy_max_file_bytes;
        let high = cfg.entropy_high_bits;
        let med  = cfg.entropy_medium_bits;
        let tx   = alert_tx.clone();
        let seq2 = id_seq.clone();
        let cfg2 = cfg.clone();

        tokio::spawn(async move {
            if let Ok(Some(entropy)) = crate::detection::entropy::file_entropy(&path_clone, sample, max).await {
                let (pts, label) = if entropy >= high {
                    (20u32, "high")
                } else if entropy >= med {
                    (10u32, "medium")
                } else {
                    return; // not suspicious
                };

                analyze!(
                    "entropy {} ({:.2} bits/byte): {} (pid={pid})",
                    label, entropy, path_clone.display()
                );

                let id = seq2.fetch_add(1, Ordering::Relaxed);
                let signal = ThreatSignal::HighEntropy {
                    path: path_clone,
                    entropy,
                    pid: Some(pid),
                };

                // We can't mutate `windows` from a spawned task, so emit as an
                // AlertMsg with a synthetic ScoredEvent at score=pts.
                // The orchestrator accumulates these.
                let scored = ScoredEvent {
                    id,
                    ts: chrono::Utc::now(),
                    signal,
                    delta: pts,
                    cumulative: Score(pts),
                    level: ThreatLevel::from_score_cfg(Score(pts), &cfg2),
                };
                let _ = tx.send(AlertMsg { event: scored, incident: None }).await;
            }
        });
    }
}

async fn score_and_emit(
    pid:         u32,
    signal:      ThreatSignal,
    pts:         u32,
    cfg:         &ScoringConfig,
    windows:     &mut HashMap<u32, ScoreWindow>,
    alert_tx:    &mpsc::Sender<AlertMsg>,
    id_seq:      &Arc<AtomicU64>,
    raw_ev:      &RawFsEvent,
    dry_run:     bool,
) {
    let win = windows.entry(pid).or_insert_with(|| ScoreWindow::new(pid));
    win.score += Score(pts);
    win.last_signal = Instant::now();

    let level = ThreatLevel::from_score_cfg(win.score, cfg);
    if level > win.peak_level {
        win.peak_level = level;
    }

    let id = id_seq.fetch_add(1, Ordering::Relaxed);
    let scored = ScoredEvent {
        id,
        ts:         raw_ev.ts,
        signal:     signal.clone(),
        delta:      pts,
        cumulative: win.score,
        level,
    };
    win.events.push(scored.clone());

    analyze!(
        "score={} level={} pid={pid} +{pts}pts",
        win.score, level
    );

    // Build incident if critical
    let incident = if level == ThreatLevel::Critical {
        Some(build_incident(win, raw_ev))
    } else {
        None
    };

    if level > ThreatLevel::Nominal || incident.is_some() {
        let _ = alert_tx.send(AlertMsg { event: scored, incident }).await;
    }
}

fn build_incident(win: &ScoreWindow, raw_ev: &RawFsEvent) -> Incident {
    let affected: Vec<PathBuf> = win.events.iter()
        .filter_map(|e| e.signal.path().cloned())
        .collect();

    Incident {
        id:             Uuid::new_v4(),
        started_at:     chrono::Utc::now() - chrono::Duration::seconds(win.started_at.elapsed().as_secs() as i64),
        closed_at:      chrono::Utc::now(),
        peak_level:     win.peak_level,
        final_score:    win.score,
        events:         win.events.clone(),
        affected_paths: affected,
        primary_pid:    if win.pid == 0 { None } else { Some(win.pid) },
        process_info:   None, // enriched by orchestrator
        actions_taken:  Vec::new(),
        snapshot_used:  None,
        report_path:    None,
    }
}

fn gc_windows(cfg: &ScoringConfig, windows: &mut HashMap<u32, ScoreWindow>) {
    let expired_pids: Vec<u32> = windows
        .iter()
        .filter(|(_, w)| w.is_expired(cfg.window_secs))
        .map(|(&pid, _)| pid)
        .collect();

    for pid in expired_pids {
        if let Some(win) = windows.get_mut(&pid) {
            if win.score.0 >= cfg.elevated_threshold {
                // Keep watching but decay
                win.reset_with_decay();
            } else {
                windows.remove(&pid);
            }
        }
    }
}
