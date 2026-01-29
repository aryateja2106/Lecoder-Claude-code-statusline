# Claude Code Statusline

[![Version](https://img.shields.io/badge/version-3.0.0-blue.svg)](https://github.com/aryateja/Lecoder-Claude-code-statusline)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)]()

A customizable multi-line statusline for [Claude Code](https://docs.anthropic.com/en/docs/claude-code) with 20+ components, themes, and cross-platform support.

## ✨ Features

- **20+ Components** - Git status, model info, MCP servers, usage limits, cost tracking, and more
- **Usage Limits Display** - See your 5-hour and 7-day usage with reset countdowns
- **Container Stats** - Monitor Docker/Podman container CPU and memory usage
- **Git Worktrees** - Track multiple git worktrees in your project
- **4 Themes** - Classic, Garden, Catppuccin, or create your own
- **Cross-Platform** - Works on macOS and Linux (Ubuntu, Debian, etc.)
- **Easy Installation** - Interactive wizard or one-liner install

## 📸 Screenshots

```
~/Projects/my-app
🎵 Claude 3.5 Sonnet │ CC:2.1.19 │ 🕐 PST: 10:30 PM
MCP:2/2: github, filesystem │ ⏱ 5H at 03:00 (2h 30m) 6% • 7DAY Wed 10:00 PM (1%)
🐳 3 running • CPU:45% MEM:2.1G │ 🌳 feature-x [3 worktrees]
```

## 🚀 Quick Install

### One-Liner (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/aryateja/Lecoder-Claude-code-statusline/main/install.sh | bash
```

### Manual Installation

```bash
git clone https://github.com/aryateja/Lecoder-Claude-code-statusline.git
cd Lecoder-Claude-code-statusline
./install.sh
```

### Non-Interactive

```bash
./install.sh --profile=standard --theme=catppuccin --no-interactive
```

## ⚙️ Configuration

Edit `~/.claude/statusline/Config.toml` to customize:

```toml
# Theme selection
theme.name = "catppuccin"  # classic, garden, catppuccin, custom

# Feature toggles
features.show_commits = true
features.show_mcp_status = true
features.show_cost_tracking = true
features.show_reset_info = true

# Model emojis
emojis.opus = "🧠"
emojis.haiku = "⚡"
emojis.sonnet = "🎵"
```

See [docs/CONFIGURATION.md](docs/CONFIGURATION.md) for all 200+ options.

## 📦 Installation Profiles

| Profile | Features |
|---------|----------|
| `minimal` | Git status, model, directory |
| `standard` | + MCP, usage limits, cost tracking |
| `full` | + Container stats, worktrees, session info |
| `developer` | + Debug timing, cache statistics |

## 🎨 Themes

- **Classic** - Traditional terminal colors
- **Garden** - Earthy greens and browns
- **Catppuccin** - Modern pastel theme (default)
- **Custom** - Define your own colors

## 🔧 Components

| Component | Description |
|-----------|-------------|
| `directory` | Current working directory path |
| `git_status` | Branch, commits ahead/behind, dirty status |
| `model_info` | Current Claude model with emoji |
| `mcp_status` | MCP server connection status |
| `usage_limits` | 5-hour and 7-day usage percentages |
| `usage_reset` | Usage with reset countdown timers |
| `cost_tracking` | Session cost in USD |
| `container_stats` | Docker/Podman container resources |
| `git_worktrees` | Git worktree count and status |
| `location_time` | Configurable timezone display |

## 📋 Requirements

- **Bash 4.0+** (macOS: `brew install bash`, Linux: usually pre-installed)
- **jq** - JSON processor
- **git** - Version control
- **bc** - Calculator for math operations
- **curl** - For updates and installation

## 🐳 Container Support

### Docker

The statusline automatically detects Docker and shows container stats:
```
🐳 3 running • CPU:45% MEM:2.1G
```

### Dev Container

Use the included dev container config for VS Code:
```bash
# Open in VS Code with Remote-Containers extension
code .
# Then: Cmd/Ctrl+Shift+P -> "Reopen in Container"
```

## 🤝 Contributing

Contributions welcome! See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

### Adding a Component

1. Create `statusline/lib/components/your_component.sh`
2. Implement `collect_your_component_data()` and `render_your_component()`
3. Register with `register_component "your_component" "description" "cache" "true"`
4. Submit a PR!

## 📄 License

MIT License - see [LICENSE](LICENSE)

## 🙏 Acknowledgments

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) by Anthropic
- [Catppuccin](https://github.com/catppuccin) color palette
- All contributors!
