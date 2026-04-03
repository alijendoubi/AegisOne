use tracing_subscriber::{EnvFilter, fmt};
use crate::config::LogConfig;

/// Initialize the tracing subscriber.
/// Must be called once, before any tracing macros are used.
pub fn init(cfg: &LogConfig, verbose: u8, json_logs: bool) {
    let level = match verbose {
        0 => cfg.level.as_str(),
        1 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("aegisone_agent={level},warn")));

    if json_logs || cfg.json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_current_span(false)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(false)
            .compact()
            .init();
    }
}

/// Emit an agent-phase log line matching the product terminal format:
/// `[HH:MM:SS] phase    ▶ message`
#[macro_export]
macro_rules! phase {
    ($phase:expr, $($arg:tt)*) => {{
        let ts = chrono::Utc::now().format("%H:%M:%S");
        let msg = format!($($arg)*);
        tracing::info!("[{}] {:<8} ▶ {}", ts, $phase, msg);
    }};
}

/// Shorthand phase macros matching the product's six phases.
#[macro_export] macro_rules! observe  { ($($t:tt)*) => { $crate::phase!("observe",  $($t)*) }; }
#[macro_export] macro_rules! analyze  { ($($t:tt)*) => { $crate::phase!("analyze",  $($t)*) }; }
#[macro_export] macro_rules! act      { ($($t:tt)*) => { $crate::phase!("act",      $($t)*) }; }
#[macro_export] macro_rules! isolate  { ($($t:tt)*) => { $crate::phase!("isolate",  $($t)*) }; }
#[macro_export] macro_rules! recover  { ($($t:tt)*) => { $crate::phase!("recover",  $($t)*) }; }
#[macro_export] macro_rules! explain  { ($($t:tt)*) => { $crate::phase!("explain",  $($t)*) }; }

/// Formats the terminal banner printed at startup.
pub fn print_banner(version: &str) {
    println!();
    println!("  \x1b[36maegisone-agent\x1b[0m v{version}  •  on-device AI active");
    println!("  watching: \x1b[36mfilesystem, processes, network\x1b[0m");
    println!("  policy:   \x1b[36mdefensive-only\x1b[0m  |  privacy: \x1b[36mon-device\x1b[0m");
    println!();
}
