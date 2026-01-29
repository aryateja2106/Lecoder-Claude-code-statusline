#!/bin/bash

# ============================================================================
# Claude Code Statusline - Enhanced Git Status Component (Warp-style)
# ============================================================================
#
# Shows git status in Warp terminal style:
# - Number of changed files (📄N)
# - Lines added (+N in green)
# - Lines removed (-N in red)
# - Commits made today (📝N)
#
# Display format: 📄 3 • +45 -12 • 📝2
#
# Dependencies: git.sh, display.sh
# ============================================================================

# Component data storage
COMPONENT_GIT_CHANGED_FILES=""
COMPONENT_GIT_LINES_ADDED=""
COMPONENT_GIT_LINES_REMOVED=""
COMPONENT_GIT_COMMITS_TODAY=""

# ============================================================================
# COMPONENT DATA COLLECTION
# ============================================================================

collect_git_status_data() {
    debug_log "Collecting git_status component data" "INFO"

    # Initialize defaults
    COMPONENT_GIT_CHANGED_FILES="0"
    COMPONENT_GIT_LINES_ADDED="0"
    COMPONENT_GIT_LINES_REMOVED="0"
    COMPONENT_GIT_COMMITS_TODAY="0"

    if ! is_git_repository 2>/dev/null; then
        debug_log "Not a git repository" "INFO"
        return 0
    fi

    # Get count of all changed files (staged + modified + untracked)
    local staged modified untracked
    staged=$(git diff --cached --name-only 2>/dev/null | wc -l | tr -d ' ')
    modified=$(git diff --name-only 2>/dev/null | wc -l | tr -d ' ')
    untracked=$(git ls-files --others --exclude-standard 2>/dev/null | wc -l | tr -d ' ')
    COMPONENT_GIT_CHANGED_FILES=$((staged + modified + untracked))

    # Get lines added and removed (from both staged and unstaged changes)
    local diffstat
    diffstat=$(git diff --numstat HEAD 2>/dev/null)

    if [[ -n "$diffstat" ]]; then
        COMPONENT_GIT_LINES_ADDED=$(echo "$diffstat" | awk '{sum+=$1} END {print sum+0}')
        COMPONENT_GIT_LINES_REMOVED=$(echo "$diffstat" | awk '{sum+=$2} END {print sum+0}')
    fi

    # Get commits made today
    COMPONENT_GIT_COMMITS_TODAY=$(git log --oneline --since="midnight" 2>/dev/null | wc -l | tr -d ' ')

    debug_log "git_status: files=$COMPONENT_GIT_CHANGED_FILES +$COMPONENT_GIT_LINES_ADDED -$COMPONENT_GIT_LINES_REMOVED commits=$COMPONENT_GIT_COMMITS_TODAY" "INFO"
    return 0
}

# ============================================================================
# COMPONENT RENDERING
# ============================================================================

render_git_status() {
    local output=""
    local parts=()

    # Check if we're in a git repo
    if ! is_git_repository 2>/dev/null; then
        echo ""
        return 0
    fi

    # Show number of changed files
    if [[ "$COMPONENT_GIT_CHANGED_FILES" -gt 0 ]]; then
        parts+=("📄 ${COMPONENT_GIT_CHANGED_FILES}")
    fi

    # Show lines added/removed
    if [[ "$COMPONENT_GIT_LINES_ADDED" -gt 0 ]] || [[ "$COMPONENT_GIT_LINES_REMOVED" -gt 0 ]]; then
        local line_changes=""

        if [[ "$COMPONENT_GIT_LINES_ADDED" -gt 0 ]]; then
            line_changes="${CONFIG_GREEN}+${COMPONENT_GIT_LINES_ADDED}${CONFIG_RESET}"
        fi

        if [[ "$COMPONENT_GIT_LINES_REMOVED" -gt 0 ]]; then
            [[ -n "$line_changes" ]] && line_changes="${line_changes} "
            line_changes="${line_changes}${CONFIG_RED}-${COMPONENT_GIT_LINES_REMOVED}${CONFIG_RESET}"
        fi

        if [[ -n "$line_changes" ]]; then
            parts+=("$line_changes")
        fi
    fi

    # Show commits made today
    if [[ "$COMPONENT_GIT_COMMITS_TODAY" -gt 0 ]]; then
        parts+=("${CONFIG_BLUE}📝${COMPONENT_GIT_COMMITS_TODAY}${CONFIG_RESET}")
    fi

    # If no changes, show clean status
    if [[ ${#parts[@]} -eq 0 ]]; then
        echo "${CONFIG_GREEN}✓ clean${CONFIG_RESET}"
        return 0
    fi

    # Join parts with bullet separator
    local first=true
    for part in "${parts[@]}"; do
        if [[ "$first" == "true" ]]; then
            output="$part"
            first=false
        else
            output="${output} ${CONFIG_LIGHT_GRAY}•${CONFIG_RESET} $part"
        fi
    done

    echo "$output"
    return 0
}

# ============================================================================
# COMPONENT CONFIGURATION
# ============================================================================

get_git_status_config() {
    local config_key="$1"
    local default_value="$2"

    case "$config_key" in
        "enabled")
            get_component_config "git_status" "enabled" "${default_value:-true}"
            ;;
        *)
            echo "$default_value"
            ;;
    esac
}

# ============================================================================
# COMPONENT REGISTRATION
# ============================================================================

register_component \
    "git_status" \
    "Enhanced git status with changes and worktree info" \
    "git" \
    "$(get_git_status_config 'enabled' 'true')"

debug_log "Git status component loaded" "INFO"
