#!/bin/bash
# ============================================================================
# Claude Code Statusline - Git Worktrees Component
# ============================================================================
# Displays git worktree status and count
# Format: 🌳 feature-x [3 worktrees]
# ============================================================================

# Component data storage
COMPONENT_WORKTREE_NAME=""
COMPONENT_WORKTREE_COUNT=""
COMPONENT_IS_WORKTREE=""

# ============================================================================
# DATA COLLECTION
# ============================================================================

collect_git_worktrees_data() {
    debug_log "Collecting git_worktrees component data" "INFO" 2>/dev/null || true

    COMPONENT_WORKTREE_NAME=""
    COMPONENT_WORKTREE_COUNT=""
    COMPONENT_IS_WORKTREE="false"

    # Check if in git repo
    local git_dir common_dir toplevel
    git_dir=$(git rev-parse --git-dir 2>/dev/null) || return 0
    common_dir=$(git rev-parse --git-common-dir 2>/dev/null)
    toplevel=$(git rev-parse --show-toplevel 2>/dev/null)

    # Check if we're in a linked worktree (git-dir != git-common-dir)
    if [[ "$git_dir" != "$common_dir" ]] && [[ -n "$common_dir" ]]; then
        COMPONENT_IS_WORKTREE="true"
        COMPONENT_WORKTREE_NAME=$(basename "$toplevel")
    fi

    # Get total worktree count
    COMPONENT_WORKTREE_COUNT=$(git worktree list 2>/dev/null | wc -l | tr -d ' ')

    debug_log "git_worktrees: name=${COMPONENT_WORKTREE_NAME}, count=${COMPONENT_WORKTREE_COUNT}, is_worktree=${COMPONENT_IS_WORKTREE}" "INFO" 2>/dev/null || true
}

# ============================================================================
# RENDERING
# ============================================================================

render_git_worktrees() {
    local theme_enabled="${1:-true}"

    # Only show if there are multiple worktrees or we're in a linked worktree
    if [[ -z "$COMPONENT_WORKTREE_COUNT" || "$COMPONENT_WORKTREE_COUNT" -le 1 ]]; then
        return 1  # No content - single worktree (main) only
    fi

    local emoji="🌳"
    local output=""

    if [[ "$COMPONENT_IS_WORKTREE" == "true" && -n "$COMPONENT_WORKTREE_NAME" ]]; then
        # In a linked worktree
        output="${emoji} ${COMPONENT_WORKTREE_NAME} [${COMPONENT_WORKTREE_COUNT} worktrees]"
    else
        # In main worktree but have linked worktrees
        output="${emoji} main [${COMPONENT_WORKTREE_COUNT} worktrees]"
    fi

    echo "$output"
}

# ============================================================================
# COMPONENT REGISTRATION
# ============================================================================

# Only register if the function exists (may not be loaded in standalone tests)
if declare -f register_component &>/dev/null; then
    register_component \
        "git_worktrees" \
        "Git worktree status and count" \
        "" \
        "true"
fi

debug_log "Git worktrees component loaded" "INFO" 2>/dev/null || true
