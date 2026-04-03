use crate::error::{AgentError, Result};
use tracing::{info, warn};

/// Block all outbound traffic for a process by PID.
/// On Windows this creates a Windows Firewall rule via `netsh`.
/// Returns the firewall rule name for later removal.
pub async fn block_outbound(pid: u32) -> Result<String> {
    let rule_name = format!("aegisone-block-{pid}");

    #[cfg(windows)]
    {
        // Get the executable path for the process so we can block by path
        // (netsh advfirewall rules can filter by program path)
        let exe_path = get_exe_path(pid);
        block_windows(pid, &rule_name, exe_path.as_deref()).await?;
    }

    #[cfg(not(windows))]
    {
        block_iptables(pid, &rule_name).await?;
    }

    info!("Outbound blocked for pid={pid} via rule `{rule_name}`");
    Ok(rule_name)
}

/// Remove a previously installed firewall rule.
pub async fn unblock(rule_name: &str) -> Result<()> {
    #[cfg(windows)]
    {
        let output = tokio::process::Command::new("netsh")
            .args([
                "advfirewall", "firewall", "delete", "rule",
                &format!("name={rule_name}"),
            ])
            .output()
            .await
            .map_err(|e| AgentError::NetworkBlock { pid: 0, reason: e.to_string() })?;

        if !output.status.success() {
            warn!("netsh delete rule `{rule_name}` returned non-zero");
        }
    }

    #[cfg(not(windows))]
    {
        // Delete the iptables rule (best-effort)
        let _ = tokio::process::Command::new("iptables")
            .args(["-D", "OUTPUT", "-m", "owner", "--pid-owner",
                   &rule_name.trim_start_matches("aegisone-block-"), "-j", "DROP"])
            .output()
            .await;
    }

    info!("Firewall rule `{rule_name}` removed");
    Ok(())
}

// ── Windows implementation ─────────────────────────────────────────────────────

#[cfg(windows)]
async fn block_windows(pid: u32, rule_name: &str, exe_path: Option<&str>) -> Result<()> {
    // Block by program path if we have it, otherwise block by PID (less precise)
    let mut args = vec![
        "advfirewall".to_string(),
        "firewall".to_string(),
        "add".to_string(),
        "rule".to_string(),
        format!("name={rule_name}"),
        "dir=out".to_string(),
        "action=block".to_string(),
    ];

    if let Some(path) = exe_path {
        args.push(format!("program={path}"));
    }

    let output = tokio::process::Command::new("netsh")
        .args(&args)
        .output()
        .await
        .map_err(|e| AgentError::NetworkBlock { pid, reason: e.to_string() })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AgentError::NetworkBlock {
            pid,
            reason: format!("netsh failed: {stderr}"),
        });
    }
    Ok(())
}

#[cfg(windows)]
fn get_exe_path(pid: u32) -> Option<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);
        if ok.is_ok() {
            Some(String::from_utf16_lossy(&buf[..size as usize]))
        } else {
            None
        }
    }
}

// ── Non-Windows fallback ───────────────────────────────────────────────────────

#[cfg(not(windows))]
async fn block_iptables(pid: u32, _rule_name: &str) -> Result<()> {
    let output = tokio::process::Command::new("iptables")
        .args([
            "-A", "OUTPUT",
            "-m", "owner",
            "--pid-owner", &pid.to_string(),
            "-j", "DROP",
        ])
        .output()
        .await
        .map_err(|e| AgentError::NetworkBlock { pid, reason: e.to_string() })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AgentError::NetworkBlock {
            pid,
            reason: format!("iptables failed: {stderr}"),
        });
    }
    Ok(())
}
