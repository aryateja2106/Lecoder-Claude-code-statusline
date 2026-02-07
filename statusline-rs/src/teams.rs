use serde_json::Value;

/// Agent team information.
#[derive(Debug, Default)]
pub struct TeamsInfo {
    pub teams: Vec<Team>,
}

#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub member_count: usize,
}

/// Read ~/.claude/teams/*/config.json to find active agent teams.
pub fn collect() -> TeamsInfo {
    let mut info = TeamsInfo::default();

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

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let config_path = path.join("config.json");
        if !config_path.exists() {
            continue;
        }

        let team_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let member_count = read_team_member_count(&config_path).unwrap_or(0);

        info.teams.push(Team {
            name: team_name,
            member_count,
        });
    }

    info
}

fn read_team_member_count(config_path: &std::path::Path) -> Option<usize> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;

    // Try "members" array
    if let Some(members) = json.get("members").and_then(|m| m.as_array()) {
        return Some(members.len());
    }

    // Try "agents" array
    if let Some(agents) = json.get("agents").and_then(|a| a.as_array()) {
        return Some(agents.len());
    }

    // Try "member_count" number
    if let Some(count) = json.get("member_count").and_then(|c| c.as_u64()) {
        return Some(count as usize);
    }

    None
}
