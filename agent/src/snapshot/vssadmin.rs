use std::path::PathBuf;
use tokio::process::Command;
use tracing::{debug, info};

use crate::error::{AgentError, Result};
use crate::events::types::SnapshotInfo;

/// Create a new VSS shadow copy for the given volume (e.g. "C:").
/// Returns the new `SnapshotInfo` on success.
pub async fn create_shadow(volume: &str) -> Result<SnapshotInfo> {
    info!("Creating VSS shadow copy for volume {volume}");

    let output = Command::new("vssadmin")
        .args(["create", "shadow", &format!("/for={volume}")])
        .output()
        .await
        .map_err(|e| AgentError::vss(format!("vssadmin exec failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AgentError::vss(format!(
            "vssadmin create shadow failed.\nstdout: {stdout}\nstderr: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("vssadmin output: {stdout}");

    parse_create_output(&stdout, volume)
}

/// List all existing VSS shadow copies for the given volume.
pub async fn list_shadows(volume: &str) -> Result<Vec<SnapshotInfo>> {
    let output = Command::new("vssadmin")
        .args(["list", "shadows", &format!("/for={volume}")])
        .output()
        .await
        .map_err(|e| AgentError::vss(format!("vssadmin list exec failed: {e}")))?;

    if !output.status.success() {
        return Ok(Vec::new()); // No shadows is not an error
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_list_output(&stdout, volume))
}

/// Delete a specific shadow copy by ID (GUID string).
pub async fn delete_shadow(shadow_id: &str) -> Result<()> {
    let output = Command::new("vssadmin")
        .args([
            "delete", "shadows",
            &format!("/shadow={shadow_id}"),
            "/quiet",
        ])
        .output()
        .await
        .map_err(|e| AgentError::vss(format!("vssadmin delete exec failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AgentError::vss(format!("vssadmin delete failed: {stderr}")));
    }
    info!("Deleted shadow copy {shadow_id}");
    Ok(())
}

// ── Parsers ────────────────────────────────────────────────────────────────────
//
// vssadmin output is locale-dependent on Windows, so we use flexible regex-free
// parsing that looks for key substrings rather than exact formats.

fn parse_create_output(stdout: &str, volume: &str) -> Result<SnapshotInfo> {
    // Sample output:
    //   Successfully created shadow copy for 'C:\'
    //   Shadow Copy ID: {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}
    //   Shadow Copy Volume Name: \\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy1

    let shadow_id = extract_between(stdout, "Shadow Copy ID:", "\n")
        .or_else(|| extract_between(stdout, "Shadow Copy ID:", "\r"))
        .unwrap_or_default()
        .trim()
        .to_string();

    let device_path_str = extract_between(stdout, "Shadow Copy Volume Name:", "\n")
        .or_else(|| extract_between(stdout, "Shadow Copy Volume Name:", "\r"))
        .unwrap_or_default()
        .trim()
        .to_string();

    if shadow_id.is_empty() {
        return Err(AgentError::vss("Could not parse shadow copy ID from vssadmin output"));
    }

    Ok(SnapshotInfo {
        shadow_id,
        volume:      volume.to_string(),
        device_path: PathBuf::from(device_path_str),
        created_at:  chrono::Utc::now(),
        size_bytes:  None,
    })
}

fn parse_list_output(stdout: &str, volume: &str) -> Vec<SnapshotInfo> {
    // Each shadow copy block looks like:
    //   Contents of shadow copy set ID: ...
    //      Shadow Copy ID: {GUID}
    //      Shadow Copy Volume Name: \\?\GLOBALROOT\...
    //      Original Volume Name: C:\
    //      Creation Time: ...

    let mut results = Vec::new();
    let mut current_id = String::new();
    let mut current_device = String::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(id) = trimmed.strip_prefix("Shadow Copy ID:") {
            // Flush previous block if complete
            if !current_id.is_empty() && !current_device.is_empty() {
                results.push(SnapshotInfo {
                    shadow_id:   current_id.clone(),
                    volume:      volume.to_string(),
                    device_path: PathBuf::from(current_device.clone()),
                    created_at:  chrono::Utc::now(),
                    size_bytes:  None,
                });
            }
            current_id     = id.trim().to_string();
            current_device = String::new();
        } else if let Some(dev) = trimmed.strip_prefix("Shadow Copy Volume Name:") {
            current_device = dev.trim().to_string();
        }
    }

    // Flush last block
    if !current_id.is_empty() && !current_device.is_empty() {
        results.push(SnapshotInfo {
            shadow_id:   current_id,
            volume:      volume.to_string(),
            device_path: PathBuf::from(current_device),
            created_at:  chrono::Utc::now(),
            size_bytes:  None,
        });
    }

    results
}

fn extract_between<'a>(text: &'a str, after: &str, before: &str) -> Option<&'a str> {
    let start = text.find(after)? + after.len();
    let rest = &text[start..];
    let end = rest.find(before).unwrap_or(rest.len());
    Some(&rest[..end])
}
