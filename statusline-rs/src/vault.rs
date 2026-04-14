//! Vault metrics for SecondBrain-style knowledge vaults.
//!
//! Detects vault presence via `AGENTS.md` or `justfile` in the workspace,
//! then collects inbox count, health status, and journal state.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Collected vault metrics.
#[derive(Debug, Default)]
pub struct VaultInfo {
    /// Whether a vault was detected in the workspace
    pub detected: bool,
    /// Path to the vault root
    pub root: Option<PathBuf>,
    /// Number of items in 00-Inbox/
    pub inbox_count: usize,
    /// Number of pending observations
    pub observations: usize,
    /// Number of pending tensions
    pub tensions: usize,
    /// Number of open tasks (unchecked checkboxes in ops/tasks.md)
    pub open_tasks: usize,
    /// Whether today's journal entry exists
    pub journal_today: bool,
    /// Overall health status
    pub health: VaultHealth,
}

/// Health level based on inbox pressure and maintenance state.
#[derive(Debug, Default, PartialEq)]
pub enum VaultHealth {
    #[default]
    Unknown,
    /// Inbox < 20, no critical issues
    Healthy,
    /// Inbox 20-40 or some maintenance needed
    Warning,
    /// Inbox > 40 or critical issues
    Critical,
}

/// Vault configuration from Config.toml.
#[derive(Debug, serde::Deserialize)]
pub struct VaultConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Override vault path (auto-detected from cwd if not set)
    #[serde(default)]
    pub path: Option<String>,
    /// Inbox warning threshold (default 20)
    #[serde(default = "default_inbox_warn")]
    pub inbox_warn: usize,
    /// Inbox critical threshold (default 40)
    #[serde(default = "default_inbox_critical")]
    pub inbox_critical: usize,
}

fn default_inbox_warn() -> usize {
    20
}
fn default_inbox_critical() -> usize {
    40
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: None,
            inbox_warn: default_inbox_warn(),
            inbox_critical: default_inbox_critical(),
        }
    }
}

/// Detect and collect vault metrics.
///
/// Detection: looks for `AGENTS.md` in the workspace directory.
/// If `config.path` is set, uses that directly.
pub fn collect(cwd: &Path, config: &VaultConfig) -> VaultInfo {
    if !config.enabled {
        return VaultInfo::default();
    }

    // Determine vault root
    let root = if let Some(ref p) = config.path {
        let path = PathBuf::from(shellexpand(p));
        if path.join("AGENTS.md").exists() || path.join("00-Inbox").exists() {
            Some(path)
        } else {
            None
        }
    } else {
        detect_vault(cwd)
    };

    let Some(root) = root else {
        return VaultInfo::default();
    };

    let inbox_count = count_md_files(&root.join("00-Inbox"));
    let observations = count_md_files(&root.join("ops/observations"));
    let tensions = count_md_files(&root.join("ops/tensions"));
    let open_tasks = count_open_tasks(&root.join("ops/tasks.md"));
    let journal_today = check_today_journal(&root);

    let health = if inbox_count >= config.inbox_critical {
        VaultHealth::Critical
    } else if inbox_count >= config.inbox_warn || observations >= 10 || tensions >= 5 {
        VaultHealth::Warning
    } else {
        VaultHealth::Healthy
    };

    VaultInfo {
        detected: true,
        root: Some(root),
        inbox_count,
        observations,
        tensions,
        open_tasks,
        journal_today,
        health,
    }
}

/// Walk up from cwd looking for a vault root (has AGENTS.md or 00-Inbox/).
fn detect_vault(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    for _ in 0..5 {
        if dir.join("AGENTS.md").exists() || dir.join("00-Inbox").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Count .md files in a directory (non-recursive, depth 1).
fn count_md_files(dir: &Path) -> usize {
    fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "md")
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

/// Count unchecked checkboxes (`- [ ]`) in a markdown file.
fn count_open_tasks(path: &Path) -> usize {
    fs::read_to_string(path)
        .ok()
        .map(|content| content.lines().filter(|l| l.contains("- [ ]")).count())
        .unwrap_or(0)
}

/// Check if today's journal entry exists.
/// Uses `date` command for local timezone accuracy.
fn check_today_journal(root: &Path) -> bool {
    // Use system `date` command for correct local timezone
    let output = std::process::Command::new("date")
        .arg("+%Y/%m/%Y-%m-%d")
        .output()
        .ok();

    let date_str = output
        .as_ref()
        .and_then(|o| std::str::from_utf8(&o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if date_str.is_empty() {
        return false;
    }

    let path = root.join(format!("journal/{}.md", date_str));
    path.exists()
}

/// Expand ~ in paths.
fn shellexpand(s: &str) -> String {
    if s.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &s[1..]);
        }
    }
    s.to_string()
}
