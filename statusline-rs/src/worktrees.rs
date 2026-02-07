use git2::Repository;
use std::path::Path;

/// Git worktree information.
#[derive(Debug, Default)]
pub struct WorktreeInfo {
    pub worktrees: Vec<Worktree>,
}

/// Represents a single git worktree with detailed information.
#[derive(Debug)]
pub struct Worktree {
    pub name: String,           // e.g., "mconnect-mcp-integration"
    pub branch: Option<String>, // e.g., "feat/container-mcp-integration"
    pub path: String,           // e.g., "~/Desktop/Claude-WorkOnMac/worktrees/mconnect-mcp-integration"
    pub is_main: bool,          // true if this is the main worktree (not a linked worktree)
}

/// List git worktrees for the repository at `cwd`.
///
/// This function collects information about all linked worktrees in the repository.
/// Note: The main worktree is NOT included in this list, as it's already displayed
/// in the primary statusline. This only returns linked worktrees created via
/// `git worktree add`.
pub fn collect(cwd: &Path) -> WorktreeInfo {
    let mut info = WorktreeInfo::default();

    let repo = match Repository::discover(cwd) {
        Ok(r) => r,
        Err(_) => return info,
    };

    let worktree_names = match repo.worktrees() {
        Ok(wt) => wt,
        Err(_) => return info,
    };

    for name_opt in worktree_names.iter() {
        if let Some(name) = name_opt {
            // Get the worktree object to access its path
            let worktree = match repo.find_worktree(name) {
                Ok(wt) => wt,
                Err(_) => continue,
            };

            // Get the worktree path
            let wt_path = worktree.path();
            let path_str = wt_path.to_string_lossy().to_string();
            let shortened_path = shorten_path(&path_str);

            // Get the branch name for this worktree
            let branch = get_worktree_branch(&repo, name);

            // All worktrees from repo.worktrees() are linked worktrees, not the main one
            info.worktrees.push(Worktree {
                name: name.to_string(),
                branch,
                path: shortened_path,
                is_main: false,
            });
        }
    }

    info
}

/// Get the current branch name for a specific worktree.
fn get_worktree_branch(repo: &Repository, name: &str) -> Option<String> {
    let wt = repo.find_worktree(name).ok()?;
    // Try to open the worktree as a repo to get its HEAD
    let wt_path = wt.path();
    let wt_repo = Repository::open(wt_path).ok()?;
    let head = wt_repo.head().ok()?;
    head.shorthand().map(|s| s.to_string())
}

/// Shorten file paths by replacing the home directory with ~.
///
/// This makes paths more readable in the statusline by converting paths like:
/// `/Users/username/Desktop/project` to `~/Desktop/project`
fn shorten_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        if path.starts_with(&home_str) {
            return format!("~{}", &path[home_str.len()..]);
        }
    }
    path.to_string()
}
