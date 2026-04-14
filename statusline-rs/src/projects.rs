/// Multi-project awareness: scan ~/.claude/projects/ for active sessions.

#[derive(Debug, Default)]
pub struct ProjectsInfo {
    pub projects: Vec<ProjectEntry>,
}

#[derive(Debug)]
pub struct ProjectEntry {
    pub name: String,
}

/// Scan ~/.claude/projects/ for project directories.
/// Each subdirectory represents a project the user has worked on.
/// We show the most recent ones based on directory modification time.
pub fn collect() -> ProjectsInfo {
    let mut info = ProjectsInfo::default();

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return info,
    };

    let projects_dir = home.join(".claude/projects");
    if !projects_dir.is_dir() {
        return info;
    }

    let entries = match std::fs::read_dir(&projects_dir) {
        Ok(e) => e,
        Err(_) => return info,
    };

    // Collect project dirs with their modification times
    let mut projects: Vec<(String, std::time::SystemTime)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Claude Code encodes project paths as directory names with dashes
        // e.g., "-Users-arya-Projects-flow" => "flow"
        let project_name = extract_project_name(&dir_name);

        let mtime = path
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::UNIX_EPOCH);

        projects.push((project_name, mtime));
    }

    // Sort by most recently modified first
    projects.sort_by(|a, b| b.1.cmp(&a.1));

    // Take up to 5 most recent
    for (name, _) in projects.into_iter().take(5) {
        info.projects.push(ProjectEntry { name });
    }

    info
}

/// Extract a human-readable project name from the encoded directory name.
/// "-Users-arya-Desktop-Claude-WorkOnMac-flow" => "flow"
/// "-Users-arya-Projects-mconnect" => "mconnect"
fn extract_project_name(encoded: &str) -> String {
    // The directory name is the full path with / replaced by -
    // Take the last segment as the project name
    let parts: Vec<&str> = encoded.split('-').collect();
    if let Some(last) = parts.last() {
        if !last.is_empty() {
            return last.to_string();
        }
    }
    // Fallback: try last two segments
    if parts.len() >= 2 {
        let second_last = parts[parts.len() - 2];
        let last = parts[parts.len() - 1];
        if last.is_empty() && !second_last.is_empty() {
            return second_last.to_string();
        }
    }
    encoded.to_string()
}
