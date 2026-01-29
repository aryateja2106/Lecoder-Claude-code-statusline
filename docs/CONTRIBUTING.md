# Contributing Guide

Thank you for your interest in contributing to Claude Code Statusline!

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/Lecoder-Claude-code-statusline.git
   ```
3. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Bash 4.0+
- jq
- shellcheck (for linting)
- Docker (for testing)

### Install Dependencies

**macOS:**
```bash
brew install bash jq shellcheck
```

**Ubuntu/Debian:**
```bash
sudo apt install jq shellcheck
```

### Using Dev Container

If you use VS Code with the Remote-Containers extension:
1. Open the project folder
2. Press `Cmd/Ctrl+Shift+P`
3. Select "Reopen in Container"

This sets up a complete development environment automatically.

## Adding a New Component

Components live in `statusline/lib/components/`. Here's how to add one:

### 1. Create the Component File

```bash
touch statusline/lib/components/my_component.sh
chmod +x statusline/lib/components/my_component.sh
```

### 2. Implement the Component

```bash
#!/bin/bash
# ============================================================================
# Claude Code Statusline - My Component
# ============================================================================
# Brief description of what this component does
# Format: 🎯 Output format example
# ============================================================================

# Component data storage
COMPONENT_MY_DATA=""

# ============================================================================
# DATA COLLECTION
# ============================================================================

collect_my_component_data() {
    debug_log "Collecting my_component data" "INFO" 2>/dev/null || true

    # Reset data
    COMPONENT_MY_DATA=""

    # Collect your data here
    COMPONENT_MY_DATA="some value"

    debug_log "my_component: data=${COMPONENT_MY_DATA}" "INFO" 2>/dev/null || true
}

# ============================================================================
# RENDERING
# ============================================================================

render_my_component() {
    local theme_enabled="${1:-true}"

    # Return 1 if no content to display
    if [[ -z "$COMPONENT_MY_DATA" ]]; then
        return 1
    fi

    local emoji="🎯"
    echo "${emoji} ${COMPONENT_MY_DATA}"
}

# ============================================================================
# COMPONENT REGISTRATION
# ============================================================================

if declare -f register_component &>/dev/null; then
    register_component \
        "my_component" \
        "Brief description for config" \
        "cache" \  # or "" for no caching
        "true"     # enabled by default
fi

debug_log "My component loaded" "INFO" 2>/dev/null || true
```

### 3. Add Config Options (Optional)

Add to `Config.toml`:

```toml
# My Component settings
features.show_my_component = true
my_component.some_option = "value"
```

### 4. Test Your Component

```bash
# Syntax check
bash -n statusline/lib/components/my_component.sh

# Run shellcheck
shellcheck statusline/lib/components/my_component.sh

# Test in Docker
docker build -t test -f docker/Dockerfile.test .
docker run --rm test /bin/bash -c "
    ./install.sh --no-interactive && \
    echo '{\"workspace\":{\"current_dir\":\"/tmp\"},\"model\":{\"display_name\":\"Test\"}}' | \
    /home/testuser/.claude/statusline/statusline.sh
"
```

## Code Style

### Shell Scripts

- Use `#!/bin/bash` shebang
- Follow [Google's Shell Style Guide](https://google.github.io/styleguide/shellguide.html)
- Use `local` for function variables
- Quote all variables: `"$var"` not `$var`
- Use `[[ ]]` for conditionals
- Run `shellcheck` before committing

### Documentation

- Use clear, concise language
- Include code examples
- Update README if adding features

## Testing

### Unit Tests

Run syntax checks on all scripts:
```bash
find statusline -name "*.sh" -exec bash -n {} \;
```

### Integration Tests

Test the full statusline:
```bash
./install.sh --profile=developer --no-interactive
echo '{"workspace":{"current_dir":"'"$(pwd)"'"},"model":{"display_name":"Test"}}' | \
    ~/.claude/statusline/statusline.sh
```

### Cross-Platform Testing

Test in Docker for Linux compatibility:
```bash
docker build -t statusline-test -f docker/Dockerfile.test .
docker run --rm statusline-test /bin/bash -c "./install.sh --no-interactive"
```

## Pull Request Process

1. Ensure all tests pass
2. Run shellcheck with no warnings
3. Update documentation if needed
4. Create a descriptive PR title
5. Link any related issues

### PR Title Format

```
feat: Add container stats component
fix: Correct timezone parsing on Linux
docs: Update configuration reference
```

## Questions?

Open an issue or discussion on GitHub!
