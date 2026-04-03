use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::{AgentError, Result};
use crate::events::types::SnapshotInfo;

/// Manifest written alongside each rollback for audit purposes.
#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackManifest {
    pub incident_id:    Uuid,
    pub shadow_id:      String,
    pub started_at:     chrono::DateTime<chrono::Utc>,
    pub completed_at:   Option<chrono::DateTime<chrono::Utc>>,
    pub files:          Vec<FileRecord>,
    pub files_restored: usize,
    pub files_failed:   usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub original:   PathBuf,
    pub shadow_src: PathBuf,
    pub status:     FileStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Restored,
    Failed { reason: String },
    Staged  { staging_path: PathBuf },
}

/// Restore `target_paths` from `snapshot`.
/// Returns the number of files successfully restored and the manifest path.
pub async fn restore_files(
    incident_id:  Uuid,
    snapshot:     &SnapshotInfo,
    target_paths: &[PathBuf],
    report_dir:   &Path,
) -> Result<(usize, PathBuf)> {
    let staging_dir = std::env::temp_dir().join(format!("aegisone-staging-{incident_id}"));
    fs::create_dir_all(&staging_dir).await.map_err(|e| AgentError::io(staging_dir.display().to_string(), e))?;

    let manifest_path = report_dir.join(format!("rollback_manifest_{incident_id}.json"));

    let mut manifest = RollbackManifest {
        incident_id,
        shadow_id:    snapshot.shadow_id.clone(),
        started_at:   chrono::Utc::now(),
        completed_at: None,
        files:        Vec::new(),
        files_restored: 0,
        files_failed:   0,
    };

    for target in target_paths {
        let shadow_src = shadow_path(&snapshot.device_path, target);
        let record = restore_one(&shadow_src, target, &staging_dir).await;
        match &record.status {
            FileStatus::Restored         => manifest.files_restored += 1,
            FileStatus::Failed { .. }    => manifest.files_failed   += 1,
            FileStatus::Staged { .. }    => manifest.files_restored += 1, // staged = success
        }
        manifest.files.push(record);
    }

    manifest.completed_at = Some(chrono::Utc::now());

    // Write manifest
    fs::create_dir_all(report_dir).await.ok();
    let json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, json)
        .await
        .map_err(|e| AgentError::io(manifest_path.display().to_string(), e))?;

    info!(
        "Rollback complete: {}/{} files restored. Manifest: {}",
        manifest.files_restored,
        target_paths.len(),
        manifest_path.display()
    );

    Ok((manifest.files_restored, manifest_path))
}

async fn restore_one(shadow_src: &Path, target: &Path, staging_dir: &Path) -> FileRecord {
    let shadow_src_clone = shadow_src.to_path_buf();
    let target_clone     = target.to_path_buf();

    // Try direct overwrite first
    match fs::copy(shadow_src, target).await {
        Ok(_) => {
            info!("Restored: {}", target.display());
            FileRecord {
                original:   target.to_path_buf(),
                shadow_src: shadow_src.to_path_buf(),
                status:     FileStatus::Restored,
            }
        }
        Err(e) if is_locked_error(&e) => {
            // File is locked; copy to staging area instead
            let file_name = target.file_name().unwrap_or_default();
            let staged = staging_dir.join(file_name);
            match fs::copy(shadow_src, &staged).await {
                Ok(_) => {
                    warn!(
                        "File locked, staged at {}: {}",
                        staged.display(), target.display()
                    );
                    FileRecord {
                        original:   target.to_path_buf(),
                        shadow_src: shadow_src.to_path_buf(),
                        status:     FileStatus::Staged { staging_path: staged },
                    }
                }
                Err(e2) => {
                    error!("Rollback failed for {}: {e2}", target.display());
                    FileRecord {
                        original:   target.to_path_buf(),
                        shadow_src: shadow_src.to_path_buf(),
                        status:     FileStatus::Failed { reason: e2.to_string() },
                    }
                }
            }
        }
        Err(e) => {
            error!("Rollback failed for {}: {e}", target.display());
            FileRecord {
                original:   target.to_path_buf(),
                shadow_src: shadow_src.to_path_buf(),
                status:     FileStatus::Failed { reason: e.to_string() },
            }
        }
    }
}

/// Translate a live path to its shadow-copy equivalent.
/// e.g. `C:\Users\Alice\doc.txt` with device `\\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy1`
/// → `\\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy1\Users\Alice\doc.txt`
fn shadow_path(device_path: &Path, target: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        // Strip drive letter (e.g. "C:") and leading separator
        let target_str = target.to_string_lossy();
        let stripped = if target_str.len() >= 2 && target_str.chars().nth(1) == Some(':') {
            &target_str[2..] // strip "C:"
        } else {
            &target_str[..]
        };
        let stripped = stripped.trim_start_matches('\\').trim_start_matches('/');
        device_path.join(stripped)
    }
    #[cfg(not(windows))]
    {
        // On Linux the shadow would be a mount point
        device_path.join(target.strip_prefix("/").unwrap_or(target))
    }
}

fn is_locked_error(e: &std::io::Error) -> bool {
    use std::io::ErrorKind;
    matches!(e.kind(), ErrorKind::PermissionDenied | ErrorKind::WouldBlock)
        || e.raw_os_error() == Some(32) // ERROR_SHARING_VIOLATION on Windows
}
