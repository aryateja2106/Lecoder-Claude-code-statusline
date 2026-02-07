use std::process::Command;

/// Docker container status.
#[derive(Debug, Default)]
pub struct ContainerInfo {
    pub containers: Vec<Container>,
}

#[derive(Debug)]
pub struct Container {
    pub name: String,
    pub status: String,
}

/// Check for running Docker containers.
/// Uses `docker ps --format` which is acceptable for infrequent calls.
pub fn collect() -> ContainerInfo {
    let mut info = ContainerInfo::default();

    let output = match Command::new("docker")
        .args(["ps", "--format", "{{.Names}}\t{{.Status}}"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return info, // Docker not installed or not running
    };

    if !output.status.success() {
        return info;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() == 2 {
            info.containers.push(Container {
                name: parts[0].to_string(),
                status: simplify_status(parts[1]),
            });
        }
    }

    info
}

fn simplify_status(status: &str) -> String {
    let lower = status.to_lowercase();
    if lower.starts_with("up") {
        "running".into()
    } else if lower.contains("exited") {
        "exited".into()
    } else if lower.contains("created") {
        "created".into()
    } else {
        status.to_string()
    }
}
