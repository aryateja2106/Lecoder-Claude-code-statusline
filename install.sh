#!/usr/bin/env bash
# ============================================================================
# Claude Code Statusline - Interactive Installer
# ============================================================================
# Installs the Claude Code statusline with interactive configuration wizard.
# Supports macOS and Linux (Ubuntu, Debian, etc.)
#
# Usage:
#   Interactive:     ./install.sh
#   Non-interactive: ./install.sh --profile=minimal --no-interactive
#   Help:            ./install.sh --help
# ============================================================================

set -euo pipefail

# Version
INSTALLER_VERSION="3.0.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default values
INSTALL_DIR="$HOME/.claude/statusline"
SETTINGS_FILE="$HOME/.claude/settings.json"
PROFILE="standard"
THEME="catppuccin"
INTERACTIVE=true
FORCE=false

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

print_banner() {
    echo -e "${CYAN}"
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║       Claude Code Statusline Installer v${INSTALLER_VERSION}              ║"
    echo "║       A customizable multi-line statusline for Claude Code    ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ============================================================================
# PLATFORM DETECTION
# ============================================================================

detect_platform() {
    local platform
    platform=$(uname -s)

    case "$platform" in
        Darwin)
            echo "macos"
            ;;
        Linux)
            # Check if we're in a container
            if [[ -f /.dockerenv ]] || grep -q 'docker\|lxc' /proc/1/cgroup 2>/dev/null; then
                echo "linux-container"
            else
                echo "linux"
            fi
            ;;
        *)
            echo "unsupported"
            ;;
    esac
}

get_bash_version() {
    echo "${BASH_VERSION%%.*}"
}

# ============================================================================
# DEPENDENCY CHECKING
# ============================================================================

check_dependencies() {
    local missing=()
    local platform
    platform=$(detect_platform)

    # Check bash version (4.0+ required for associative arrays)
    if [[ $(get_bash_version) -lt 4 ]]; then
        log_warn "Bash ${BASH_VERSION} detected. Version 4.0+ recommended for full features."
        if [[ "$platform" == "macos" ]]; then
            log_info "Install with: brew install bash"
        else
            log_info "Install with: sudo apt install bash"
        fi
    fi

    # Check for jq
    if ! command -v jq &>/dev/null; then
        missing+=("jq")
    fi

    # Check for git
    if ! command -v git &>/dev/null; then
        missing+=("git")
    fi

    # Check for bc (math operations)
    if ! command -v bc &>/dev/null; then
        missing+=("bc")
    fi

    # Check for curl (for updates)
    if ! command -v curl &>/dev/null; then
        missing+=("curl")
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing dependencies: ${missing[*]}"
        echo ""
        if [[ "$platform" == "macos" ]]; then
            log_info "Install with: brew install ${missing[*]}"
        else
            log_info "Install with: sudo apt install ${missing[*]}"
        fi
        echo ""
        return 1
    fi

    log_success "All dependencies satisfied"
    return 0
}

# ============================================================================
# INTERACTIVE WIZARD
# ============================================================================

prompt_choice() {
    local prompt="$1"
    local default="$2"
    shift 2
    local options=("$@")

    echo ""
    echo -e "${CYAN}$prompt${NC}"
    echo ""

    local i=1
    for opt in "${options[@]}"; do
        if [[ "$opt" == "$default" ]]; then
            echo -e "  ${GREEN}$i)${NC} $opt ${GREEN}(default)${NC}"
        else
            echo "  $i) $opt"
        fi
        ((i++))
    done

    echo ""
    read -rp "Enter choice [1-${#options[@]}] (default: find default): " choice

    if [[ -z "$choice" ]]; then
        echo "$default"
        return
    fi

    if [[ "$choice" =~ ^[0-9]+$ ]] && [[ "$choice" -ge 1 ]] && [[ "$choice" -le ${#options[@]} ]]; then
        echo "${options[$((choice-1))]}"
    else
        echo "$default"
    fi
}

run_wizard() {
    echo ""
    log_info "Starting interactive configuration wizard..."
    echo ""

    # Profile selection
    PROFILE=$(prompt_choice "Select installation profile:" "standard" \
        "minimal" "standard" "full" "developer")

    echo -e "${GREEN}Selected profile:${NC} $PROFILE"

    # Theme selection
    THEME=$(prompt_choice "Select theme:" "catppuccin" \
        "classic" "garden" "catppuccin" "custom")

    echo -e "${GREEN}Selected theme:${NC} $THEME"

    # Features based on profile
    echo ""
    log_info "Profile '$PROFILE' features:"
    case "$PROFILE" in
        minimal)
            echo "  • Git status"
            echo "  • Model info"
            echo "  • Directory path"
            ;;
        standard)
            echo "  • Git status"
            echo "  • Model info"
            echo "  • MCP server status"
            echo "  • Usage limits"
            echo "  • Cost tracking"
            ;;
        full)
            echo "  • All standard features"
            echo "  • Container stats (Docker/Podman)"
            echo "  • Git worktrees"
            echo "  • Session info"
            ;;
        developer)
            echo "  • All full features"
            echo "  • Debug timing"
            echo "  • Cache statistics"
            ;;
    esac

    echo ""
    read -rp "Continue with installation? [Y/n] " confirm
    if [[ "$confirm" =~ ^[Nn] ]]; then
        log_info "Installation cancelled"
        exit 0
    fi
}

