use crate::stdin_data::StdinData;

/// Session information — model, version, session ID.
#[derive(Debug, Default)]
pub struct SessionInfo {
    /// Full model ID (e.g. "claude-opus-4-6")
    pub model: String,
    /// Short name (e.g. "opus")
    pub model_short: String,
    /// Display name from Claude Code (e.g. "Opus 4.6")
    pub model_display: String,
    /// Claude Code version (e.g. "2.1.34")
    pub cc_version: String,
    /// Session ID
    pub session_id: Option<String>,
}

/// Extract session info from stdin data, with env var fallback.
pub fn from_stdin(data: &StdinData) -> SessionInfo {
    let mut info = SessionInfo::default();

    // Model from stdin JSON
    if let Some(ref model) = data.model {
        info.model_display = model.display_name.clone().unwrap_or_default();
        info.model = model.id.clone().unwrap_or_default();
        info.model_short = shorten_model_name(&info.model);
        // If display_name is missing, use short name
        if info.model_display.is_empty() {
            info.model_display = capitalize_first(&info.model_short);
        }
        // Append version from model ID if display name lacks it
        if !info.model_display.is_empty() && !info.model_display.chars().any(|c| c.is_ascii_digit()) {
            if let Some(ver) = extract_model_version(&info.model) {
                info.model_display = format!("{} {}", info.model_display, ver);
            }
        }
    }

    // CC version from stdin JSON
    info.cc_version = data.version.clone().unwrap_or_default();
    info.session_id = data.session_id.clone();

    // Fallback to env vars if stdin didn't provide model
    if info.model.is_empty() {
        let raw_model = std::env::var("CLAUDE_MODEL")
            .or_else(|_| std::env::var("ANTHROPIC_MODEL"))
            .unwrap_or_default();

        if !raw_model.is_empty() {
            info.model_short = shorten_model_name(&raw_model);
            info.model_display = capitalize_first(&info.model_short);
            info.model = raw_model;
        } else {
            // Last resort: read settings.json
            if let Some(home) = dirs::home_dir() {
                let settings_path = home.join(".claude/settings.json");
                if let Ok(content) = std::fs::read_to_string(&settings_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(model) = json.get("model").and_then(|m| m.as_str()) {
                            info.model = model.to_string();
                            info.model_short = shorten_model_name(model);
                            info.model_display = capitalize_first(&info.model_short);
                        }
                    }
                }
            }
        }
    }

    if info.model.is_empty() {
        info.model = "unknown".into();
        info.model_short = "unknown".into();
        info.model_display = "Unknown".into();
    }

    info
}

/// Legacy collect function for thread-based fallback.
pub fn collect() -> SessionInfo {
    from_stdin(&StdinData::default())
}

fn shorten_model_name(model: &str) -> String {
    let lower = model.to_lowercase();
    if lower.contains("opus") {
        "opus".into()
    } else if lower.contains("sonnet") {
        "sonnet".into()
    } else if lower.contains("haiku") {
        "haiku".into()
    } else {
        model.rsplit('/').next().unwrap_or(model).to_string()
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Extract model version from model ID (e.g., "claude-opus-4-6" -> "4.6").
fn extract_model_version(model_id: &str) -> Option<String> {
    let lower = model_id.to_lowercase();

    // Find position after the model family name
    let start_pos = if let Some(pos) = lower.find("opus") {
        pos + 4 // "opus".len()
    } else if let Some(pos) = lower.find("sonnet") {
        pos + 6 // "sonnet".len()
    } else if let Some(pos) = lower.find("haiku") {
        pos + 5 // "haiku".len()
    } else {
        return None;
    };

    // Extract the part after the family name
    let remainder = &model_id[start_pos..];

    // Split by '-' and collect numeric segments
    let numeric_parts: Vec<&str> = remainder
        .split('-')
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
        .take(2) // Take first two numeric segments (major.minor)
        .collect();

    // Need at least 2 numeric segments for version
    if numeric_parts.len() >= 2 {
        Some(format!("{}.{}", numeric_parts[0], numeric_parts[1]))
    } else {
        None
    }
}
