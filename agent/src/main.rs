pub mod cli;
pub mod config;
pub mod detection;
pub mod error;
pub mod events;
pub mod report;
pub mod response;
pub mod snapshot;
pub mod telemetry;

use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::mpsc;
use tracing::{info, warn};

use cli::{CanaryAction, Command, ConfigAction, SnapshotAction};
use config::Config;
use events::types::ControlMsg;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    // ── Load config ────────────────────────────────────────────────────────
    let cfg_path = cli.config.clone().unwrap_or_else(Config::default_path);
    let mut cfg  = Config::load_or_default(&cfg_path);

    // CLI flags override config
    if cli.dry_run {
        cfg.response.dry_run = true;
    }

    // ── Init telemetry ─────────────────────────────────────────────────────
    telemetry::init(&cfg.log, cli.verbose, cli.json_logs);

    let use_ollama = !cli.no_ollama;

    // ── Dispatch subcommand ────────────────────────────────────────────────
    match cli.command.unwrap_or(Command::Watch { path: vec![] }) {
        Command::Watch { path: extra_paths } => {
            if !extra_paths.is_empty() {
                cfg.watch.paths.extend(extra_paths);
            }
            run_watch(cfg, use_ollama).await?;
        }

        Command::Status => {
            println!("Status: agent not currently running in daemon mode.");
            println!("Start with: aegisone-agent watch");
        }

        Command::Snapshot { action } => {
            run_snapshot_cmd(&cfg, action).await?;
        }

        Command::Report { incident_id, .. } => {
            println!("Report generation for past incidents not yet implemented.");
            println!("Incident ID: {incident_id}");
        }

        Command::Canary { action } => {
            run_canary_cmd(&cfg, action).await?;
        }

        Command::Config { action } => {
            run_config_cmd(&cfg, &cfg_path, action)?;
        }
    }

    Ok(())
}

// ── Watch mode ─────────────────────────────────────────────────────────────────

async fn run_watch(cfg: Config, use_ollama: bool) -> anyhow::Result<()> {
    telemetry::print_banner(env!("CARGO_PKG_VERSION"));

    // ── Check elevation (Windows) ──────────────────────────────────────────
    #[cfg(windows)]
    if !is_elevated() {
        eprintln!("[warn] Agent is not running as Administrator.");
        eprintln!("[warn] Kill-switch, network blocking, and VSS rollback require elevation.");
        eprintln!("[warn] Re-run as Administrator for full functionality.");
        eprintln!();
    }

    // ── Shared ID sequence ─────────────────────────────────────────────────
    let id_seq = Arc::new(AtomicU64::new(1));

    // ── Channels ───────────────────────────────────────────────────────────
    let (raw_tx,      raw_rx)      = mpsc::channel(512);
    let (alert_tx,    alert_rx)    = mpsc::channel(64);
    let (report_tx,   report_rx)   = mpsc::channel(64);
    let (snapshot_tx, snapshot_rx) = mpsc::channel(16);
    let (ctrl_tx,     mut ctrl_rx) = mpsc::channel(32);

    // ── Install canary files ───────────────────────────────────────────────
    let canary_cfg = cfg.canary.clone();
    let mut canary_mgr = detection::canary::CanaryManager::new(canary_cfg.clone());
    if let Err(e) = canary_mgr.install().await {
        warn!("Canary install failed: {e}");
    }
    let canary_paths: Vec<PathBuf> = canary_mgr.paths().cloned().collect();

    // ── Start filesystem watcher ───────────────────────────────────────────
    let _watcher = detection::watcher::FsWatcher::start(
        cfg.watch.clone(),
        raw_tx.clone(),
        id_seq.clone(),
    )?;
    info!("Watching {} paths", cfg.watch.paths.len());

    // ── Spawn subsystem tasks ──────────────────────────────────────────────
    let scoring_cfg    = cfg.scoring.clone();
    let watch_cfg      = cfg.watch.clone();
    let response_cfg   = cfg.response.clone();
    let snapshot_cfg   = cfg.snapshot.clone();
    let report_cfg     = cfg.report.clone();
    let report_dir     = cfg.report.output_dir.clone();
    let dry_run        = cfg.response.dry_run;

    let scorer_handle = {
        let alert_tx2 = alert_tx.clone();
        let id_seq2   = id_seq.clone();
        tokio::spawn(async move {
            detection::scorer::run(
                scoring_cfg,
                raw_rx,
                alert_tx2,
                id_seq2,
                canary_paths,
                dry_run,
                watch_cfg,
            ).await;
        })
    };

    let response_handle = {
        let ctrl_tx2   = ctrl_tx.clone();
        let report_tx2 = report_tx.clone();
        let snap_tx2   = snapshot_tx.clone();
        tokio::spawn(async move {
            response::run(
                response_cfg,
                alert_rx,
                report_tx2,
                snap_tx2,
                ctrl_tx2,
                use_ollama,
            ).await;
        })
    };

    let snapshot_handle = {
        let ctrl_tx2 = ctrl_tx.clone();
        tokio::spawn(async move {
            snapshot::run(snapshot_cfg, snapshot_rx, ctrl_tx2, report_dir).await;
        })
    };

    let report_handle = {
        let ctrl_tx2 = ctrl_tx.clone();
        tokio::spawn(async move {
            report::run(report_cfg, report_rx, ctrl_tx2).await;
        })
    };

    // Canary file events are detected by the main FS watcher (canary paths are
    // inside the watched directories). No separate periodic checker needed for MVP.

    info!("AegisOne agent running. Press Ctrl+C to stop.");

    // ── Supervisor loop ────────────────────────────────────────────────────
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C — shutting down");
                break;
            }
            Some(msg) = ctrl_rx.recv() => {
                match msg {
                    ControlMsg::Shutdown { reason } => {
                        info!("Shutdown requested: {reason}");
                        break;
                    }
                    ControlMsg::SubsystemError { subsystem, error: err } => {
                        warn!("Subsystem `{subsystem}` error: {err}");
                        // Non-fatal: log and continue
                    }
                    ControlMsg::HealthPing => {
                        info!("Health ping received");
                    }
                }
            }
        }
    }

    // Cleanup: remove firewall rules, etc. is handled by Drop impls where needed.
    info!("AegisOne agent stopped.");
    Ok(())
}

