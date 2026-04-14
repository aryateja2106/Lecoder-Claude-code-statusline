use serde_json::Value;

/// Agent team information with health summary.
#[derive(Debug, Default)]
pub struct TeamsInfo {
    pub teams: Vec<Team>,
    /// Total agents with in_progress tasks
    pub active_agents: usize,
    /// Total agents with no in_progress tasks
    pub idle_agents: usize,
}

#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub member_count: usize,
}

/// Read ~/.claude/teams/*/config.json to find agent teams relevant to the current session.
///
/// Only includes teams where:
/// - The current session is the team lead (`leadSessionId` matches), OR
/// - The current session's agent name appears in the team's member list
///
/// Also scans ~/.claude/tasks/ to determine agent health (active vs idle).
pub fn collect(session_id: Option<&str>, agent_name: Option<&str>) -> TeamsInfo {
    let mut info = TeamsInfo::default();

    // Without a session_id we can't determine relevance — skip
    if session_id.is_none() && agent_name.is_none() {
        return info;
    }

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return info,
    };

    let teams_dir = home.join(".claude/teams");
    if !teams_dir.is_dir() {
        return info;
    }

    let entries = match std::fs::read_dir(&teams_dir) {
        Ok(e) => e,
        Err(_) => return info,
    };

    let mut all_member_names: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let config_path = path.join("config.json");
        if !config_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let json: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Check if this team belongs to the current session
        if !is_team_relevant(&json, session_id, agent_name) {
            continue;
        }

        let team_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let member_count = count_members(&json).unwrap_or(0);

        // Collect member names for health check
        if let Some(members) = json.get("members").and_then(|m| m.as_array()) {
            for member in members {
                if let Some(name) = member.get("name").and_then(|n| n.as_str()) {
                    all_member_names.push(name.to_string());
                }
            }
        }

        info.teams.push(Team {
            name: team_name,
            member_count,
        });
    }

    // Scan tasks to determine active vs idle agents
    if !all_member_names.is_empty() {
        let (active, idle) = compute_agent_health(&home, &all_member_names);
        info.active_agents = active;
        info.idle_agents = idle;
    }

    info
}

/// Check task files to see which agents have in_progress tasks.
fn compute_agent_health(home: &std::path::Path, member_names: &[String]) -> (usize, usize) {
    let tasks_dir = home.join(".claude/tasks");
    if !tasks_dir.is_dir() {
        return (0, member_names.len());
    }

    let mut active_owners: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Scan all session dirs under tasks/
    if let Ok(sessions) = std::fs::read_dir(&tasks_dir) {
        for session_entry in sessions.flatten() {
            let session_path = session_entry.path();
            if !session_path.is_dir() {
                continue;
            }

            if let Ok(task_files) = std::fs::read_dir(&session_path) {
                for task_entry in task_files.flatten() {
                    let task_path = task_entry.path();
                    if task_path.extension().and_then(|e| e.to_str()) != Some("json") {
                        continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(&task_path) {
                        if let Ok(json) = serde_json::from_str::<Value>(&content) {
                            let status = json.get("status").and_then(|s| s.as_str()).unwrap_or("");
                            let owner = json.get("owner").and_then(|o| o.as_str()).unwrap_or("");

                            if status == "in_progress" && !owner.is_empty() {
                                active_owners.insert(owner.to_lowercase());
                            }
                        }
                    }
                }
            }
        }
    }

    let active = member_names
        .iter()
        .filter(|name| active_owners.contains(&name.to_lowercase()))
        .count();
    let idle = member_names.len().saturating_sub(active);

    (active, idle)
}

/// Check if a team config is relevant to the current session.
fn is_team_relevant(json: &Value, session_id: Option<&str>, agent_name: Option<&str>) -> bool {
    // Match 1: current session is the team lead
    if let Some(sid) = session_id {
        if let Some(lead_sid) = json.get("leadSessionId").and_then(|v| v.as_str()) {
            if lead_sid == sid {
                return true;
            }
        }
    }

    // Match 2: current agent name appears in the members list
    if let Some(name) = agent_name {
        if let Some(members) = json.get("members").and_then(|m| m.as_array()) {
            for member in members {
                if let Some(member_name) = member.get("name").and_then(|n| n.as_str()) {
                    if member_name == name {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Count team members from config JSON.
fn count_members(json: &Value) -> Option<usize> {
    if let Some(members) = json.get("members").and_then(|m| m.as_array()) {
        return Some(members.len());
    }

    if let Some(agents) = json.get("agents").and_then(|a| a.as_array()) {
        return Some(agents.len());
    }

    if let Some(count) = json.get("member_count").and_then(|c| c.as_u64()) {
        return Some(count as usize);
    }

    None
}