# ============================================================================
# INSTALLATION
# ============================================================================

backup_existing() {
    if [[ -d "$INSTALL_DIR" ]]; then
        local backup_dir="${INSTALL_DIR}.backup.$(date +%Y%m%d_%H%M%S)"
        log_info "Backing up existing installation to $backup_dir"
        mv "$INSTALL_DIR" "$backup_dir"
    fi

    if [[ -f "$SETTINGS_FILE" ]]; then
        local backup_file="${SETTINGS_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
        log_info "Backing up settings to $backup_file"
        cp "$SETTINGS_FILE" "$backup_file"
    fi
}

install_files() {
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    log_info "Installing statusline to $INSTALL_DIR"

    # Create directory structure
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$HOME/.claude"

    # Copy statusline files
    if [[ -d "$script_dir/statusline" ]]; then
        cp -r "$script_dir/statusline/"* "$INSTALL_DIR/"
    else
        log_error "Source statusline directory not found"
        return 1
    fi

    # Make scripts executable
    find "$INSTALL_DIR" -name "*.sh" -exec chmod +x {} \;

    log_success "Statusline files installed"
}

configure_theme() {
    local config_file="$INSTALL_DIR/Config.toml"

    if [[ -f "$config_file" ]]; then
        # Update theme in config
        if [[ "$(uname -s)" == "Darwin" ]]; then
            sed -i '' "s/^theme.name = .*/theme.name = \"$THEME\"/" "$config_file"
        else
            sed -i "s/^theme.name = .*/theme.name = \"$THEME\"/" "$config_file"
        fi
        log_success "Theme set to: $THEME"
    fi
}

configure_profile() {
    local config_file="$INSTALL_DIR/Config.toml"

    if [[ ! -f "$config_file" ]]; then
        return 0
    fi

    case "$PROFILE" in
        minimal)
            # Disable most features
            if [[ "$(uname -s)" == "Darwin" ]]; then
                sed -i '' 's/features.show_mcp_status = true/features.show_mcp_status = false/' "$config_file"
                sed -i '' 's/features.show_cost_tracking = true/features.show_cost_tracking = false/' "$config_file"
                sed -i '' 's/features.show_reset_info = true/features.show_reset_info = false/' "$config_file"
            else
                sed -i 's/features.show_mcp_status = true/features.show_mcp_status = false/' "$config_file"
                sed -i 's/features.show_cost_tracking = true/features.show_cost_tracking = false/' "$config_file"
                sed -i 's/features.show_reset_info = true/features.show_reset_info = false/' "$config_file"
            fi
            ;;
        full|developer)
            # Enable container stats
            if ! grep -q "features.show_container_stats" "$config_file"; then
                echo "" >> "$config_file"
                echo "# Container monitoring" >> "$config_file"
                echo "features.show_container_stats = true" >> "$config_file"
            fi
            ;;
    esac

    log_success "Profile configured: $PROFILE"
}

