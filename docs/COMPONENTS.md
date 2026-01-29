# Component Reference

Detailed documentation for all available statusline components.

## Core Components

### directory

Displays the current working directory with path truncation.

**Output:** `~/Projects/my-app`

**Config:**
```toml
# No specific config - always enabled
```

---

### git_status

Shows git branch, commits ahead/behind, and dirty status.

**Output:** `main ↑2 ↓1 📁`

**Config:**
```toml
features.show_commits = true
emojis.clean_status = "✅"
emojis.dirty_status = "📁"
```

---

### model_info

Displays the current Claude model with emoji.

**Output:** `🎵 Claude 3.5 Sonnet`

**Config:**
```toml
emojis.opus = "🧠"
emojis.haiku = "⚡"
emojis.sonnet = "🎵"
emojis.default_model = "🤖"
```

---

### mcp_status

Shows MCP server connection status.

**Output:** `MCP:2/2: github, filesystem`

**Config:**
```toml
features.show_mcp_status = true
timeouts.mcp = "10s"
```

---

### usage_limits

Displays 5-hour and 7-day usage percentages.

**Output:** `Limit: 5h:6% • 7d:1%`

**Config:**
```toml
features.show_reset_info = true  # Set to false for just percentages
```

---

### usage_reset

Extended usage display with reset countdown timers.

**Output:** `⏱ 5H at 03:00 (2h 30m) 6% • 7DAY Wed 10:00 PM (1%)`

**Config:**
```toml
features.show_reset_info = true
```

---

### cost_tracking

Shows session cost in USD.

**Output:** `$0.05`

**Config:**
```toml
features.show_cost_tracking = true
cost.session_source = "auto"  # "auto", "native", or "ccusage"

# Optional alerts
cost.alerts.enabled = false
cost.alerts.session_warning = 1.00
cost.alerts.session_critical = 5.00
```

---

## Additional Components

### container_stats

Monitors Docker/Podman container resource usage.

**Output:** `🐳 3 running • CPU:45% MEM:2.1G`

**Config:**
```toml
features.show_container_stats = true
```

**Requirements:**
- Docker or Podman installed and running
- User has permission to run docker/podman commands

---

### git_worktrees

Tracks git worktree count and current worktree name.

**Output:** `🌳 feature-x [3 worktrees]`

**Config:**
```toml
# No specific config - shows automatically when worktrees exist
```

**Behavior:**
- Hidden when only main worktree exists
- Shows worktree name when in a linked worktree
- Shows "main" when in the main worktree with linked worktrees

---

### location_time

Configurable timezone display for multiple locations.

**Output:** `🌍 PST: 10:30 PM • NYC: 1:30 AM`

**Config:**
```toml
features.show_location_time = true
location.primary_timezone = "America/Los_Angeles"
location.primary_label = "PST"
location.secondary_timezone = "America/New_York"
location.secondary_label = "NYC"
location.time_format = "12h"  # or "24h"
```

---

### version_info

Shows Claude Code version.

**Output:** `CC:2.1.19`

**Config:**
```toml
features.show_version = true
timeouts.version = "10s"
```

---

### submodule_status

Displays git submodule status.

**Output:** `SUB:3/3 ✓`

**Config:**
```toml
features.show_submodules = true
features.hide_submodules_when_empty = true
```

---

## Component Architecture

### Data Flow

```
1. main() calls collect_all_component_data()
2. Each component's collect_*_data() function runs
3. Data stored in COMPONENT_* global variables
4. render_all_components() calls each render_*() function
5. Components return 1 to hide, or echo output
```

### Creating Custom Components

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed instructions on adding new components.

### Component Registration

Components register themselves at load time:

```bash
register_component \
    "component_name" \      # Unique identifier
    "Description" \         # Shown in debug output
    "cache" \               # Cache strategy: "cache" or ""
    "true"                  # Enabled by default: "true" or "false"
```

### Caching

Components can specify caching behavior:
- `"cache"` - Component output is cached
- `""` - Component runs fresh each time

Cache TTL is controlled via config:
```toml
cache.component_name_ttl = 60  # seconds
```
