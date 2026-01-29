# Configuration Reference

Complete reference for all Claude Code Statusline configuration options.

## Configuration File Location

```
~/.claude/statusline/Config.toml
```

## Theme Configuration

### Theme Selection

```toml
# Available: "classic", "garden", "catppuccin", "custom"
theme.name = "catppuccin"
```

### Custom Colors

Only used when `theme.name = "custom"`:

```toml
# Basic ANSI colors
colors.basic.red = "\\033[31m"
colors.basic.blue = "\\033[34m"
colors.basic.green = "\\033[32m"
colors.basic.yellow = "\\033[33m"
colors.basic.magenta = "\\033[35m"
colors.basic.cyan = "\\033[36m"
colors.basic.white = "\\033[37m"

# Extended 256-color palette
colors.extended.orange = "\\033[38;5;208m"
colors.extended.light_gray = "\\033[38;5;248m"
colors.extended.bright_green = "\\033[92m"
colors.extended.purple = "\\033[95m"
colors.extended.teal = "\\033[38;5;73m"
colors.extended.gold = "\\033[38;5;220m"

# Text formatting
colors.formatting.dim = "\\033[2m"
colors.formatting.italic = "\\033[3m"
colors.formatting.reset = "\\033[0m"
```

## Feature Toggles

```toml
# Core features
features.show_commits = true
features.show_version = true
features.show_submodules = true
features.hide_submodules_when_empty = true
features.show_mcp_status = true
features.show_cost_tracking = true
features.show_reset_info = true
features.show_session_info = true

# Optional features (disabled by default)
features.show_prayer_times = false
features.show_hijri_date = false
features.show_location_time = false
features.show_container_stats = false
```

## Model Emojis

```toml
emojis.opus = "🧠"
emojis.haiku = "⚡"
emojis.sonnet = "🎵"
emojis.default_model = "🤖"
emojis.clean_status = "✅"
emojis.dirty_status = "📁"
emojis.clock = "🕐"
emojis.live_block = "🔥"
```

## Timeouts

```toml
# Timeout for external commands
timeouts.mcp = "10s"
timeouts.version = "10s"
timeouts.ccusage = "10s"
timeouts.prayer = "10s"
```

## Cost Tracking

```toml
# Cost data source: "auto", "native", or "ccusage"
cost.session_source = "auto"

# Cost threshold alerts
cost.alerts.enabled = false
cost.alerts.session_warning = 1.00
cost.alerts.session_critical = 5.00
cost.alerts.daily_warning = 10.00
cost.alerts.daily_critical = 25.00
```

## Location/Timezone

```toml
# Enable timezone display
features.show_location_time = true

# Primary timezone
location.primary_timezone = "America/Los_Angeles"
location.primary_label = "PST"

# Secondary timezone
location.secondary_timezone = "America/New_York"
location.secondary_label = "NYC"

# Time format: "12h" or "24h"
location.time_format = "12h"
```

## Cache Settings

```toml
# Cache TTL in seconds
cache.mcp_ttl = 60
cache.version_ttl = 300
cache.git_ttl = 30
```

## Environment Variables

Override any config with environment variables:

```bash
# Enable debug mode
export STATUSLINE_DEBUG=true

# Override cache TTL
export CONTAINER_STATS_CACHE_TTL=30

# Force compatibility mode (no advanced features)
export STATUSLINE_COMPATIBILITY_MODE=true
```
