use std::process::Command;

use crate::cache::Cache;

/// MCP server with connection status.
#[derive(Debug)]
pub struct McpServer {
    pub name: String,
    pub connected: bool,
}

/// Information about MCP servers and their connection status.
#[derive(Debug, Default)]
pub struct McpInfo {
    pub servers: Vec<McpServer>,
    pub connected: usize,
    pub total: usize,
}

/// Get real-time MCP server status.
/// Uses cached `claude mcp list` output (30s TTL) to avoid slow CLI calls every refresh.
/// Falls back to reading .mcp.json config file.
pub fn collect() -> McpInfo {
    let cache = Cache::new();

    // Check cache first (30 second TTL — MCP connections rarely change)
    if let Some(cached) = cache.get("mcp_cli_output", 30) {
        if let Some(info) = parse_mcp_output(&cached) {
            return info;
        }
    }

    // Try fresh CLI call
    if let Some(info) = collect_from_cli(&cache) {
        return info;
    }
    // Fallback to config file
    collect_from_config()
}

/// Run `claude mcp list` to get real-time connected/disconnected status.
/// Caches the output for 30 seconds to avoid repeated slow CLI invocations.
fn collect_from_cli(cache: &Cache) -> Option<McpInfo> {
    let output = Command::new("claude")
        .args(["mcp", "list"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // Cache the raw output
    cache.set("mcp_cli_output", &stdout);

    parse_mcp_output(&stdout)
}

/// Parse the output of `claude mcp list` into McpInfo.
fn parse_mcp_output(stdout: &str) -> Option<McpInfo> {
    let mut info = McpInfo::default();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip header/info lines
        if trimmed.starts_with("Checking")
            || trimmed.starts_with("MCP servers")
            || trimmed.starts_with("No MCP")
            || trimmed.contains("---")
        {
            continue;
        }

        // Parse lines like:
        //   "  MCP_DOCKER  ✓ Connected"
        //   "  httpx  ✗ Disconnected"
        //   "  server_name  ❌ Error"
        // The name is the first token, status indicated by ✓/✗/❌
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        // First token that's not a status indicator is the name
        let mut name = String::new();
        let mut found_status = false;
        let mut is_connected = false;

        for part in &parts {
            if *part == "✓" || part.to_lowercase() == "connected" {
                is_connected = true;
                found_status = true;
            } else if *part == "✗" || *part == "❌"
                || part.to_lowercase() == "disconnected"
                || part.to_lowercase() == "error"
            {
                found_status = true;
            } else if name.is_empty() && !found_status {
                // Strip trailing colon if present (e.g., "MCP_DOCKER:" -> "MCP_DOCKER")
                name = part.trim_end_matches(':').to_string();
            }
        }

        if name.is_empty() {
            continue;
        }

        if is_connected {
            info.connected += 1;
        }
        info.total += 1;

        info.servers.push(McpServer {
            name,
            connected: is_connected,
        });
    }

    // Only return if we found at least something
    if info.total > 0 {
        Some(info)
    } else {
        // Try to parse the output differently - maybe it's just server names
        let mut info2 = McpInfo::default();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with("Checking")
                && !trimmed.starts_with("MCP")
                && !trimmed.starts_with("No ")
                && !trimmed.contains("---")
            {
                let name = trimmed.split_whitespace().next().unwrap_or("").to_string();
                if !name.is_empty() {
                    info2.total += 1;
                    info2.servers.push(McpServer { name, connected: false });
                }
            }
        }
        if info2.total > 0 { Some(info2) } else { None }
    }
}

/// Fallback: read ~/.claude/.mcp.json for configured server names.
fn collect_from_config() -> McpInfo {
    let mut info = McpInfo::default();

    let path = if let Some(home) = dirs::home_dir() {
        home.join(".claude/.mcp.json")
    } else {
        return info;
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return info,
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return info,
    };

    if let Some(servers) = json.get("mcpServers").and_then(|s| s.as_object()) {
        for (name, config) in servers {
            // Check if server is disabled
            let disabled = config
                .get("disabled")
                .and_then(|d| d.as_bool())
                .unwrap_or(false);

            if !disabled {
                info.total += 1;
                // We can't know connection status from config alone
                info.servers.push(McpServer {
                    name: name.clone(),
                    connected: false, // Unknown from config
                });
            }
        }
    }

    info
}
