pub mod rollback;
pub mod vssadmin;

use std::path::Path;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::config::SnapshotConfig;
use crate::events::types::{ControlMsg, SnapshotCmd};
use crate::error::Result;

/// Snapshot manager: listens for `SnapshotCmd` requests and handles VSS lifecycle.
pub async fn run(
    cfg:             SnapshotConfig,
    mut snapshot_rx: mpsc::Receiver<SnapshotCmd>,
    _ctrl_tx:        mpsc::Sender<ControlMsg>,
    _report_dir:     std::path::PathBuf,
) {
    // Optionally create a baseline shadow copy at startup
    if cfg.create_on_start {
        match vssadmin::create_shadow(&cfg.volume).await {
            Ok(snap) => {
                info!("Baseline snapshot created: {} @ {}", snap.shadow_id, snap.created_at);
                let _ = enforce_retention(&cfg).await;
            }
            Err(e) => warn!("Baseline snapshot failed (agent may lack elevation): {e}"),
        }
    }

    while let Some(cmd) = snapshot_rx.recv().await {
        let result = vssadmin::create_shadow(&cmd.volume).await;
        let _ = cmd.reply.send(result);
    }
}

/// Create a shadow copy and immediately roll back the given paths.
#[allow(dead_code)]
pub async fn create_and_rollback(
    cfg:          &SnapshotConfig,
    incident_id:  uuid::Uuid,
    target_paths: &[std::path::PathBuf],
    report_dir:   &Path,
) -> Result<(usize, std::path::PathBuf)> {
    // Use most recent existing shadow if available to avoid creating duplicates
    let shadows = vssadmin::list_shadows(&cfg.volume).await?;
    let snap = if let Some(latest) = shadows.into_iter().last() {
        info!("Using existing shadow: {}", latest.shadow_id);
        latest
    } else {
        info!("No existing shadows; creating one now");
        vssadmin::create_shadow(&cfg.volume).await?
    };

    info!(
        "Rolling back {} files from shadow {} (volume {})",
        target_paths.len(),
        snap.shadow_id,
        snap.volume
    );

    rollback::restore_files(incident_id, &snap, target_paths, report_dir).await
}

async fn enforce_retention(cfg: &SnapshotConfig) -> Result<()> {
    let mut shadows = vssadmin::list_shadows(&cfg.volume).await?;
    // Sort oldest-first
    shadows.sort_by_key(|s| s.created_at);

    while shadows.len() > cfg.max_retained {
        let oldest = shadows.remove(0);
        if let Err(e) = vssadmin::delete_shadow(&oldest.shadow_id).await {
            warn!("Failed to delete old shadow {}: {e}", oldest.shadow_id);
        }
    }
    Ok(())
}
