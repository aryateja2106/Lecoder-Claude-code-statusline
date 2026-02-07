use git2::{Repository, StatusOptions};
use std::path::Path;

/// Collected git repository information.
#[derive(Debug, Default)]
pub struct GitInfo {
    pub branch: String,
    pub is_clean: bool,
    pub added: usize,
    pub deleted: usize,
    pub modified: usize,
    pub ahead: usize,
    pub behind: usize,
    pub repo_path: String,
}

/// Gather git status for the current working directory.
/// Returns None if not inside a git repository.
pub fn collect(cwd: &Path) -> Option<GitInfo> {
    let repo = Repository::discover(cwd).ok()?;

    let mut info = GitInfo::default();

    // Repository root path
    if let Some(workdir) = repo.workdir() {
        info.repo_path = workdir.display().to_string();
    }

    // Branch name
    info.branch = get_branch_name(&repo);

    // File status
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false)
        .include_ignored(false);

    if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
        let mut clean = true;
        for entry in statuses.iter() {
            let s = entry.status();
            if s.intersects(
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_RENAMED
                    | git2::Status::INDEX_TYPECHANGE,
            ) {
                clean = false;
            }
            if s.intersects(
                git2::Status::WT_NEW
                    | git2::Status::WT_MODIFIED
                    | git2::Status::WT_DELETED
                    | git2::Status::WT_RENAMED
                    | git2::Status::WT_TYPECHANGE,
            ) {
                clean = false;
            }
            if s.intersects(git2::Status::INDEX_NEW | git2::Status::WT_NEW) {
                info.added += 1;
            }
            if s.intersects(git2::Status::INDEX_DELETED | git2::Status::WT_DELETED) {
                info.deleted += 1;
            }
            if s.intersects(git2::Status::INDEX_MODIFIED | git2::Status::WT_MODIFIED) {
                info.modified += 1;
            }
        }
        info.is_clean = clean;
    }

    // Ahead/behind tracking branch
    if let Ok(head) = repo.head() {
        if let Some(local_oid) = head.target() {
            let branch_name = info.branch.clone();
            let upstream_name = format!("refs/remotes/origin/{branch_name}");
            if let Ok(upstream_ref) = repo.find_reference(&upstream_name) {
                if let Some(upstream_oid) = upstream_ref.target() {
                    if let Ok((ahead, behind)) = repo.graph_ahead_behind(local_oid, upstream_oid) {
                        info.ahead = ahead;
                        info.behind = behind;
                    }
                }
            }
        }
    }

    Some(info)
}

fn get_branch_name(repo: &Repository) -> String {
    // Try HEAD reference first
    if let Ok(head) = repo.head() {
        if head.is_branch() {
            if let Some(name) = head.shorthand() {
                return name.to_string();
            }
        }
        // Detached HEAD — show short hash
        if let Some(oid) = head.target() {
            let short = &oid.to_string()[..7];
            return format!("detached:{short}");
        }
    }
    "unknown".into()
}
