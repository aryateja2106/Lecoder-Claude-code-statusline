#!/bin/bash
#
# Get git repository status for statusline display
#
# Outputs: BRANCH MODIFIED ADDED (or "none" if not in git repo)
#

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "none 0 0"
    exit 0
fi

# Get current branch name
branch=$(git symbolic-ref --short HEAD 2>/dev/null || git rev-parse --short HEAD 2>/dev/null || echo "detached")

# Get file status counts
modified=0
added=0

# Parse git status --porcelain output
# Format: XY PATH
# X = index status, Y = working tree status
# M = modified, A = added, D = deleted, R = renamed, C = copied, U = updated but unmerged
while IFS= read -r line; do
    status="${line:0:2}"
    index_status="${status:0:1}"
    worktree_status="${status:1:1}"

    # Count modified files (both staged and unstaged)
    if [[ "$index_status" == "M" ]] || [[ "$worktree_status" == "M" ]]; then
        modified=$((modified + 1))
    fi

    # Count added/untracked files
    if [[ "$index_status" == "A" ]] || [[ "$index_status" == "?" ]] || [[ "$worktree_status" == "?" ]]; then
        added=$((added + 1))
    fi
done < <(git status --porcelain 2>/dev/null)

echo "${branch} ${modified} ${added}"
