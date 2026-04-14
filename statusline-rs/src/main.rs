mod cache;
mod config;
mod containers;
mod context;
mod countdown;
mod git;
mod mcp;
mod projects;
mod session;
mod session_timer;
mod skills;
mod stdin_data;
mod teams;
mod theme;
mod usage_limits;
mod vault;
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
    projects: projects::ProjectsInfo,
    session_timer: session_timer::SessionTimerInfo,
    skills: skills::SkillsInfo,
    countdown: Option<countdown::CountdownInfo>,
    /// Session cumulative lines added (from stdin cost data)
    lines_added: Option<u64>,
    /// Session cumulative lines removed (from stdin cost data)
    lines_removed: Option<u64>,
    /// Session cost in USD
    cost_usd: Option<f64>,
    /// Session duration in ms
    duration_ms: Option<u64>,
    /// Agent name (when running with --agent)
    agent_name: Option<String>,
    /// Vault metrics (SecondBrain)
    vault: vault::VaultInfo,
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

    let data = collect_all(&cwd, &stdin, &cfg);
    let output = format_statusline(&cfg, &theme, &data, &cwd);

    print!("{output}");
}

/// Collect data from all modules concurrently using std::thread.
fn collect_all(cwd: &PathBuf, stdin: &stdin_data::StdinData, cfg: &Config) -> StatusData {
    let cwd_git = cwd.clone();
    let cwd_wt = cwd.clone();

    let (tx, rx) = mpsc::channel::<(&str, Box<dyn std::any::Any + Send>)>();

    // Session and context come directly from stdin (no thread needed)
    let session_data = session::from_stdin(stdin);
    let context_data = context::from_stdin(stdin);

    // Cost data from stdin
    let lines_added = stdin.cost.as_ref().and_then(|c| c.total_lines_added);
    let lines_removed = stdin.cost.as_ref().and_then(|c| c.total_lines_removed);
    let cost_usd = stdin.cost.as_ref().and_then(|c| c.total_cost_usd);
    let duration_ms = stdin.cost.as_ref().and_then(|c| c.total_duration_ms);
    let agent_name = stdin.agent.as_ref().and_then(|a| a.name.clone());

    // Session timer: API duration for "today" (real work), wall-clock for "session"
    let timer_session_id = stdin.session_id.clone();
    let timer_api_ms = stdin.cost.as_ref().and_then(|c| c.total_api_duration_ms);
    let timer_wall_ms = duration_ms;

    // Countdown (config-driven, no I/O)
    let countdown_data = countdown::collect(&cfg.countdown);

    // Skills (filesystem scan, fast)
    let skills_data = skills::collect(&cwd);

    // Clone config values needed by threads
    let containers_config = config::ContainersConfig {
        enabled: cfg.containers.enabled,
        filter_by_user: cfg.containers.filter_by_user,
        show_exited: cfg.containers.show_exited,
        include_names: cfg.containers.include_names.clone(),
        exclude_names: cfg.containers.exclude_names.clone(),
    };

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
        let result = containers::collect(&containers_config);
        let _ = tx_containers.send(("containers", Box::new(result)));
    });

    let tx_wt = tx.clone();
    thread::spawn(move || {
        let result = worktrees::collect(&cwd_wt);
        let _ = tx_wt.send(("worktrees", Box::new(result)));
    });

    let tx_teams = tx.clone();
    let teams_session_id = stdin.session_id.clone();
    let teams_agent_name = stdin.agent.as_ref().and_then(|a| a.name.clone());
    thread::spawn(move || {
        let result = teams::collect(
            teams_session_id.as_deref(),
            teams_agent_name.as_deref(),
        );
        let _ = tx_teams.send(("teams", Box::new(result)));
    });

    let tx_projects = tx.clone();
    thread::spawn(move || {
        let result = projects::collect();
        let _ = tx_projects.send(("projects", Box::new(result)));
    });

    let tx_timer = tx.clone();
    thread::spawn(move || {
        let result = session_timer::collect(
            timer_session_id.as_deref(),
            timer_api_ms,
            timer_wall_ms,
        );
        let _ = tx_timer.send(("session_timer", Box::new(result)));
    });

    let tx_vault = tx.clone();
    let cwd_vault = cwd.clone();
    let vault_config = vault::VaultConfig {
        enabled: cfg.vault.enabled,
        path: cfg.vault.path.clone(),
        inbox_warn: cfg.vault.inbox_warn,
        inbox_critical: cfg.vault.inbox_critical,
    };
    thread::spawn(move || {
        let result = vault::collect(&cwd_vault, &vault_config);
        let _ = tx_vault.send(("vault", Box::new(result)));
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
        projects: projects::ProjectsInfo::default(),
        session_timer: session_timer::SessionTimerInfo::default(),
        skills: skills_data,
        countdown: countdown_data,
        lines_added,
        lines_removed,
        cost_usd,
        duration_ms,
        agent_name,
        vault: vault::VaultInfo::default(),
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
            "projects" => {
                if let Ok(v) = value.downcast::<projects::ProjectsInfo>() {
                    data.projects = *v;
                }
            }
            "session_timer" => {
                if let Ok(v) = value.downcast::<session_timer::SessionTimerInfo>() {
                    data.session_timer = *v;
                }
            }
            "vault" => {
                if let Ok(v) = value.downcast::<vault::VaultInfo>() {
                    data.vault = *v;
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

    // LINE 2: 🧠 Model │ ██░░░░ ctx% │ 📄 +N • -N │ $cost │ ⏱ time
    lines.push(format_line2(cfg, theme, data));

    // LINE 3: ⏱ 5H usage • 7DAY usage │ MCP │ SK
    let line3 = format_line3(theme, data);
    if !line3.is_empty() {
        lines.push(line3);
    }

    // LINE 4+: Worktrees, containers, teams, projects, countdown
    if cfg.display.lines >= 4 {
        let line4 = format_line4(cfg, theme, data);
        if !line4.is_empty() {
            lines.push(line4);
        }
    }

    lines.join("\n")
}

/// Line 1: ~/path (branch) 📁 [VIM:INSERT]
fn format_line1(theme: &Theme, data: &StatusData, cwd: &PathBuf) -> String {
    let r = theme.reset;
    let folder = shorten_path(cwd);

    let vim_suffix = if let Some(ref mode) = data.session.vim_mode {
        let color = if mode == "INSERT" { theme.green } else { theme.yellow };
        format!(" {}[VIM:{}]{}", color, mode, r)
    } else {
        String::new()
    };

    if let Some(ref git) = data.git {
        let status_icon = if git.is_clean {
            format!("{}\u{2713}{}", theme.green, r)
        } else {
            format!("{}\u{2717}{}", theme.yellow, r)
        };
        format!(
            "{}{}{}  {}({}){} {} \u{1f4c1}{}",
            theme.blue, folder, r, theme.green, git.branch, r, status_icon, vim_suffix
        )
    } else {
        format!("{}{}{} \u{1f4c1}{}", theme.blue, folder, r, vim_suffix)
    }
}

/// Line 2: 🧠 Model │ Ctx% │ +N/-N $cost │ ⏱ today/session (compact)
fn format_line2(cfg: &Config, theme: &Theme, data: &StatusData) -> String {
    let r = theme.reset;
    let sep = format!(" {}\u{2502}{} ", theme.dim, r);
    let mut parts: Vec<String> = Vec::new();

    // Agent name prefix (compact)
    if let Some(ref agent) = data.agent_name {
        parts.push(format!("{}@{}{}", theme.yellow, agent, r));
    }

    // Model with emoji (compact — no session name to save space)
    let model_emoji = model_emoji(&data.session.model_short, cfg);
    parts.push(format!(
        "{}{} {}{}",
        theme.cyan, model_emoji, data.session.model_display, r
    ));

    // Context window — always percentage, never bar (saves ~15 chars)
    if cfg.features.show_context_window {
        if let Some(pct) = data.context.usage_percent {
            let color = if pct >= cfg.context_window.critical_threshold as f64 {
                theme.red
            } else if pct >= cfg.context_window.warn_threshold as f64 {
                theme.yellow
            } else {
                theme.green
            };
            parts.push(format!("{}{:.0}%{}", color, pct, r));
        }
    }

    // Lines +/- and cost combined (compact)
    let added = data.lines_added.unwrap_or(0);
    let removed = data.lines_removed.unwrap_or(0);
    let cost_str = data.cost_usd
        .filter(|c| *c > 0.0)
        .map(|c| format!(" {}${:.2}{}", theme.yellow, c, r))
        .unwrap_or_default();
    if added > 0 || removed > 0 {
        parts.push(format!(
            "{}+{}{}/{}{}{}{}",
            theme.green, added, r, theme.red, removed, r, cost_str
        ));
    } else if !cost_str.is_empty() {
        parts.push(cost_str.trim().to_string());
    }

    // Session timer (compact: "⏱ 3m/0m" instead of "⏱ 3m today | 0m session")
    let daily = data.session_timer.daily_minutes;
    let session = data.session_timer.session_minutes;
    if daily > 0 || session > 0 {
        let daily_str = format_duration_hm(daily);
        let session_str = format_duration_hm(session);
        parts.push(format!("\u{23f1}{}/{}", daily_str, session_str));
    } else if let Some(ms) = data.duration_ms {
        let secs = ms / 1000;
        let mins = secs / 60;
        if mins > 0 {
            parts.push(format!("{}m", mins));
        }
    }

    parts.join(&sep)
}

/// Line 3: 5H:33% 1h9m • 7D:67% │ MCP:4/7 │ SK:33 (compact)
fn format_line3(theme: &Theme, data: &StatusData) -> String {
    let r = theme.reset;
    let sep = format!(" {}\u{2502}{} ", theme.dim, r);
    let mut major_parts: Vec<String> = Vec::new();

    // Usage limits (compact: "5H:33% 1h9m • 7D:67%")
    let mut usage_parts: Vec<String> = Vec::new();
    if let Some(ref _reset) = data.usage.five_hour_reset {
        let remaining = data
            .usage
            .five_hour_remaining
            .as_deref()
            .unwrap_or("?");
        let pct_str = data
            .usage
            .five_hour_percent
            .map(|p| {
                let color = if p >= 80.0 { theme.red } else if p >= 50.0 { theme.yellow } else { theme.green };
                format!("{}{:.0}%{}", color, p, r)
            })
            .unwrap_or_else(|| "?".to_string());
        usage_parts.push(format!("5H:{} {}", pct_str, remaining));
    }
    if let Some(ref _reset) = data.usage.seven_day_reset {
        let pct_str = data
            .usage
            .seven_day_percent
            .map(|p| {
                let color = if p >= 80.0 { theme.red } else if p >= 50.0 { theme.yellow } else { theme.green };
                format!("{}{:.0}%{}", color, p, r)
            })
            .unwrap_or_else(|| "?".to_string());
        usage_parts.push(format!("7D:{}", pct_str));
    }
    if !usage_parts.is_empty() {
        major_parts.push(usage_parts.join(" \u{2022} "));
    }

    // MCP — count only, no server names (saves ~40 chars)
    if data.mcp.total > 0 {
        let mcp_color = if data.mcp.connected == data.mcp.total {
            theme.bright_green
        } else if data.mcp.connected > 0 {
            theme.yellow
        } else {
            theme.red
        };
        major_parts.push(format!(
            "{}MCP:{}/{}{}",
            mcp_color, data.mcp.connected, data.mcp.total, r
        ));
    }

    // Skills
    let total_skills = data.skills.total();
    if total_skills > 0 {
        major_parts.push(format!(
            "{}SK:{}{}",
            theme.cyan, total_skills, r,
        ));
    }

    // Vault metrics (SecondBrain)
    if data.vault.detected {
        let health_color = match data.vault.health {
            vault::VaultHealth::Healthy => theme.green,
            vault::VaultHealth::Warning => theme.yellow,
            vault::VaultHealth::Critical => theme.red,
            vault::VaultHealth::Unknown => theme.dim,
        };
        let journal_icon = if data.vault.journal_today { "\u{2713}" } else { "\u{2717}" };
        let journal_color = if data.vault.journal_today { theme.green } else { theme.yellow };
        major_parts.push(format!(
            "{}IN:{}{}{}J:{}{}",
            health_color, data.vault.inbox_count, r,
            " ",
            journal_color, r,
        ));
        // Show journal check icon
        let vault_str = format!(
            "{}IN:{}{} {}J:{}{}",
            health_color, data.vault.inbox_count, r,
            journal_color, journal_icon, r,
        );
        // Replace the last push with the full vault string
        major_parts.pop();
        major_parts.push(vault_str);
    }

    if major_parts.is_empty() {
        return String::new();
    }

    major_parts.join(&sep)
}

/// Line 4+: Worktrees (each on own line) + containers + teams + projects + countdown
fn format_line4(cfg: &Config, theme: &Theme, data: &StatusData) -> String {
    let r = theme.reset;
    let mut lines: Vec<String> = Vec::new();

    // Worktrees — show OTHER worktrees with navigable paths (max 3, then +N more)
    if data.worktrees.total_count > 1 {
        // Build list of "other" worktrees (excluding current)
        let current_name = data
            .worktrees
            .current
            .as_ref()
            .map(|c| c.name.as_str())
            .unwrap_or("");
        let current_is_main = data
            .worktrees
            .current
            .as_ref()
            .map(|c| c.is_main)
            .unwrap_or(false);

        let mut others: Vec<&worktrees::Worktree> = Vec::new();

        // If current is a linked worktree, include main in the list
        if !current_is_main {
            if let Some(ref main_wt) = data.worktrees.main_worktree {
                others.push(main_wt);
            }
        }

        // Add linked worktrees that aren't the current one
        for wt in &data.worktrees.worktrees {
            if wt.name != current_name {
                others.push(wt);
            }
        }

        let max_show = 3;
        let total_others = others.len();

        for (i, wt) in others.iter().take(max_show).enumerate() {
            let branch_str = wt
                .branch
                .as_deref()
                .map(|b| format!("{}[{}]{}", theme.cyan, b, r))
                .unwrap_or_default();

            let suffix = if i == max_show - 1 && total_others > max_show {
                format!(" {}(+{} more){}", theme.dim, total_others - max_show, r)
            } else {
                String::new()
            };

            if i == 0 {
                // First line includes total count
                lines.push(format!(
                    "\u{1f333} {}{} worktrees{} {}\u{2502}{} {} {}{}{}{}",
                    theme.green, data.worktrees.total_count, r,
                    theme.dim, r,
                    branch_str,
                    theme.dim, wt.path, r,
                    suffix
                ));
            } else {
                lines.push(format!(
                    "\u{1f333} {} {}{}{}{}",
                    branch_str,
                    theme.dim, wt.path, r,
                    suffix
                ));
            }
        }
    }

    // Docker containers — show filtered with status and stats
    if !data.containers.containers.is_empty() {
        let container_strs: Vec<String> = data
            .containers
            .containers
            .iter()
            .map(|c| {
                let color = if c.status == "running" {
                    theme.green
                } else if c.status == "exited" {
                    theme.dim
                } else {
                    theme.red
                };
                let mut s = format!("{}{}{}", color, c.name, r);
                // Show status for non-running containers
                if c.status != "running" {
                    s.push_str(&format!(" {}{}{}", theme.dim, c.status, r));
                }
                // Show stats for running containers
                if let Some(cpu) = c.cpu_percent {
                    s.push_str(&format!(" {:.1}%cpu", cpu));
                }
                if let Some(ref mem) = c.mem_usage {
                    s.push_str(&format!(" {}", mem));
                }
                s
            })
            .collect();
        lines.push(format!("\u{1f433} {}", container_strs.join(" | ")));
    }

    // Teams with agent health
    if !data.teams.teams.is_empty() {
        let total_agents: usize = data.teams.teams.iter().map(|t| t.member_count).sum();
        let health = if data.teams.active_agents > 0 || data.teams.idle_agents > 0 {
            format!(
                " ({}{} active{}, {}{} idle{})",
                theme.green, data.teams.active_agents, r,
                theme.dim, data.teams.idle_agents, r
            )
        } else {
            String::new()
        };
        lines.push(format!(
            "{}\u{1f916} {} agents{}{}",
            theme.cyan, total_agents, health, r
        ));
    }

    // Multi-project awareness (gated by config)
    if cfg.features.show_projects && !data.projects.projects.is_empty() {
        let names: Vec<&str> = data.projects.projects.iter().map(|p| p.name.as_str()).collect();
        let display = if names.len() <= 3 {
            names.join(", ")
        } else {
            format!("{}, +{} more", names[..3].join(", "), names.len() - 3)
        };
        lines.push(format!(
            "{}\u{1f4c2} {} projects ({}){}",
            theme.magenta,
            data.projects.projects.len(),
            display,
            r
        ));
    }

    // Countdown timer
    if let Some(ref cd) = data.countdown {
        let color = match cd.urgency {
            countdown::Urgency::Green => theme.green,
            countdown::Urgency::Yellow => theme.yellow,
            countdown::Urgency::Red => theme.red,
            countdown::Urgency::Expired => theme.red,
        };
        let label = if cd.label.is_empty() { "Deadline" } else { &cd.label };
        lines.push(format!(
            "{}\u{23f3} {}: {}{}",
            color, label, cd.remaining, r
        ));
    }

    lines.join("\n")
}

/// Format minutes as "Xh Ym" or "Ym" for compact display.
fn format_duration_hm(minutes: u64) -> String {
    if minutes >= 60 {
        format!("{}h{}m", minutes / 60, minutes % 60)
    } else {
        format!("{}m", minutes)
    }
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

/// Show parent/project for context (e.g., "Projects/gharna" or "Documents/SecondBrain").
/// For multi-user setups, shows username when not the current user.
fn short_project_path(path: &PathBuf) -> String {
    let home = dirs::home_dir();
    let display = path.display().to_string();

    // Check if path is under a different user's home (multi-user setup)
    if let Some(ref home_dir) = home {
        let home_str = home_dir.display().to_string();
        if display.starts_with(&home_str) {
            // Under current user's home — show parent/folder
            let relative = &display[home_str.len()..];
            let parts: Vec<&str> = relative.trim_start_matches('/').split('/').collect();
            return match parts.len() {
                0 => "~".to_string(),
                1 => parts[0].to_string(),
                _ => format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1]),
            };
        }
        // Under different user — extract username
        // /Users/agent1/Projects/foo → agent1:Projects/foo
        if display.starts_with("/Users/") {
            let after_users = &display[7..]; // skip "/Users/"
            if let Some(slash_pos) = after_users.find('/') {
                let username = &after_users[..slash_pos];
                let rest = &after_users[slash_pos + 1..];
                let rest_parts: Vec<&str> = rest.split('/').collect();
                let short = if rest_parts.len() >= 2 {
                    format!("{}/{}", rest_parts[rest_parts.len() - 2], rest_parts[rest_parts.len() - 1])
                } else {
                    rest.to_string()
                };
                return format!("{}:{}", username, short);
            }
        }
    }

    // Fallback: just show last two path components
    let parts: Vec<&str> = display.trim_end_matches('/').rsplit('/').collect();
    if parts.len() >= 2 {
        format!("{}/{}", parts[1], parts[0])
    } else {
        display
    }
}

/// Truncate a string to max_len chars, appending "…" if truncated.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}\u{2026}", truncated)
    }
}

/// Render a visual bar for context window usage.
/// Example: "█████████░ 90%" (10-char bar)
fn render_context_bar(pct: f64, color: &str, dim: &str, reset: &str) -> String {
    let bar_width = 10;
    let filled = ((pct / 100.0) * bar_width as f64).round() as usize;
    let filled = filled.min(bar_width);
    let empty = bar_width - filled;
    let bar: String = "\u{2588}".repeat(filled);
    let empty_bar: String = "\u{2591}".repeat(empty);
    format!("{}{}{}{}{} {:.0}%{}", color, bar, dim, empty_bar, color, pct, reset)
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