update_settings_json() {
    log_info "Updating Claude Code settings.json"

    # Create settings.json if it doesn't exist
    if [[ ! -f "$SETTINGS_FILE" ]]; then
        echo '{}' > "$SETTINGS_FILE"
    fi

    # Read current settings
    local current_settings
    current_settings=$(cat "$SETTINGS_FILE")

    # Check if statusLine already configured
    if echo "$current_settings" | jq -e '.statusLine' &>/dev/null; then
        log_info "statusLine already configured in settings.json"
        return 0
    fi

    # Add statusLine configuration
    local statusline_config
    statusline_config=$(cat <<EOF
{
  "command": "$INSTALL_DIR/statusline.sh",
  "enabled": true,
  "updateInterval": 5000
}
EOF
)

    # Merge into settings
    local new_settings
    new_settings=$(echo "$current_settings" | jq --argjson sl "$statusline_config" '. + {statusLine: $sl}')

    echo "$new_settings" > "$SETTINGS_FILE"
    log_success "settings.json updated with statusLine configuration"
}

# ============================================================================
# VERIFICATION
# ============================================================================

verify_installation() {
    log_info "Verifying installation..."

    local errors=0

    # Check main script exists
    if [[ ! -x "$INSTALL_DIR/statusline.sh" ]]; then
        log_error "statusline.sh not found or not executable"
        ((errors++))
    fi

    # Check config exists
    if [[ ! -f "$INSTALL_DIR/Config.toml" ]]; then
        log_error "Config.toml not found"
        ((errors++))
    fi

    # Check settings.json has statusLine
    if [[ -f "$SETTINGS_FILE" ]]; then
        if ! jq -e '.statusLine' "$SETTINGS_FILE" &>/dev/null; then
            log_warn "statusLine not found in settings.json"
        fi
    fi

    if [[ $errors -eq 0 ]]; then
        log_success "Installation verified successfully!"
        return 0
    else
        log_error "Installation verification failed with $errors errors"
        return 1
    fi
}

# ============================================================================
# HELP
# ============================================================================

show_help() {
    echo "Claude Code Statusline Installer v${INSTALLER_VERSION}"
    echo ""
    echo "Usage: ./install.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help              Show this help message"
    echo "  --no-interactive    Skip interactive wizard"
    echo "  --force             Overwrite existing installation without backup"
    echo "  --profile=PROFILE   Installation profile (minimal, standard, full, developer)"
    echo "  --theme=THEME       Theme name (classic, garden, catppuccin, custom)"
    echo ""
    echo "Profiles:"
    echo "  minimal    - Basic git/model info only"
    echo "  standard   - Git, model, MCP, usage limits, cost (default)"
    echo "  full       - All features including containers and worktrees"
    echo "  developer  - Full + debug timing and cache stats"
    echo ""
    echo "Examples:"
    echo "  ./install.sh                              # Interactive installation"
    echo "  ./install.sh --profile=minimal --no-interactive"
    echo "  ./install.sh --theme=garden --profile=full"
    echo ""
    echo "One-liner install:"
    echo "  curl -fsSL https://raw.githubusercontent.com/aryateja/Lecoder-Claude-code-statusline/main/install.sh | bash"
}

# ============================================================================
# MAIN
# ============================================================================

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --help|-h)
                show_help
                exit 0
                ;;
            --no-interactive)
                INTERACTIVE=false
                shift
                ;;
            --force)
                FORCE=true
                shift
                ;;
            --profile=*)
                PROFILE="${1#*=}"
                shift
                ;;
            --theme=*)
                THEME="${1#*=}"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    print_banner

    # Detect platform
    local platform
    platform=$(detect_platform)
    log_info "Detected platform: $platform"

    if [[ "$platform" == "unsupported" ]]; then
        log_error "Unsupported platform: $(uname -s)"
        exit 1
    fi

    # Check dependencies
    if ! check_dependencies; then
        exit 1
    fi

    # Run wizard or use defaults
    if [[ "$INTERACTIVE" == true ]]; then
        run_wizard
    else
        log_info "Using profile: $PROFILE, theme: $THEME"
    fi

    # Backup existing if not forced
    if [[ "$FORCE" != true ]]; then
        backup_existing
    fi

    # Install
    install_files
    configure_theme
    configure_profile
    update_settings_json

    # Verify
    verify_installation

    # Done
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║               Installation Complete! 🎉                       ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Statusline installed to: $INSTALL_DIR"
    echo ""
    echo "To test manually:"
    echo "  echo '{\"workspace\":{\"current_dir\":\"'\$(pwd)'\"},\"model\":{\"display_name\":\"Test\"}}' | $INSTALL_DIR/statusline.sh"
    echo ""
    echo "Restart Claude Code to see your new statusline!"
}

# Run main
main "$@"
