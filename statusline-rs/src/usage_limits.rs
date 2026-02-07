use chrono::{Local, DateTime, Timelike};
use serde_json::Value;
use std::process::Command;

use crate::cache::Cache;

/// Usage limit information for 5-hour and 7-day windows.
#[derive(Debug, Default)]
pub struct UsageLimitsInfo {
    /// 5-hour block reset time display (e.g., "at 14:59")
    pub five_hour_reset: Option<String>,
    /// Time remaining in 5-hour block (e.g., "1 hr 8 min")
    pub five_hour_remaining: Option<String>,
    /// 5-hour usage percentage (0-100)
    pub five_hour_percent: Option<f64>,

    /// 7-day reset time display (e.g., "Wed 9:59 PM")
    pub seven_day_reset: Option<String>,
    /// 7-day usage percentage (0-100)
    pub seven_day_percent: Option<f64>,
}

/// Collect usage limit information.
/// Priority: OAuth API (cached) → local files → time-based estimates.
pub fn collect() -> UsageLimitsInfo {
    let cache = Cache::new();

    // Try cached OAuth API response first (5 min TTL)
    if let Some(cached) = cache.get("usage_limits_api", 300) {
        if let Ok(json) = serde_json::from_str::<Value>(&cached) {
            let info = parse_api_response(&json);
            if info.five_hour_percent.is_some() {
                return info;
            }
        }
    }

    // Try fresh OAuth API call
    if let Some(info) = fetch_from_oauth_api(&cache) {
        return info;
    }

    // Fallback: try local files
    if let Some(info) = try_local_files() {
        return info;
    }

    // Last resort: time-based estimates (no percentage data)
    estimate_from_time()
}

/// Fetch usage data from Anthropic OAuth API.
fn fetch_from_oauth_api(cache: &Cache) -> Option<UsageLimitsInfo> {
    let token = get_oauth_token()?;

    let output = Command::new("curl")
        .args([
            "-s",
            "--max-time", "5",
            "-H", &format!("Authorization: Bearer {}", token),
            "-H", "Content-Type: application/json",
            "-H", "anthropic-beta: oauth-2025-04-20",
            "-H", "Accept: application/json",
            "https://api.anthropic.com/api/oauth/usage",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&body).ok()?;

    // Verify response has expected data
    if json.get("five_hour").is_none() {
        return None;
    }

    // Cache the successful response
    cache.set("usage_limits_api", &body);

    let info = parse_api_response(&json);
    if info.five_hour_percent.is_some() {
        Some(info)
    } else {
        None
    }
}

/// Parse the OAuth API response JSON.
fn parse_api_response(json: &Value) -> UsageLimitsInfo {
    let mut info = UsageLimitsInfo::default();

    // 5-hour block
    if let Some(five_hour) = json.get("five_hour") {
        info.five_hour_percent = five_hour
            .get("utilization")
            .and_then(|v| v.as_f64())
            .map(|v| v.round());

        if let Some(resets_at) = five_hour.get("resets_at").and_then(|v| v.as_str()) {
            let (clock, remaining) = format_reset_times(resets_at);
            info.five_hour_reset = clock;
            info.five_hour_remaining = remaining;
        }
    }

    // 7-day block
    if let Some(seven_day) = json.get("seven_day") {
        info.seven_day_percent = seven_day
            .get("utilization")
            .and_then(|v| v.as_f64())
            .map(|v| v.round());

        if let Some(resets_at) = seven_day.get("resets_at").and_then(|v| v.as_str()) {
            let (display, _) = format_reset_times(resets_at);
            info.seven_day_reset = display;
        }
    }

    info
}

/// Get OAuth access token from macOS Keychain.
fn get_oauth_token() -> Option<String> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() {
        return None;
    }

    // Parse JSON to extract access token
    let json: Value = serde_json::from_str(&raw).ok()?;

    // Try nested path first, then flat paths
    json.get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            json.get("accessToken")
                .or_else(|| json.get("access_token"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

/// Format ISO timestamp into clock time and remaining time.
/// Returns (clock_display, remaining_display).
fn format_reset_times(iso_timestamp: &str) -> (Option<String>, Option<String>) {
    let reset_dt = DateTime::parse_from_rfc3339(iso_timestamp)
        .or_else(|_| {
            // Try without fractional seconds
            let clean = iso_timestamp.split('.').next().unwrap_or(iso_timestamp);
            let with_z = if clean.ends_with('Z') || clean.contains('+') {
                clean.to_string()
            } else {
                format!("{}Z", clean)
            };
            DateTime::parse_from_rfc3339(&with_z)
        })
        .ok();

    let reset_dt = match reset_dt {
        Some(dt) => dt.with_timezone(&Local),
        None => return (None, None),
    };

    let now = Local::now();
    let diff = reset_dt.signed_duration_since(now);
    let total_secs = diff.num_seconds();

    if total_secs <= 0 {
        return (Some("now".into()), Some("now".into()));
    }

    // Clock time display
    let clock = format!("{:02}:{:02}", reset_dt.hour(), reset_dt.minute());

    // Remaining time display
    let remaining = if total_secs < 3600 {
        let mins = total_secs / 60;
        format!("{} min", mins)
    } else if total_secs < 86400 {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        if mins > 0 {
            format!("{} hr {} min", hours, mins)
        } else {
            format!("{} hr", hours)
        }
    } else {
        reset_dt.format("%a %-I:%M %p").to_string()
    };

    // For 7-day, show day + time if > 24h
    let clock_display = if total_secs >= 86400 {
        reset_dt.format("%a %-I:%M %p").to_string()
    } else {
        clock
    };

    (Some(clock_display), Some(remaining))
}

/// Try reading from local files.
fn try_local_files() -> Option<UsageLimitsInfo> {
    let home = dirs::home_dir()?;

    // Try Claude's cached usage data
    let usage_cache = home.join(".claude/usage_limits.json");
    if usage_cache.exists() {
        if let Ok(content) = std::fs::read_to_string(&usage_cache) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                let mut info = UsageLimitsInfo::default();
                if let Some(five_hour) = json.get("five_hour") {
                    info.five_hour_percent =
                        five_hour.get("usage_percent").and_then(|v| v.as_f64());
                    info.five_hour_reset = five_hour
                        .get("reset_time")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    info.five_hour_remaining = five_hour
                        .get("remaining")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }
                if let Some(seven_day) = json.get("seven_day") {
                    info.seven_day_percent =
                        seven_day.get("usage_percent").and_then(|v| v.as_f64());
                    info.seven_day_reset = seven_day
                        .get("reset_time")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }
                if info.five_hour_percent.is_some() {
                    return Some(info);
                }
            }
        }
    }

    None
}

/// Last resort: estimate reset times from current time (no usage % available).
fn estimate_from_time() -> UsageLimitsInfo {
    let mut info = UsageLimitsInfo::default();
    let now = Local::now();
    let current_hour = now.hour() as i64;
    let block_end_hour = ((current_hour / 5) + 1) * 5;

    let hours_remaining = block_end_hour - current_hour;
    let mins_remaining = 60 - now.minute() as i64;

    if hours_remaining > 0 {
        let reset_hour = (block_end_hour % 24) as u32;
        info.five_hour_reset = Some(format!("{:02}:{:02}", reset_hour, 0));

        let h = if mins_remaining == 60 { hours_remaining } else { hours_remaining - 1 };
        let m = if mins_remaining == 60 { 0 } else { mins_remaining };
        info.five_hour_remaining = Some(format!("{}h{}m", h, m));
    }

    info
}
