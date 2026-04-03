use crate::error::{AgentError, Result};
use tracing::{info, warn, error};

/// Kill a process and all its descendants.
/// Returns the list of PIDs that were successfully terminated.
pub async fn kill_process_tree(root_pid: u32) -> Result<Vec<u32>> {
    tokio::task::spawn_blocking(move || kill_tree_sync(root_pid))
        .await
        .map_err(|e| AgentError::other(format!("kill task panicked: {e}")))?
}

#[cfg(windows)]
fn kill_tree_sync(root_pid: u32) -> Result<Vec<u32>> {
    use windows::Win32::Foundation::{CloseHandle, FALSE};
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
    use windows::Win32::System::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
        PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };

    // 1. Snapshot all running processes
    let snapshot = unsafe {
        CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| AgentError::kill(root_pid, format!("CreateToolhelp32Snapshot: {e}")))?
    };

    // 2. Build parent → children map
    let mut children: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let ok = unsafe { Process32FirstW(snapshot, &mut entry) };
    if ok.is_err() {
        unsafe { let _ = CloseHandle(snapshot); }
        return Err(AgentError::kill(root_pid, "Process32FirstW failed"));
    }

    loop {
        children
            .entry(entry.th32ParentProcessID)
            .or_default()
            .push(entry.th32ProcessID);

        if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
            break;
        }
    }
    unsafe { let _ = CloseHandle(snapshot); }

    // 3. Collect descendants via BFS
    let mut to_kill: Vec<u32> = vec![root_pid];
    let mut queue = std::collections::VecDeque::from([root_pid]);
    while let Some(pid) = queue.pop_front() {
        if let Some(kids) = children.get(&pid) {
            for &child in kids {
                to_kill.push(child);
                queue.push_back(child);
            }
        }
    }

    // 4. Kill children first, root last
    to_kill.reverse();
    let mut killed = Vec::new();

    for pid in to_kill {
        let result = unsafe {
            OpenProcess(PROCESS_TERMINATE, FALSE, pid)
        };
        match result {
            Ok(handle) => {
                let terminated = unsafe { TerminateProcess(handle, 1) };
                unsafe { let _ = CloseHandle(handle); }
                match terminated {
                    Ok(_) => {
                        info!("Killed pid={pid}");
                        killed.push(pid);
                    }
                    Err(e) => warn!("TerminateProcess pid={pid}: {e}"),
                }
            }
            Err(e) => warn!("OpenProcess pid={pid}: {e}"),
        }
    }

    Ok(killed)
}

#[cfg(not(windows))]
fn kill_tree_sync(root_pid: u32) -> Result<Vec<u32>> {
    // On non-Windows platforms use SIGKILL via kill(2)
    use std::process::Command;

    // First collect children via pgrep -P
    let output = Command::new("pgrep")
        .args(["-P", &root_pid.to_string()])
        .output();

    let mut pids = vec![root_pid];
    if let Ok(o) = output {
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            if let Ok(pid) = line.trim().parse::<u32>() {
                pids.push(pid);
            }
        }
    }

    let mut killed = Vec::new();
    for pid in pids {
        let result = unsafe { libc::kill(pid as i32, libc::SIGKILL) };
        if result == 0 {
            info!("Killed pid={pid}");
            killed.push(pid);
        } else {
            warn!("kill({pid}, SIGKILL) failed");
        }
    }
    Ok(killed)
}
