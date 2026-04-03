pub mod ollama;
pub mod template;

use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::config::ReportConfig;
use crate::events::types::{
    ControlMsg, Incident, ReportRequest, ResponseAction, ThreatSignal,
};
use crate::error::{AgentError, Result};
use crate::explain;

/// Report generator: assembles incidents into Markdown reports with optional
/// Ollama narrative enrichment.
pub async fn run(
    cfg:             ReportConfig,
    mut report_rx:   mpsc::Receiver<ReportRequest>,
    ctrl_tx:         mpsc::Sender<ControlMsg>,
) {
    while let Some(req) = report_rx.recv().await {
        match generate(&cfg, req.incident, req.use_ollama).await {
            Ok(path) => {
                explain!("incident report written to {}", path.display());
            }
            Err(e) => {
                error!("Failed to generate report: {e}");
                let _ = ctrl_tx.send(ControlMsg::SubsystemError {
                    subsystem: "report",
                    error: e.to_string(),
                }).await;
            }
        }
    }
}

/// Generate a full incident report and write it to disk.
/// Returns the path to the written report file.
pub async fn generate(
    cfg:        &ReportConfig,
    incident:   Incident,
    use_ollama: bool,
) -> Result<PathBuf> {
    fs::create_dir_all(&cfg.output_dir)
        .await
        .map_err(|e| AgentError::io(cfg.output_dir.display().to_string(), e))?;

    // Optionally enrich with Ollama narrative
    let ai_narrative = if use_ollama && cfg.use_ollama {
        match ollama::generate_narrative(&incident, &cfg.ollama_url, &cfg.ollama_model).await {
            Ok(text) => format!("## AI Summary\n\n{text}\n"),
            Err(e)   => {
                warn!("Ollama unavailable ({e}), proceeding without AI narrative");
                String::new()
            }
        }
    } else {
        String::new()
    };

    let report_md = render_report(&incident, &ai_narrative)?;

    let filename = format!("incident_{}.md", incident.id);
    let report_path = cfg.output_dir.join(&filename);

    fs::write(&report_path, report_md.as_bytes())
        .await
        .map_err(|e| AgentError::io(report_path.display().to_string(), e))?;

    info!("Report written: {}", report_path.display());
    Ok(report_path)
}

fn render_report(incident: &Incident, ai_narrative: &str) -> Result<String> {
    let tmpl = template::Template::incident_report();

    // ── Simple vars ────────────────────────────────────────────────────────
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("incident_id",    incident.id.to_string());
    vars.insert("generated_at",   chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
    vars.insert("agent_version",  env!("CARGO_PKG_VERSION").to_string());
    vars.insert("started_at",     incident.started_at.format("%Y-%m-%d %H:%M:%S UTC").to_string());
    vars.insert("closed_at",      incident.closed_at.format("%Y-%m-%d %H:%M:%S UTC").to_string());
    vars.insert("peak_level",     incident.peak_level.to_string());
    vars.insert("final_score",    incident.final_score.to_string());
    vars.insert("affected_count", incident.affected_paths.len().to_string());
    vars.insert("ai_narrative",   ai_narrative.to_string());

    let primary = incident.process_info
        .as_ref()
        .map(|p| format!("{} (pid={})", p.name, p.pid))
        .or_else(|| incident.primary_pid.map(|p| format!("pid={p}")))
        .unwrap_or_else(|| "Unknown".into());
    vars.insert("primary_process", primary);

    // ── Timeline ───────────────────────────────────────────────────────────
    let timeline_lines: Vec<String> = incident.events.iter().map(|e| {
        let ts = e.ts.format("%H:%M:%S");
        let sig = signal_short(&e.signal);
        format!("| {} | {} | +{} pts | {} |", ts, e.level, e.delta, sig)
    }).collect();

    let timeline = if timeline_lines.is_empty() {
        "| Time | Level | Delta | Signal |\n|------|-------|-------|--------|\n| — | — | — | — |".to_string()
    } else {
        format!(
            "| Time | Level | Delta | Signal |\n|------|-------|-------|--------|\n{}",
            timeline_lines.join("\n")
        )
    };
    vars.insert("timeline", timeline);

    // ── Affected files ─────────────────────────────────────────────────────
    let affected = if incident.affected_paths.is_empty() {
        "_No files recorded._".to_string()
    } else {
        incident.affected_paths.iter()
            .map(|p| format!("- `{}`", p.display()))
            .collect::<Vec<_>>()
            .join("\n")
    };
    vars.insert("affected_files", affected);

    // ── Actions taken ──────────────────────────────────────────────────────
    let actions = if incident.actions_taken.is_empty() {
        "_No automated actions taken (dry-run or threshold not reached)._".to_string()
    } else {
        incident.actions_taken.iter()
            .map(action_md)
            .collect::<Vec<_>>()
            .join("\n")
    };
    vars.insert("actions_taken", actions);

    // ── Raw JSON ───────────────────────────────────────────────────────────
    let incident_json = serde_json::to_string_pretty(incident)
        .unwrap_or_else(|_| "{}".into());
    vars.insert("incident_json", incident_json);

    Ok(tmpl.render(&vars, &HashMap::new()))
}

fn signal_short(signal: &ThreatSignal) -> String {
    match signal {
        ThreatSignal::HighEntropy { entropy, .. }        => format!("entropy {entropy:.1} bits/byte"),
        ThreatSignal::BurstWrites { writes_per_sec, .. } => format!("{writes_per_sec:.0} writes/sec"),
        ThreatSignal::ExtensionChange { new_ext, .. }    => format!("renamed to {new_ext}"),
        ThreatSignal::CanaryTouched { .. }               => "canary touched".into(),
        ThreatSignal::ShadowDeleteAttempt { .. }         => "shadow delete attempt (T1490)".into(),
        ThreatSignal::SuspiciousRename { .. }            => "suspicious rename".into(),
        ThreatSignal::RansomNoteCreated { .. }           => "ransom note created".into(),
    }
}

fn action_md(action: &ResponseAction) -> String {
    match action {
        ResponseAction::ProcessTreeKilled { root_pid, killed_pids } =>
            format!("- **Killed** process tree rooted at pid={root_pid} ({} processes terminated)", killed_pids.len()),
        ResponseAction::NetworkBlocked { pid, rule_name } =>
            format!("- **Blocked** outbound network for pid={pid} (rule: `{rule_name}`)"),
        ResponseAction::RollbackStarted { snapshot_id, target_paths } =>
            format!("- **Rollback started** from snapshot `{snapshot_id}` ({} files)", target_paths.len()),
        ResponseAction::RollbackCompleted { files_restored, manifest_path } =>
            format!("- **Rollback completed**: {files_restored} files restored. Manifest: `{}`", manifest_path.display()),
        ResponseAction::ReportGenerated { path } =>
            format!("- **Report generated**: `{}`", path.display()),
        ResponseAction::DryRun { would_have } =>
            format!("- **[DRY-RUN]** Would have: {would_have}"),
    }
}