// ── Subcommand handlers ─────────────────────────────────────────────────────────

async fn run_snapshot_cmd(cfg: &Config, action: SnapshotAction) -> anyhow::Result<()> {
    match action {
        SnapshotAction::List => {
            let shadows = snapshot::vssadmin::list_shadows(&cfg.snapshot.volume).await?;
            if shadows.is_empty() {
                println!("No shadow copies found on volume {}.", cfg.snapshot.volume);
            } else {
                println!("Shadow copies on {}:", cfg.snapshot.volume);
                for s in shadows {
                    println!("  {} — created {} — {}", s.shadow_id, s.created_at, s.device_path.display());
                }
            }
        }
        SnapshotAction::Create => {
            let snap = snapshot::vssadmin::create_shadow(&cfg.snapshot.volume).await?;
            println!("Created: {}", snap.shadow_id);
        }
        SnapshotAction::Rollback { shadow_id, file } => {
            let shadows = snapshot::vssadmin::list_shadows(&cfg.snapshot.volume).await?;
            let snap = shadows.into_iter().find(|s| s.shadow_id == shadow_id)
                .ok_or_else(|| anyhow::anyhow!("Shadow ID not found: {shadow_id}"))?;

            if file.is_empty() {
                anyhow::bail!("Specify at least one --file to roll back");
            }

            let (restored, manifest) = snapshot::rollback::restore_files(
                uuid::Uuid::new_v4(),
                &snap,
                &file,
                &cfg.report.output_dir,
            ).await?;

            println!("Restored {restored} file(s). Manifest: {}", manifest.display());
        }
    }
    Ok(())
}

async fn run_canary_cmd(cfg: &Config, action: CanaryAction) -> anyhow::Result<()> {
    let mut mgr = detection::canary::CanaryManager::new(cfg.canary.clone());
    match action {
        CanaryAction::Install => {
            mgr.install().await?;
            println!("Canary files installed.");
        }
        CanaryAction::Verify => {
            // Need to install first to load sentinels, then verify
            mgr.install().await?;
            let missing = mgr.verify().await;
            if missing.is_empty() {
                println!("All canary files are intact.");
            } else {
                println!("ALERT: {} canary file(s) missing or tampered:", missing.len());
                for p in missing {
                    println!("  {}", p.display());
                }
            }
        }
        CanaryAction::Remove => {
            mgr.install().await.ok();
            mgr.remove().await?;
            println!("Canary files removed.");
        }
    }
    Ok(())
}

fn run_config_cmd(cfg: &Config, path: &PathBuf, action: ConfigAction) -> anyhow::Result<()> {
    match action {
        ConfigAction::Show => {
            let toml = toml::to_string_pretty(cfg)?;
            println!("{toml}");
        }
        ConfigAction::Validate => {
            println!("Config at `{}` is valid.", path.display());
        }
        ConfigAction::Init => {
            let parent = path.parent().unwrap_or(path);
            std::fs::create_dir_all(parent)?;
            let toml = toml::to_string_pretty(&Config::default())?;
            std::fs::write(path, toml)?;
            println!("Default config written to `{}`.", path.display());
        }
    }
    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────────────────

#[cfg(windows)]
fn is_elevated() -> bool {
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows::Win32::Foundation::CloseHandle;

    unsafe {
        let mut token = windows::Win32::Foundation::HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut ret_len: u32 = 0;
        let _ = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );
        let _ = CloseHandle(token);
        elevation.TokenIsElevated != 0
    }
}
