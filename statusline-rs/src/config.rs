use serde::Deserialize;
use std::path::PathBuf;

/// Top-level configuration parsed from Config.toml.
/// Uses flattened dot-notation keys (e.g. `theme.name = "catppuccin"`).
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub features: FeaturesConfig,
    #[serde(default)]
    pub emojis: EmojisConfig,
    #[serde(default)]
    pub timeouts: TimeoutsConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub labels: LabelsConfig,
    #[serde(default)]
    pub context_window: ContextWindowConfig,
    #[serde(default)]
    pub usage_limits: UsageLimitsConfig,
    #[serde(default)]
    pub session_info: SessionInfoConfig,
}

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_theme_name")]
    pub name: String,
    #[serde(default)]
    pub dynamic: Option<DynamicThemeConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DynamicThemeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub mode: Option<String>,
    pub day_theme: Option<String>,
    pub night_theme: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturesConfig {
    #[serde(default = "bool_true")]
    pub show_commits: bool,
    #[serde(default = "bool_true")]
    pub show_version: bool,
    #[serde(default = "bool_true")]
    pub show_mcp_status: bool,
    #[serde(default = "bool_true")]
    pub show_session_info: bool,
    #[serde(default = "bool_true")]
    pub show_context_window: bool,
    #[serde(default = "bool_true")]
    pub show_usage_limits: bool,
    #[serde(default = "bool_true")]
    pub show_code_productivity: bool,
    #[serde(default)]
    pub show_cost_tracking: bool,
    #[serde(default)]
    pub show_reset_info: bool,
    #[serde(default)]
    pub show_submodules: bool,
    #[serde(default)]
    pub show_prayer_times: bool,
}

#[derive(Debug, Deserialize)]
pub struct EmojisConfig {
    #[serde(default = "default_opus_emoji")]
    pub opus: String,
    #[serde(default = "default_haiku_emoji")]
    pub haiku: String,
    #[serde(default = "default_sonnet_emoji")]
    pub sonnet: String,
    #[serde(default = "default_model_emoji")]
    pub default_model: String,
    #[serde(default = "default_clean_emoji")]
    pub clean_status: String,
    #[serde(default = "default_dirty_emoji")]
    pub dirty_status: String,
}

#[derive(Debug, Deserialize)]
pub struct TimeoutsConfig {
    #[serde(default = "default_timeout")]
    pub mcp: String,
    #[serde(default = "default_timeout")]
    pub version: String,
    #[serde(default = "default_timeout")]
    pub ccusage: String,
}

#[derive(Debug, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_display_lines")]
    pub lines: u8,
    #[serde(default)]
    pub line1: Option<LineConfig>,
    #[serde(default)]
    pub line2: Option<LineConfig>,
    #[serde(default)]
    pub line3: Option<LineConfig>,
    #[serde(default)]
    pub line4: Option<LineConfig>,
    #[serde(default = "default_time_format")]
    pub time_format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LineConfig {
    #[serde(default)]
    pub components: Vec<String>,
    #[serde(default = "default_separator")]
    pub separator: String,
    #[serde(default)]
    pub show_when_empty: bool,
}

#[derive(Debug, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_cache_dir")]
    pub base_directory: String,
    #[serde(default = "bool_true")]
    pub enable_universal_caching: bool,
    #[serde(default)]
    pub durations: Option<CacheDurations>,
}

#[derive(Debug, Deserialize)]
pub struct CacheDurations {
    #[serde(default = "default_cache_git_status")]
    pub git_status: u64,
    #[serde(default = "default_cache_mcp")]
    pub mcp_server_list: u64,
    #[serde(default = "default_cache_git_branch")]
    pub git_current_branch: u64,
}

#[derive(Debug, Deserialize)]
pub struct LabelsConfig {
    #[serde(default = "default_mcp_label")]
    pub mcp: String,
}

#[derive(Debug, Deserialize)]
pub struct ContextWindowConfig {
    #[serde(default = "default_ctx_emoji")]
    pub emoji: String,
    #[serde(default = "bool_true")]
    pub show_tokens: bool,
    #[serde(default = "default_warn_threshold")]
    pub warn_threshold: u8,
    #[serde(default = "default_critical_threshold")]
    pub critical_threshold: u8,
}

