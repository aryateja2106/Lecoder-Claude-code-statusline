use std::collections::HashMap;
use std::process::Command;

use crate::cache::Cache;

/// Docker container status.
#[derive(Debug, Default)]
pub struct ContainerInfo {
    pub containers: Vec<Container>,
}

#[derive(Debug)]
pub struct Container {
    pub name: String,
    pub status: String,        // "running", "exited", "created", "paused"
    pub image: String,         // e.g., "alpine:3.19"
    pub cpu_percent: Option<f64>,  // e.g., 0.5 for 0.5%
    pub mem_usage: Option<String>, // e.g., "12.5MiB / 7.67GiB"
}

/// Collect information about ALL Docker containers (running + stopped).
/// Uses caching (30s TTL) to avoid slow Docker calls on every refresh.
/// For running containers, also fetches CPU and memory stats.
pub fn collect() -> ContainerInfo {
    let cache = Cache::new();

    // Check cache first (30 second TTL)
    if let Some(cached) = cache.get("docker_containers", 30) {
        if let Some(info) = parse_cached_output(&cached) {
            return info;
        }
    }

    // Cache miss — fetch fresh data
    let info = collect_fresh(&cache);
    info
}

/// Fetch fresh Docker container data and cache it.
fn collect_fresh(cache: &Cache) -> ContainerInfo {
    let mut info = ContainerInfo::default();

    // Step 1: Get ALL containers (running + stopped)
    let ps_output = match Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Names}}\t{{.Status}}\t{{.Image}}"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return info, // Docker not installed or not running
    };

    if !ps_output.status.success() {
        return info;
    }

    let stdout = String::from_utf8_lossy(&ps_output.stdout);
    let mut running_containers = Vec::new();

    // Parse container list
    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() == 3 {
            let name = parts[0].to_string();
            let raw_status = parts[1];
            let status = simplify_status(raw_status);
            let image = parts[2].to_string();

            // Track running containers for stats collection
            if status == "running" {
                running_containers.push(name.clone());
            }

            info.containers.push(Container {
                name,
                status,
                image,
                cpu_percent: None,
                mem_usage: None,
            });
        }
    }

    // Step 2: If there are running containers, fetch their stats
    if !running_containers.is_empty() {
        let stats = fetch_container_stats(&running_containers);

        // Match stats to containers by name
        for container in &mut info.containers {
            if let Some(stats) = stats.get(&container.name) {
                container.cpu_percent = stats.cpu_percent;
                container.mem_usage = stats.mem_usage.clone();
            }
        }
    }

    // Cache the result as serialized JSON-like format
    let cached_output = serialize_container_info(&info);
    cache.set("docker_containers", &cached_output);

    info
}

/// Fetch CPU and memory stats for running containers.
fn fetch_container_stats(container_names: &[String]) -> HashMap<String, ContainerStats> {
    let mut stats_map = HashMap::new();

    // Run docker stats for all running containers at once
    let stats_output = match Command::new("docker")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}",
        ])
        .output()
    {
        Ok(o) => o,
        Err(_) => return stats_map,
    };

    if !stats_output.status.success() {
        return stats_map;
    }

    let stdout = String::from_utf8_lossy(&stats_output.stdout);

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() == 3 {
            let name = parts[0].to_string();
            let cpu_str = parts[1]; // e.g., "0.50%"
            let mem_usage = parts[2].to_string(); // e.g., "12.5MiB / 7.67GiB"

            // Parse CPU percentage (remove '%' and parse as float)
            let cpu_percent = cpu_str
                .trim_end_matches('%')
                .parse::<f64>()
                .ok();

            // Only include if this is one of the containers we're tracking
            if container_names.contains(&name) {
                stats_map.insert(
                    name,
                    ContainerStats {
                        cpu_percent,
                        mem_usage: Some(mem_usage),
                    },
                );
            }
        }
    }

    stats_map
}

#[derive(Debug)]
struct ContainerStats {
    cpu_percent: Option<f64>,
    mem_usage: Option<String>,
}

/// Simplify Docker status strings into standard states.
fn simplify_status(status: &str) -> String {
    let lower = status.to_lowercase();

    if lower.starts_with("up") {
        "running".into()
    } else if lower.contains("exited") {
        "exited".into()
    } else if lower.contains("created") {
        "created".into()
    } else if lower.contains("paused") {
        "paused".into()
    } else if lower.contains("restarting") {
        "restarting".into()
    } else if lower.contains("removing") {
        "removing".into()
    } else if lower.contains("dead") {
        "dead".into()
    } else {
        // Keep original if we don't recognize it
        status.to_string()
    }
}

/// Serialize ContainerInfo to a cacheable string format.
/// Format: one line per container: name|status|image|cpu|mem
fn serialize_container_info(info: &ContainerInfo) -> String {
    let mut lines = Vec::new();

    for container in &info.containers {
        let cpu = container
            .cpu_percent
            .map(|c| c.to_string())
            .unwrap_or_else(|| "".to_string());
        let mem = container
            .mem_usage
            .clone()
            .unwrap_or_else(|| "".to_string());

        lines.push(format!(
            "{}|{}|{}|{}|{}",
            container.name,
            container.status,
            container.image,
            cpu,
            mem
        ));
    }

    lines.join("\n")
}

/// Parse cached output back into ContainerInfo.
fn parse_cached_output(cached: &str) -> Option<ContainerInfo> {
    let mut info = ContainerInfo::default();

    for line in cached.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 5 {
            let name = parts[0].to_string();
            let status = parts[1].to_string();
            let image = parts[2].to_string();
            let cpu_percent = if parts[3].is_empty() {
                None
            } else {
                parts[3].parse::<f64>().ok()
            };
            let mem_usage = if parts[4].is_empty() {
                None
            } else {
                Some(parts[4].to_string())
            };

            info.containers.push(Container {
                name,
                status,
                image,
                cpu_percent,
                mem_usage,
            });
        }
    }

    if info.containers.is_empty() {
        None
    } else {
        Some(info)
    }
}
