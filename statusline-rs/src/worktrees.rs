use git2::Repository;
use std::path::Path;

/// Git worktree information.
#[derive(Debug, Default)]
pub struct WorktreeInfo {
    pub worktrees: Vec<Worktree>,
}

#[derive(Debug)]
pub struct Worktree {
    pub name: String,
    pub branch: Option<String>,
}

/// List git worktrees for the repository at `cwd`.
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
            let branch = get_worktree_branch(&repo, name);
            info.worktrees.push(Worktree {
                name: name.to_string(),
                branch,
            });
        }
    }

    info
}

fn get_worktree_branch(repo: &Repository, name: &str) -> Option<String> {
    let wt = repo.find_worktree(name).ok()?;
    // Try to open the worktree as a repo to get its HEAD
    let wt_path = wt.path();
    let wt_repo = Repository::open(wt_path).ok()?;
    let head = wt_repo.head().ok()?;
    head.shorthand().map(|s| s.to_string())
}
