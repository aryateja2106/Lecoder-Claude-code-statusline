#!/bin/bash
#
# Detect git worktree status for statusline display
#
# Outputs: WORKTREE_NAME|BRANCH|IS_WORKTREE(0/1)
# Example: my-feature|main|1  (in linked worktree)
# Example: |main|0  (in main worktree or regular repo)
#

# Check if we're in a git repository
git_dir=$(git rev-parse --git-dir 2>/dev/null) || {
  echo "||0"
  exit 0
}

# Get common git directory (for worktree detection)
common_dir=$(git rev-parse --git-common-dir 2>/dev/null)
toplevel=$(git rev-parse --show-toplevel 2>/dev/null)

# Get current branch
branch=$(git branch --show-current 2>/dev/null)
[ -z "$branch" ] && branch=$(git rev-parse --short HEAD 2>/dev/null)
[ -z "$branch" ] && branch="detached"

# Check if we're in a linked worktree
# In a linked worktree, git-dir != git-common-dir
if [ "$git_dir" != "$common_dir" ] && [ -n "$common_dir" ]; then
  worktree_name=$(basename "$toplevel")
  echo "$worktree_name|$branch|1"
else
  echo "|$branch|0"
fi
