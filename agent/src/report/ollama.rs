use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::{AgentError, Result};
use crate::events::types::Incident;

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model:  String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

/// Call a local Ollama instance to generate a plain-language incident narrative.
/// Returns the AI-generated text, or an error if Ollama is unreachable.
pub async fn generate_narrative(
    incident:    &Incident,
    ollama_url:  &str,
    model:       &str,
) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| AgentError::Ollama(e.to_string()))?;

    let prompt = build_prompt(incident);
    debug!("Sending incident to Ollama ({model})");

    let req_body = GenerateRequest {
        model:  model.to_string(),
        prompt,
        stream: false,
    };

    let url = format!("{}/api/generate", ollama_url.trim_end_matches('/'));
    let response = client
        .post(&url)
        .json(&req_body)
        .send()
        .await
        .map_err(|e| AgentError::Ollama(format!("Request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AgentError::Ollama(format!("HTTP {status}: {body}")));
    }

    let gen: GenerateResponse = response
        .json()
        .await
        .map_err(|e| AgentError::Ollama(format!("Parse error: {e}")))?;

    Ok(gen.response.trim().to_string())
}

fn build_prompt(incident: &Incident) -> String {
    let signals: Vec<String> = incident.events.iter().map(|e| {
        format!("  - {} (+{} pts)", signal_description(&e.signal), e.delta)
    }).collect();

    let actions: Vec<String> = incident.actions_taken.iter().map(|a| {
        format!("  - {}", action_description(a))
    }).collect();

    format!(
        r#"You are AegisOne, an AI security agent. A ransomware incident was detected and contained.
Write a clear, concise plain-English incident summary (3-5 sentences) for a non-technical user.
Explain what was detected, what happened to their files, and what was done to protect them.
Be reassuring but honest. Do not use jargon.

Incident data:
- Started: {}
- Threat level: {}
- Score: {} points
- Affected files: {}

Detection signals:
{}

Actions taken:
{}

Write the summary now (plain text, no markdown, 3-5 sentences):"#,
        incident.started_at.format("%Y-%m-%d %H:%M:%S UTC"),
        incident.peak_level,
        incident.final_score,
        incident.affected_paths.len(),
        signals.join("\n"),
        actions.join("\n"),
    )
}

fn signal_description(signal: &crate::events::types::ThreatSignal) -> String {
    use crate::events::types::ThreatSignal::*;
    match signal {
        HighEntropy { path, entropy, .. }        => format!("High-entropy write ({entropy:.1} bits/byte) to {}", path.display()),
        BurstWrites { writes_per_sec, .. }       => format!("Burst: {writes_per_sec:.0} files/second"),
        ExtensionChange { old_ext, new_ext, .. } => format!("File renamed {old_ext} → {new_ext}"),
        CanaryTouched { canary_path, .. }        => format!("Canary file touched: {}", canary_path.display()),
        ShadowDeleteAttempt { command_line, .. } => format!("Shadow delete attempt: {command_line}"),
        SuspiciousRename { path, .. }            => format!("Suspicious rename: {}", path.display()),
        RansomNoteCreated { path, .. }           => format!("Ransom note created: {}", path.display()),
    }
}

fn action_description(action: &crate::events::types::ResponseAction) -> String {
    use crate::events::types::ResponseAction::*;
    match action {
        ProcessTreeKilled { root_pid, killed_pids } =>
            format!("Killed process tree (root pid={root_pid}, {} total)", killed_pids.len()),
        NetworkBlocked { pid, rule_name } =>
            format!("Blocked outbound network for pid={pid} ({rule_name})"),
        RollbackStarted { snapshot_id, target_paths } =>
            format!("Started rollback from {} ({} files)", snapshot_id, target_paths.len()),
        RollbackCompleted { files_restored, .. } =>
            format!("Rollback complete: {files_restored} files restored"),
        ReportGenerated { path } =>
            format!("Report saved to {}", path.display()),
        DryRun { would_have } =>
            format!("[DRY-RUN] Would have: {would_have}"),
    }
}
