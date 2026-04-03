pub mod kill_switch;
pub mod network_block;

use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::config::ResponseConfig;
use crate::events::types::*;
use crate::{act, isolate};

/// Receives alert messages from the scorer and orchestrates the response:
/// - Elevated: log + record
/// - High: block outbound network, create snapshot
/// - Critical: kill process tree + block network + request rollback + request report
pub async fn run(
    cfg:          ResponseConfig,
    mut alert_rx: mpsc::Receiver<AlertMsg>,
    report_tx:    mpsc::Sender<ReportRequest>,
    snapshot_tx:  mpsc::Sender<SnapshotCmd>,
    ctrl_tx:      mpsc::Sender<ControlMsg>,
    use_ollama:   bool,
) {
    // Track which PIDs already have firewall rules so we don't double-block
    let mut blocked_pids: std::collections::HashSet<u32> = std::collections::HashSet::new();

    while let Some(msg) = alert_rx.recv().await {
        let level = msg.event.level;
        let pid   = msg.event.signal.pid();

        match level {
            ThreatLevel::Nominal => {}

            ThreatLevel::Elevated => {
                warn!(
                    level = "ELEVATED",
                    score = msg.event.cumulative.0,
                    pid   = ?pid,
                    "Elevated threat detected"
                );
            }

            ThreatLevel::High => {
                warn!(
                    level = "HIGH",
                    score = msg.event.cumulative.0,
                    pid   = ?pid,
                    "High threat — blocking network"
                );
                if let Some(p) = pid {
                    if !blocked_pids.contains(&p) && !cfg.dry_run && cfg.block_network {
                        match network_block::block_outbound(p).await {
                            Ok(rule) => {
                                isolate!("outbound blocked for pid={p} via rule `{rule}`");
                                blocked_pids.insert(p);
                            }
                            Err(e) => error!("Network block failed: {e}"),
                        }
                    }
                }
            }

            ThreatLevel::Critical => {
                if let Some(incident) = msg.incident {
                    respond_critical(
                        incident,
                        &cfg,
                        &report_tx,
                        &snapshot_tx,
                        &mut blocked_pids,
                        use_ollama,
                    )
                    .await;
                } else {
                    // Should not happen but handle gracefully
                    warn!("Critical alert received without incident data");
                }
            }
        }
    }
}

async fn respond_critical(
    mut incident: Incident,
    cfg:          &ResponseConfig,
    report_tx:    &mpsc::Sender<ReportRequest>,
    snapshot_tx:  &mpsc::Sender<SnapshotCmd>,
    blocked_pids: &mut std::collections::HashSet<u32>,
    use_ollama:   bool,
) {
    info!(
        incident_id = %incident.id,
        score        = incident.final_score.0,
        pid          = ?incident.primary_pid,
        "CRITICAL incident — initiating response"
    );

    // ── 1. Kill process tree ───────────────────────────────────────────────
    if let Some(pid) = incident.primary_pid {
        if cfg.dry_run {
            act!("DRY-RUN: would kill pid={pid} and descendants");
            incident.actions_taken.push(ResponseAction::DryRun {
                would_have: format!("kill process tree rooted at pid={pid}"),
            });
        } else if cfg.kill_on_critical {
            match kill_switch::kill_process_tree(pid).await {
                Ok(killed) => {
                    act!("kill pid={pid} + {} children — OK", killed.len().saturating_sub(1));
                    incident.actions_taken.push(ResponseAction::ProcessTreeKilled {
                        root_pid:    pid,
                        killed_pids: killed,
                    });
                }
                Err(e) => error!("Kill-switch failed: {e}"),
            }
        }

        // ── 2. Block network ───────────────────────────────────────────────
        if !blocked_pids.contains(&pid) && !cfg.dry_run && cfg.block_network {
            match network_block::block_outbound(pid).await {
                Ok(rule) => {
                    isolate!("outbound blocked for pid={pid} via rule `{rule}`");
                    incident.actions_taken.push(ResponseAction::NetworkBlocked {
                        pid,
                        rule_name: rule,
                    });
                    blocked_pids.insert(pid);
                }
                Err(e) => error!("Network block failed: {e}"),
            }
        }
    }

    // ── 3. Request snapshot rollback ───────────────────────────────────────
    if cfg.auto_rollback && !cfg.dry_run && !incident.affected_paths.is_empty() {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        let _ = snapshot_tx.send(SnapshotCmd {
            volume: "C:".into(),
            reply:  reply_tx,
        }).await;

        match reply_rx.await {
            Ok(Ok(snap)) => {
                incident.actions_taken.push(ResponseAction::RollbackStarted {
                    snapshot_id:  snap.shadow_id.clone(),
                    target_paths: incident.affected_paths.clone(),
                });
                incident.snapshot_used = Some(snap);
            }
            Ok(Err(e)) => error!("Snapshot failed: {e}"),
            Err(_)     => error!("Snapshot task channel dropped"),
        }
    }

    // ── 4. Generate incident report ────────────────────────────────────────
    let _ = report_tx.send(ReportRequest {
        incident,
        use_ollama,
    }).await;
}
