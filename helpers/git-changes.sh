#!/bin/bash
#
# Get detailed git changes for statusline display
#
# Outputs: MODIFIED|STAGED|UNTRACKED|BRANCH
# Example: 5|2|1|main
#
# MODIFIED = files with unstaged changes (working tree modified)
# STAGED = files staged for commit (index changes)
# UNTRACKED = untracked files
#

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
  echo "0|0|0|none"
  exit 0
fi

# Get current branch name
branch=$(git branch --show-current 2>/dev/null)
[ -z "$branch" ] && branch=$(git rev-parse --short HEAD 2>/dev/null)
[ -z "$branch" ] && branch="detached"

# Initialize counters
modified=0
staged=0
untracked=0

# Parse git status --porcelain output
# Format: XY PATH
# X = index status (staged), Y = working tree status (unstaged)
#
# Common status codes:
# ' ' = unmodified
# M = modified
# A = added
# D = deleted
# R = renamed
# C = copied
# U = updated but unmerged
# ? = untracked

status_output=$(git status --porcelain 2>/dev/null)

if [ -n "$status_output" ]; then
  while IFS= read -r line; do
    [ -z "$line" ] && continue

    index_status="${line:0:1}"
    worktree_status="${line:1:1}"

    # Untracked files
    if [ "$index_status" = "?" ]; then
      untracked=$((untracked + 1))
      continue
    fi

    # Staged changes (index has changes)
    if [[ "$index_status" =~ [MADRC] ]]; then
      staged=$((staged + 1))
    fi

    # Modified in working tree (unstaged changes)
    if [ "$worktree_status" = "M" ] || [ "$worktree_status" = "D" ]; then
      modified=$((modified + 1))
    fi
  done <<< "$status_output"
fi

echo "${modified}|${staged}|${untracked}|${branch}"
