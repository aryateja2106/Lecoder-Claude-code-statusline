/// Skills & commands awareness: count global and project-level skills/commands.
/// Global: ~/.claude/skills/*, ~/.claude/commands/*.md
/// Project: <cwd>/.claude/skills/*, <cwd>/.claude/commands/*.md

use std::path::Path;

#[derive(Debug, Default)]
pub struct SkillsInfo {
    /// Names of global skills (from ~/.claude/skills/)
    pub global_skills: Vec<String>,
    /// Names of global commands (from ~/.claude/commands/)
    pub global_commands: Vec<String>,
    /// Names of project-local skills (from <project>/.claude/skills/)
    pub project_skills: Vec<String>,
    /// Names of project-local commands (from <project>/.claude/commands/)
    pub project_commands: Vec<String>,
}

impl SkillsInfo {
    pub fn total_global(&self) -> usize {
        self.global_skills.len()
    }

    pub fn total_project(&self) -> usize {
        self.project_skills.len()
    }

    pub fn total(&self) -> usize {
        self.total_global() + self.total_project()
    }
}

/// Collect skills info for the current working directory.
pub fn collect(cwd: &Path) -> SkillsInfo {
    let mut info = SkillsInfo::default();

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return info,
    };

    // Global skills: ~/.claude/skills/*/
    let global_skills_dir = home.join(".claude/skills");
    info.global_skills = list_subdirs(&global_skills_dir);

    // Global commands: ~/.claude/commands/*.md
    let global_commands_dir = home.join(".claude/commands");
    info.global_commands = list_md_files(&global_commands_dir);

    // Project skills: <cwd>/.claude/skills/*/
    let project_skills_dir = cwd.join(".claude/skills");
    info.project_skills = list_subdirs(&project_skills_dir);

    // Project commands: <cwd>/.claude/commands/*.md
    let project_commands_dir = cwd.join(".claude/commands");
    info.project_commands = list_md_files(&project_commands_dir);

    info
}

/// List subdirectory names (for skill directories).
fn list_subdirs(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    names
}

/// List .md filenames (without extension) for commands.
fn list_md_files(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".md") {
                        names.push(name.trim_end_matches(".md").to_string());
                    }
                }
            }
        }
    }
    names.sort();
    names
}
