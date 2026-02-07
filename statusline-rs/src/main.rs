mod cache;
mod config;
mod containers;
mod context;
mod git;
mod mcp;
mod session;
mod stdin_data;
mod teams;
mod theme;
mod usage_limits;
mod worktrees;

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use config::Config;
use theme::Theme;

/// All collected data from parallel module execution.
struct StatusData {
    git: Option<git::GitInfo>,
    session: session::SessionInfo,
    mcp: mcp::McpInfo,
    context: context::ContextInfo,
    usage: usage_limits::UsageLimitsInfo,
    containers: containers::ContainerInfo,
    worktrees: worktrees::WorktreeInfo,
    teams: teams::TeamsInfo,
    /// Lines added from stdin cost data
    lines_added: Option<u64>,
    /// Lines removed from stdin cost data
    lines_removed: Option<u64>,
}

fn main() {
    // Read stdin data from Claude Code first
    let stdin = stdin_data::read_stdin();

    let cfg = Config::load();
    let theme = Theme::from_name(&cfg.theme.name);

    // Use workspace dir from stdin, fallback to cwd
    let cwd = stdin
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
        .map(PathBuf::from)
        .or_else(|| stdin.cwd.as_deref().map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let data = collect_all(&cwd, &stdin);
    let output = format_statusline(&cfg, &theme, &data, &cwd);

    print!("{output}");
}

/// Collect data from all modules concurrently using std::thread.
fn collect_all(cwd: &PathBuf, stdin: &stdin_data::StdinData) -> StatusData {
    let cwd_git = cwd.clone();
    let cwd_wt = cwd.clone();

    let (tx, rx) = mpsc::channel::<(&str, Box<dyn std::any::Any + Send>)>();

    // Session and context come directly from stdin (no thread needed)
    let session_data = session::from_stdin(stdin);
    let context_data = context::from_stdin(stdin);

    // Cost data from stdin
    let lines_added = stdin.cost.as_ref().and_then(|c| c.total_lines_added);
    let lines_removed = stdin.cost.as_ref().and_then(|c| c.total_lines_removed);

    // Spawn threads for I/O-bound modules
    let tx_git = tx.clone();
    thread::spawn(move || {
        let result = git::collect(&cwd_git);
        let _ = tx_git.send(("git", Box::new(result)));
    });

    let tx_mcp = tx.clone();
    thread::spawn(move || {
        let result = mcp::collect();
        let _ = tx_mcp.send(("mcp", Box::new(result)));
    });

    let tx_usage = tx.clone();
    thread::spawn(move || {
        let result = usage_limits::collect();
        let _ = tx_usage.send(("usage", Box::new(result)));
    });

    let tx_containers = tx.clone();
    thread::spawn(move || {
        let result = containers::collect();
        let _ = tx_containers.send(("containers", Box::new(result)));
    });

    let tx_wt = tx.clone();
    thread::spawn(move || {
        let result = worktrees::collect(&cwd_wt);
        let _ = tx_wt.send(("worktrees", Box::new(result)));
    });

    let tx_teams = tx.clone();
    thread::spawn(move || {
        let result = teams::collect();
        let _ = tx_teams.send(("teams", Box::new(result)));
    });

    drop(tx);

    let mut data = StatusData {
        git: None,
        session: session_data,
        mcp: mcp::McpInfo::default(),
        context: context_data,
        usage: usage_limits::UsageLimitsInfo::default(),
        containers: containers::ContainerInfo::default(),
        worktrees: worktrees::WorktreeInfo::default(),
        teams: teams::TeamsInfo::default(),
        lines_added,
        lines_removed,
    };

    for (name, value) in rx.iter() {
        match name {
            "git" => {
                if let Ok(v) = value.downcast::<Option<git::GitInfo>>() {
                    data.git = *v;
                }
            }
            "mcp" => {
                if let Ok(v) = value.downcast::<mcp::McpInfo>() {
                    data.mcp = *v;
                }
            }
            "usage" => {
                if let Ok(v) = value.downcast::<usage_limits::UsageLimitsInfo>() {
                    data.usage = *v;
                }
            }
            "containers" => {
                if let Ok(v) = value.downcast::<containers::ContainerInfo>() {
                    data.containers = *v;
                }
            }
            "worktrees" => {
                if let Ok(v) = value.downcast::<worktrees::WorktreeInfo>() {
                    data.worktrees = *v;
                }
            }
            "teams" => {
                if let Ok(v) = value.downcast::<teams::TeamsInfo>() {
                    data.teams = *v;
                }
            }
            _ => {}
        }
    }

    data
}

// ============================================================================
// FORMATTING — matches the bash statusline layout
// ============================================================================

/// Format the multi-line statusline output.
fn format_statusline(cfg: &Config, theme: &Theme, data: &StatusData, cwd: &PathBuf) -> String {
    let mut lines: Vec<String> = Vec::new();

    // LINE 1: ~/path (branch) 📁
    lines.push(format_line1(theme, data, cwd));

    // LINE 2: 🧠 Model │ 📄 +N • -N │ CC:version │ Ctx: pct%
    lines.push(format_line2(cfg, theme, data));

    // LINE 3: MCP:connected/total: servers │ ⏱ 5H ... • 7DAY ...
    let line3 = format_line3(theme, data);
    if !line3.is_empty() {
        lines.push(line3);
    }

    // LINE 4: 🌳 branch [N worktrees] + containers + teams
    let line4 = format_line4(theme, data);
    if !line4.is_empty() {
        lines.push(line4);
    }

    lines.join("\n")
}

/// Line 1: ~/path (branch) 📁
fn format_line1(theme: &Theme, data: &StatusData, cwd: &PathBuf) -> String {
    let r = theme.reset;
    let folder = shorten_path(cwd);

    if let Some(ref git) = data.git {
        let status_icon = if git.is_clean {
            format!("{}\u{2713}{}", theme.green, r)
        } else {
            format!("{}\u{2717}{}", theme.yellow, r)
        };
        format!(
            "{}{}{}  {}({}){} {} \u{1f4c1}",
            theme.blue, folder, r, theme.green, git.branch, r, status_icon
        )
    } else {
        format!("{}{}{} \u{1f4c1}", theme.blue, folder, r)
    }
}

/// Line 2: 🧠 Model │ 📄 +N • -N │ CC:version │ Ctx: pct%
fn format_line2(cfg: &Config, theme: &Theme, data: &StatusData) -> String {
    let r = theme.reset;
    let sep = format!(" {}\u{2502}{} ", theme.dim, r); // │ separator
    let mut parts: Vec<String> = Vec::new();

    // Model with emoji
    let model_emoji = model_emoji(&data.session.model_short, cfg);
    parts.push(format!(
        "{}{} {}{}",
        theme.cyan, model_emoji, data.session.model_display, r
    ));

    // Lines added/removed (from stdin cost data or git)
    let added = data.lines_added.unwrap_or(0);
    let removed = data.lines_removed.unwrap_or(0);
    if added > 0 || removed > 0 {
        parts.push(format!(
            "\u{1f4c4} {}+{}{} \u{2022} {}-{}{}",
            theme.green, added, r, theme.red, removed, r
        ));
    } else if let Some(ref git) = data.git {
        if !git.is_clean {
            let total_added = git.added + git.modified;
            parts.push(format!(
                "\u{1f4c4} {}+{}{} \u{2022} {}-{}{}",
                theme.green, total_added, r, theme.red, git.deleted, r
            ));
        }
    }

    // CC version
    if !data.session.cc_version.is_empty() {
        parts.push(format!(
            "{}CC:{}{}",
            theme.magenta, data.session.cc_version, r
        ));
    }

    // Context window
    if cfg.features.show_context_window {
        if let Some(pct) = data.context.usage_percent {
            let color = if pct >= cfg.context_window.critical_threshold as f64 {
                theme.red
            } else if pct >= cfg.context_window.warn_threshold as f64 {
                theme.yellow
            } else {
                theme.green
            };
            parts.push(format!("{}Ctx: {:.0}%{}", color, pct, r));
        }
    }

    parts.join(&sep)
}

/// Line 3: MCP:connected/total: servers │ ⏱ 5H at HH:MM (remaining) pct% • 7DAY time (pct%)
fn format_line3(theme: &Theme, data: &StatusData) -> String {
    let r = theme.reset;
    let sep = format!(" {}\u{2502}{} ", theme.dim, r);
    let mut major_parts: Vec<String> = Vec::new();

    // MCP section
    if data.mcp.total > 0 {
        let mcp_color = if data.mcp.connected == data.mcp.total {
            theme.bright_green
        } else if data.mcp.connected > 0 {
            theme.yellow
        } else {
            theme.red
        };

        let server_strs: Vec<String> = data
            .mcp
            .servers
            .iter()
            .map(|s| {
                if s.connected {
                    format!("{}{}{}", theme.bright_green, s.name, r)
                } else {
                    // Strikethrough for disconnected
                    format!("{}\x1b[9m{}\x1b[29m{}", theme.red, s.name, r)
                }
            })
            .collect();

        major_parts.push(format!(
            "{}MCP:{}/{}{}: {}",
            mcp_color,
            data.mcp.connected,
            data.mcp.total,
            r,
            server_strs.join(", ")
        ));
    }

    // Usage limits section
    let mut usage_parts: Vec<String> = Vec::new();

    if let Some(ref reset) = data.usage.five_hour_reset {
        let remaining = data
            .usage
            .five_hour_remaining
            .as_deref()
            .unwrap_or("?");
        let pct_str = data
            .usage
            .five_hour_percent
            .map(|p| format!(" {:.0}%", p))
            .unwrap_or_default();
        usage_parts.push(format!(
            "\u{23f1} 5H at {} ({}){}",
            reset, remaining, pct_str
        ));
    }

    if let Some(ref reset) = data.usage.seven_day_reset {
        let pct_str = data
            .usage
            .seven_day_percent
            .map(|p| format!(" ({:.0}%)", p))
            .unwrap_or_default();
        usage_parts.push(format!("7DAY {}{}", reset, pct_str));
    }

    if !usage_parts.is_empty() {
        major_parts.push(usage_parts.join(" \u{2022} "));
    }

    if major_parts.is_empty() {
        return String::new();
    }

    major_parts.join(&sep)
}

/// Line 4: 🌳 branch [N worktrees] + containers + teams
fn format_line4(theme: &Theme, data: &StatusData) -> String {
    let r = theme.reset;
    let mut parts: Vec<String> = Vec::new();

    // Worktrees with branch
    if !data.worktrees.worktrees.is_empty() {
        let branch_name = data
            .git
            .as_ref()
            .map(|g| g.branch.as_str())
            .unwrap_or("?");
        parts.push(format!(
            "{}\u{1f333} {} [{} worktrees]{}",
            theme.green,
            branch_name,
            data.worktrees.worktrees.len(),
            r
        ));
    }

    // Docker containers
    if !data.containers.containers.is_empty() {
        let container_strs: Vec<String> = data
            .containers
            .containers
            .iter()
            .map(|c| {
                let color = if c.status == "running" {
                    theme.green
                } else {
                    theme.red
                };
                format!("{}{}: {}{}", color, c.name, c.status, r)
            })
            .collect();
        parts.push(format!("\u{1f433} {}", container_strs.join(", ")));
    }

    // Teams
    if !data.teams.teams.is_empty() {
        let total_agents: usize = data.teams.teams.iter().map(|t| t.member_count).sum();
        parts.push(format!(
            "{}\u{1f465} team: {} agents{}",
            theme.cyan, total_agents, r
        ));
    }

    parts.join("  ")
}

/// Shorten a path for display (replace home with ~).
fn shorten_path(path: &PathBuf) -> String {
    let display = path.display().to_string();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        if display.starts_with(&home_str) {
            return format!("~{}", &display[home_str.len()..]);
        }
    }
    display
}

/// Get the emoji for a model name.
fn model_emoji<'a>(model_short: &str, cfg: &'a Config) -> &'a str {
    match model_short {
        "opus" => &cfg.emojis.opus,
        "sonnet" => &cfg.emojis.sonnet,
        "haiku" => &cfg.emojis.haiku,
        _ => &cfg.emojis.default_model,
    }
}
