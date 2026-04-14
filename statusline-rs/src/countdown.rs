/// Countdown timer: show time remaining until a configured deadline.

use chrono::{DateTime, Local, FixedOffset};
use crate::config::CountdownConfig;

#[derive(Debug)]
pub struct CountdownInfo {
    pub label: String,
    /// Time remaining as human-readable string (e.g., "1d 14h")
    pub remaining: String,
    /// Urgency level for coloring: "green" (>2d), "yellow" (1-2d), "red" (<1d)
    pub urgency: Urgency,
}

#[derive(Debug, PartialEq)]
pub enum Urgency {
    Green,   // > 2 days
    Yellow,  // 1-2 days
    Red,     // < 1 day
    Expired, // past deadline
}

/// Compute countdown info from config. Returns None if disabled or unparseable.
pub fn collect(config: &CountdownConfig) -> Option<CountdownInfo> {
    if !config.enabled || config.deadline.is_empty() {
        return None;
    }

    let deadline = parse_deadline(&config.deadline)?;
    let now = Local::now();
    let diff = deadline.signed_duration_since(now);

    if diff.num_seconds() <= 0 {
        return Some(CountdownInfo {
            label: config.label.clone(),
            remaining: "PAST DUE".to_string(),
            urgency: Urgency::Expired,
        });
    }

    let total_hours = diff.num_hours();
    let days = total_hours / 24;
    let hours = total_hours % 24;
    let mins = diff.num_minutes() % 60;

    let remaining = if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    };

    let urgency = if days >= 2 {
        Urgency::Green
    } else if days >= 1 {
        Urgency::Yellow
    } else {
        Urgency::Red
    };

    Some(CountdownInfo {
        label: config.label.clone(),
        remaining,
        urgency,
    })
}

/// Parse deadline string. Supports ISO 8601 with timezone.
fn parse_deadline(s: &str) -> Option<DateTime<Local>> {
    // Try parsing as DateTime<FixedOffset> (ISO 8601 with timezone)
    if let Ok(dt) = DateTime::<FixedOffset>::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Local));
    }

    // Try with chrono's more lenient parsing (e.g., "2026-02-09T23:59:00-08:00")
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%:z") {
        return Some(dt.with_timezone(&Local));
    }

    // Try naive datetime (assume local timezone)
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        let local = Local::now().timezone();
        return dt.and_local_timezone(local).single();
    }

    None
}
