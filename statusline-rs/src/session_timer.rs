/// Session timer: track cumulative daily active API time + current session duration.
/// "today" uses total_api_duration_ms (actual work only, not idle time).
/// Stores session log in ~/.cache/statusline-rs/session-log.json

use serde::{Deserialize, Serialize};

use crate::cache::Cache;

#[derive(Debug, Default)]
pub struct SessionTimerInfo {
    /// Cumulative minutes today (all sessions)
    pub daily_minutes: u64,
    /// Current session minutes (from stdin duration_ms or session tracking)
    pub session_minutes: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SessionLog {
    /// ISO date string (YYYY-MM-DD) for the day these entries belong to
    date: String,
    /// Accumulated minutes for today (excluding the current session)
    accumulated_minutes: u64,
    /// Session ID of the last tracked session
    last_session_id: String,
    /// Duration (minutes) already counted for the current session
    last_session_minutes: u64,
}

/// Collect session timer data.
/// - `session_id`: current session identifier
/// - `api_duration_ms`: active API time (for daily accumulation — real work only)
/// - `wall_duration_ms`: wall-clock session time (for current session display)
pub fn collect(
    session_id: Option<&str>,
    api_duration_ms: Option<u64>,
    wall_duration_ms: Option<u64>,
) -> SessionTimerInfo {
    let cache = Cache::new();
    let log_path = cache.dir().join("session-log.json");
    let today = today_date();

    let api_mins = api_duration_ms.unwrap_or(0) / 60_000;
    let wall_mins = wall_duration_ms.unwrap_or(0) / 60_000;
    let sid = session_id.unwrap_or("");

    // Load existing log
    let mut log = load_log(&log_path);

    // Reset if it's a new day
    if log.date != today {
        log = SessionLog {
            date: today.clone(),
            accumulated_minutes: 0,
            last_session_id: String::new(),
            last_session_minutes: 0,
        };
    }

    // Sanity-check: clamp accumulated_minutes to 16 hours max per day (960 min).
    // This guards against corruption from bad api_duration_ms values.
    const MAX_DAILY_MINUTES: u64 = 960;
    if log.accumulated_minutes > MAX_DAILY_MINUTES {
        log.accumulated_minutes = 0;
        log.last_session_minutes = 0;
    }

    // If this is a different session than the last one we tracked,
    // accumulate the previous session's API minutes
    if !sid.is_empty() && !log.last_session_id.is_empty() && log.last_session_id != sid {
        log.accumulated_minutes += log.last_session_minutes;
        log.last_session_minutes = 0;
    }

    // Update current session tracking (using API time for accumulation)
    if !sid.is_empty() {
        log.last_session_id = sid.to_string();
        log.last_session_minutes = api_mins;
    }

    // Save the log
    save_log(&log_path, &log);

    SessionTimerInfo {
        daily_minutes: log.accumulated_minutes + api_mins,
        session_minutes: wall_mins,
    }
}

fn today_date() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d").to_string()
}

fn load_log(path: &std::path::Path) -> SessionLog {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => SessionLog::default(),
    }
}

fn save_log(path: &std::path::Path, log: &SessionLog) {
    if let Ok(json) = serde_json::to_string(log) {
        let _ = std::fs::write(path, json);
    }
}