#[derive(Debug, Deserialize)]
pub struct UsageLimitsConfig {
    #[serde(default = "default_limit_label")]
    pub label: String,
    #[serde(default = "default_warn_threshold")]
    pub warn_threshold: u8,
    #[serde(default = "default_usage_critical")]
    pub critical_threshold: u8,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

#[derive(Debug, Deserialize)]
pub struct SessionInfoConfig {
    #[serde(default = "bool_true")]
    pub show_id: bool,
    #[serde(default = "bool_true")]
    pub show_project: bool,
    #[serde(default = "default_id_length")]
    pub id_length: usize,
}

// Default value functions
fn bool_true() -> bool { true }
fn default_theme_name() -> String { "catppuccin".into() }
fn default_opus_emoji() -> String { "\u{1f9e0}".into() }  // brain
fn default_haiku_emoji() -> String { "\u{26a1}".into() }   // lightning
fn default_sonnet_emoji() -> String { "\u{1f3b5}".into() } // music note
fn default_model_emoji() -> String { "\u{1f916}".into() }  // robot
fn default_clean_emoji() -> String { "\u{2705}".into() }   // checkmark
fn default_dirty_emoji() -> String { "\u{1f4c1}".into() }  // folder
fn default_timeout() -> String { "10s".into() }
fn default_display_lines() -> u8 { 3 }
fn default_separator() -> String { " \u{2502} ".into() }   // box drawing vertical
fn default_time_format() -> String { "%I:%M %p".into() }
fn default_cache_dir() -> String { "auto".into() }
fn default_cache_git_status() -> u64 { 10 }
fn default_cache_mcp() -> u64 { 120 }
fn default_cache_git_branch() -> u64 { 10 }
fn default_mcp_label() -> String { "MCP".into() }
fn default_ctx_emoji() -> String { "\u{1f9e0}".into() }
fn default_warn_threshold() -> u8 { 50 }
fn default_critical_threshold() -> u8 { 90 }
fn default_usage_critical() -> u8 { 80 }
fn default_limit_label() -> String { "Limit:".into() }
fn default_cache_ttl() -> u64 { 300 }
fn default_id_length() -> usize { 8 }

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            features: FeaturesConfig::default(),
            emojis: EmojisConfig::default(),
            timeouts: TimeoutsConfig::default(),
            display: DisplayConfig::default(),
            cache: CacheConfig::default(),
            labels: LabelsConfig::default(),
            context_window: ContextWindowConfig::default(),
            usage_limits: UsageLimitsConfig::default(),
            session_info: SessionInfoConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: default_theme_name(),
            dynamic: None,
        }
    }
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            show_commits: true,
            show_version: true,
            show_mcp_status: true,
            show_session_info: true,
            show_context_window: true,
            show_usage_limits: true,
            show_code_productivity: true,
            show_cost_tracking: false,
            show_reset_info: false,
            show_submodules: false,
            show_prayer_times: false,
        }
    }
}

impl Default for EmojisConfig {
    fn default() -> Self {
        Self {
            opus: default_opus_emoji(),
            haiku: default_haiku_emoji(),
            sonnet: default_sonnet_emoji(),
            default_model: default_model_emoji(),
            clean_status: default_clean_emoji(),
            dirty_status: default_dirty_emoji(),
        }
    }
}

impl Default for TimeoutsConfig {
    fn default() -> Self {
        Self {
            mcp: default_timeout(),
            version: default_timeout(),
            ccusage: default_timeout(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            lines: default_display_lines(),
            line1: None,
            line2: None,
            line3: None,
            line4: None,
            time_format: default_time_format(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            base_directory: default_cache_dir(),
            enable_universal_caching: true,
            durations: None,
        }
    }
}

impl Default for LabelsConfig {
    fn default() -> Self {
        Self {
            mcp: default_mcp_label(),
        }
    }
}

impl Default for ContextWindowConfig {
    fn default() -> Self {
        Self {
            emoji: default_ctx_emoji(),
            show_tokens: true,
            warn_threshold: 50,
            critical_threshold: 90,
        }
    }
}

impl Default for UsageLimitsConfig {
    fn default() -> Self {
        Self {
            label: default_limit_label(),
            warn_threshold: 50,
            critical_threshold: 80,
            cache_ttl: 300,
        }
    }
}

impl Default for SessionInfoConfig {
    fn default() -> Self {
        Self {
            show_id: true,
            show_project: true,
            id_length: 8,
        }
    }
}

impl Config {
    /// Load configuration from the standard path or return defaults.
    pub fn load() -> Self {
        // Try ~/.claude/statusline/Config.toml first
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".claude/statusline/Config.toml");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = toml::from_str::<Config>(&content) {
                        return config;
                    }
                }
            }
        }

        // Try the project-local Config.toml
        let local_path = PathBuf::from("Config.toml");
        if local_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&local_path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return config;
                }
            }
        }

        Self::default()
    }
}
