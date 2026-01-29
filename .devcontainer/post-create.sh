#!/bin/bash
# ============================================================================
# Dev Container Post-Create Script
# ============================================================================
# Runs after the dev container is created to set up the development environment.
# ============================================================================

set -e

echo "🔧 Setting up Claude Code Statusline development environment..."

# Install system dependencies
echo "📦 Installing dependencies..."
sudo apt-get update
sudo apt-get install -y jq bc shellcheck

# Install the statusline for testing
echo "📥 Installing statusline..."
./install.sh --profile=developer --no-interactive

# Install npm packages for linting (if package.json exists)
if [[ -f "package.json" ]]; then
    echo "📦 Installing npm packages..."
    npm install
fi

# Set up git hooks for development
echo "🔗 Setting up git hooks..."
if [[ -d ".git" ]]; then
    # Pre-commit hook for shellcheck
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Run shellcheck on changed shell files
changed_files=$(git diff --cached --name-only --diff-filter=ACM | grep '\.sh$' || true)
if [[ -n "$changed_files" ]]; then
    echo "Running shellcheck..."
    shellcheck $changed_files
fi
EOF
    chmod +x .git/hooks/pre-commit
fi

echo ""
echo "✅ Development environment ready!"
echo ""
echo "Useful commands:"
echo "  ./install.sh --help          - Show installer options"
echo "  shellcheck statusline/*.sh   - Lint shell scripts"
echo "  docker build -t test -f docker/Dockerfile.test . - Build test image"
echo ""
