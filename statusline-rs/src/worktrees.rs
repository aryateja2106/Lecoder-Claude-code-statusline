use git2::Repository;
use std::path::Path;

/// Git worktree information.
#[derive(Debug, Default)]
pub struct WorktreeInfo {
    pub worktrees: Vec<Worktree>,
    pub current: Option<Worktree>,
    pub main_worktree: Option<Worktree>,
    pub total_count: usize,
}

/// Represents a single git worktree with detailed information.
#[derive(Debug, Clone)]
pub struct Worktree {
    pub name: String,           // e.g., "mconnect-mcp-integration"
    pub branch: Option<String>, // e.g., "feat/container-mcp-integration"
    pub path: String,           // e.g., "~/Desktop/Claude-WorkOnMac/worktrees/mconnect-mcp-integration"
    pub is_main: bool,          // true if this is the main worktree (not a linked worktree)
}

/// List git worktrees for the repository at `cwd`.
///
/// Collects all linked worktrees and identifies which one (or main) matches `cwd`.
/// The `current` field holds whichever worktree the user is currently inside.
/// `total_count` includes the main worktree + all linked worktrees.
pub fn collect(cwd: &Path) -> WorktreeInfo {
    let mut info = WorktreeInfo::default();

    let repo = match Repository::discover(cwd) {
        Ok(r) => r,
        Err(_) => return info,
    };

    // Main worktree
    let main_dir = match repo.workdir() {
        Some(p) => p.to_path_buf(),
        None => return info, // bare repo
    };
    let main_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()));

    // Collect linked worktrees with raw paths (before shortening) for cwd matching
    let mut linked_raw: Vec<(String, Option<String>, std::path::PathBuf)> = Vec::new();

    if let Ok(worktree_names) = repo.worktrees() {
        for name_opt in worktree_names.iter() {
            if let Some(name) = name_opt {
                if let Ok(wt) = repo.find_worktree(name) {
                    let wt_path = wt.path().to_path_buf();
                    let branch = get_worktree_branch(&repo, name);
                    linked_raw.push((name.to_string(), branch, wt_path));
                }
            }
        }
    }

    info.total_count = 1 + linked_raw.len();

    // Only populate if there are linked worktrees (otherwise nothing to show)
    if linked_raw.is_empty() {
        return info;
    }

    // Determine current worktree — check linked first (more specific paths)
    for (name, branch, raw_path) in &linked_raw {
        if cwd.starts_with(raw_path) {
            info.current = Some(Worktree {
                name: name.clone(),
                branch: branch.clone(),
                path: shorten_path(&raw_path.to_string_lossy()),
                is_main: false,
            });
            break;
        }
    }

    // Always store main worktree info for display from linked worktrees
    let main_wt = Worktree {
        name: "main".to_string(),
        branch: main_branch,
        path: shorten_path(&main_dir.to_string_lossy()),
        is_main: true,
    };
    info.main_worktree = Some(main_wt.clone());

    // If not in a linked worktree, must be in main
    if info.current.is_none() && cwd.starts_with(&main_dir) {
        info.current = Some(main_wt);
    }

    // Build linked worktrees list
    info.worktrees = linked_raw
        .into_iter()
        .map(|(name, branch, raw_path)| Worktree {
            name,
            branch,
            path: shorten_path(&raw_path.to_string_lossy()),
            is_main: false,
        })
        .collect();

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
