use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::{CreateKind, ModifyKind, RenameMode, RemoveKind};
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::config::WatchConfig;
use crate::error::{AgentError, Result};
use crate::events::types::{FsEventKind, RawFsEvent};

pub struct FsWatcher {
    _watcher:  RecommendedWatcher, // must be kept alive
    cfg:       WatchConfig,
}

impl FsWatcher {
    /// Create and start the filesystem watcher.
    /// Events are forwarded to `raw_tx`.
    pub fn start(
        cfg:     WatchConfig,
        raw_tx:  mpsc::Sender<RawFsEvent>,
        id_seq:  Arc<AtomicU64>,
    ) -> Result<Self> {
        let skip_exts: Vec<String> = cfg.skip_extensions.iter()
            .map(|e| e.to_lowercase())
            .collect();

        // We bridge notify's sync callback to the tokio channel via try_send.
        let tx = raw_tx.clone();
        let seq = id_seq.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    for path in &event.paths {
                        // Skip filtered extensions
                        if should_skip(path, &skip_exts) {
                            continue;
                        }

                        let kind = translate_kind(&event.kind, &event.paths);
                        let id = seq.fetch_add(1, Ordering::Relaxed);
                        let ev = RawFsEvent {
                            id,
                            ts:  chrono::Utc::now(),
                            path: path.clone(),
                            kind,
                            pid: None, // PID enrichment requires platform-specific work
                        };

                        if let Err(e) = tx.try_send(ev) {
                            // Drop events if the channel is full rather than blocking
                            // the watcher callback thread
                            debug!("FS event dropped (channel full or closed): {e}");
                        }
                    }
                }
                Err(e) => warn!("Watcher error: {e}"),
            }
        })
        .map_err(|e| AgentError::Watcher(e.to_string()))?;

        // Register watch paths
        for path in &cfg.paths {
            if path.exists() {
                watcher
                    .watch(path, RecursiveMode::Recursive)
                    .map_err(|e| AgentError::Watcher(format!("{}: {e}", path.display())))?;
                debug!("Watching: {}", path.display());
            } else {
                warn!("Watch path does not exist, skipping: {}", path.display());
            }
        }

        Ok(Self { _watcher: watcher, cfg })
    }

    /// Add an extra path to watch at runtime.
    #[allow(dead_code)]
    pub fn add_path(&mut self, path: &Path) -> Result<()> {
        self._watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| AgentError::Watcher(format!("{}: {e}", path.display())))
    }
}

fn should_skip(path: &Path, skip_exts: &[String]) -> bool {
    if let Some(ext) = path.extension() {
        let lower = format!(".{}", ext.to_string_lossy().to_lowercase());
        skip_exts.contains(&lower)
    } else {
        false
    }
}

fn translate_kind(kind: &EventKind, paths: &[PathBuf]) -> FsEventKind {
    match kind {
        EventKind::Create(CreateKind::File)
        | EventKind::Create(CreateKind::Any)
        | EventKind::Create(_) => FsEventKind::Created,

        EventKind::Modify(ModifyKind::Name(RenameMode::To))
        | EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            // For rename events notify may give us both old and new path
            let from = paths.first().cloned().unwrap_or_default();
            FsEventKind::Renamed { from }
        }

        EventKind::Remove(RemoveKind::File)
        | EventKind::Remove(RemoveKind::Any)
        | EventKind::Remove(_) => FsEventKind::Deleted,

        // Everything else is a modification
        _ => FsEventKind::Modified,
    }
}
