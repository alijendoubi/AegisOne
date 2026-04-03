use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error};
use rand::RngCore;

use crate::config::CanaryConfig;
use crate::error::{AgentError, Result};
use crate::events::types::{RawFsEvent, FsEventKind};

/// Manages canary files — synthetic tripwires placed in watched directories.
/// Any access, modification, or deletion of a canary file is a near-certain
/// indicator of ransomware activity.
pub struct CanaryManager {
    /// Maps canary path → 32-byte sentinel (random, stored only in memory)
    sentinels: HashMap<PathBuf, [u8; 32]>,
    cfg:       CanaryConfig,
}

impl CanaryManager {
    pub fn new(cfg: CanaryConfig) -> Self {
        Self { sentinels: HashMap::new(), cfg }
    }

    /// Create canary files at all configured paths.
    pub async fn install(&mut self) -> Result<()> {
        for path_str in &self.cfg.paths {
            let path = PathBuf::from(expand_env(path_str));
            self.install_one(&path).await?;
        }
        info!("Canary files installed ({} total)", self.sentinels.len());
        Ok(())
    }

    /// Verify all canary files are present and unmodified.
    pub async fn verify(&self) -> Vec<PathBuf> {
        let mut missing = Vec::new();
        for (path, sentinel) in &self.sentinels {
            match tokio::fs::read(path).await {
                Ok(bytes) if bytes.as_slice() == sentinel.as_slice() => {}
                Ok(_) => {
                    warn!("Canary tampered: {}", path.display());
                    missing.push(path.clone());
                }
                Err(_) => {
                    warn!("Canary missing: {}", path.display());
                    missing.push(path.clone());
                }
            }
        }
        missing
    }

    /// Remove all canary files.
    pub async fn remove(&self) -> Result<()> {
        for path in self.sentinels.keys() {
            if path.exists() {
                tokio::fs::remove_file(path).await.map_err(|e| AgentError::io(path.display().to_string(), e))?;
                info!("Canary removed: {}", path.display());
            }
        }
        Ok(())
    }

    /// Returns the set of canary paths.
    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.sentinels.keys()
    }

    /// Returns true if the given path is a canary file.
    pub fn is_canary(&self, path: &Path) -> bool {
        self.sentinels.contains_key(path)
    }

    /// Periodic check task: polls canary integrity and emits a RawFsEvent if
    /// a canary has been tampered with.
    #[allow(dead_code)]
    pub async fn run_checker(
        cfg:      CanaryConfig,
        sentinels: HashMap<PathBuf, [u8; 32]>,
        tx:       mpsc::Sender<RawFsEvent>,
        id_seq:   Arc<std::sync::atomic::AtomicU64>,
    ) {
        let interval = std::time::Duration::from_secs(cfg.check_interval_secs);
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            for (path, sentinel) in &sentinels {
                let touched = match tokio::fs::read(path).await {
                    Ok(bytes) => bytes.as_slice() != sentinel.as_slice(),
                    Err(_)   => true, // missing = tampered
                };

                if touched {
                    let id = id_seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let ev = RawFsEvent {
                        id,
                        ts:   chrono::Utc::now(),
                        path: path.clone(),
                        kind: FsEventKind::Modified,
                        pid:  None,
                    };
                    if tx.send(ev).await.is_err() {
                        error!("Canary checker: channel closed, exiting");
                        return;
                    }
                }
            }
        }
    }

    // ── Private ───────────────────────────────────────────────────────────────

    async fn install_one(&mut self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                // Directory doesn't exist yet; skip this canary
                warn!("Canary parent dir missing, skipping: {}", parent.display());
                return Ok(());
            }
        }

        // Generate a 32-byte random sentinel
        let mut sentinel = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut sentinel);

        // Write sentinel bytes as the file content
        tokio::fs::write(path, &sentinel)
            .await
            .map_err(|e| AgentError::io(path.display().to_string(), e))?;

        // Hide the file on Windows
        #[cfg(windows)]
        set_hidden(path);

        info!("Canary installed: {}", path.display());
        self.sentinels.insert(path.to_path_buf(), sentinel);
        Ok(())
    }
}

/// Expand %USERNAME%, %USERPROFILE% etc. in path strings.
fn expand_env(s: &str) -> String {
    let mut result = s.to_string();
    for (key, val) in std::env::vars() {
        result = result.replace(&format!("%{key}%"), &val);
        result = result.replace(&format!("${{{key}}}"), &val);
    }
    result
}

#[cfg(windows)]
fn set_hidden(path: &Path) {
    use std::os::windows::ffi::OsStrExt;
    // Use SetFileAttributesW to set FILE_ATTRIBUTE_HIDDEN
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        windows::Win32::Storage::FileSystem::SetFileAttributesW(
            windows::core::PCWSTR(wide.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN,
        );
    }
}
