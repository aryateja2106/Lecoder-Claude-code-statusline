use serde::Deserialize;

/// Data provided by Claude Code on stdin as JSON.
#[derive(Debug, Default, Deserialize)]
pub struct StdinData {
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub model: Option<ModelInfo>,
    #[serde(default)]
    pub workspace: Option<WorkspaceInfo>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub cost: Option<CostInfo>,
    #[serde(default)]
    pub context_window: Option<ContextWindowInfo>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ModelInfo {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct WorkspaceInfo {
    #[serde(default)]
    pub current_dir: Option<String>,
    #[serde(default)]
    pub project_dir: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct CostInfo {
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub total_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_api_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_lines_added: Option<u64>,
    #[serde(default)]
    pub total_lines_removed: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ContextWindowInfo {
    #[serde(default)]
    pub total_input_tokens: Option<u64>,
    #[serde(default)]
    pub total_output_tokens: Option<u64>,
    #[serde(default)]
    pub context_window_size: Option<u64>,
    #[serde(default)]
    pub used_percentage: Option<f64>,
    #[serde(default)]
    pub remaining_percentage: Option<f64>,
}

/// Read and parse JSON from stdin. Claude Code pipes session data here.
/// If stdin is a terminal (running standalone), returns defaults.
pub fn read_stdin() -> StdinData {
    use std::io::{IsTerminal, Read};

    // Don't block on terminal input when running standalone
    if std::io::stdin().is_terminal() {
        return StdinData::default();
    }

    let mut input = String::new();
    match std::io::stdin().read_to_string(&mut input) {
        Ok(_) if !input.trim().is_empty() => {
            serde_json::from_str(&input).unwrap_or_default()
        }
        _ => StdinData::default(),
    }
}
