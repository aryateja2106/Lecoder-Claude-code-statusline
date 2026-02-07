use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Simple file-based cache with TTL.
pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new() -> Self {
        let dir = if let Some(home) = dirs::home_dir() {
            home.join(".cache/statusline-rs")
        } else {
            PathBuf::from("/tmp/statusline-rs-cache")
        };
        // Create the cache directory if it doesn't exist
        let _ = std::fs::create_dir_all(&dir);
        Self { dir }
    }

    /// Get a cached value if it exists and hasn't expired.
    pub fn get(&self, key: &str, ttl_secs: u64) -> Option<String> {
        let path = self.key_path(key);
        let meta = std::fs::metadata(&path).ok()?;
        let modified = meta.modified().ok()?;
        let age = SystemTime::now().duration_since(modified).ok()?;

        if age > Duration::from_secs(ttl_secs) {
            // Expired — remove stale file
            let _ = std::fs::remove_file(&path);
            return None;
        }

        std::fs::read_to_string(&path).ok()
    }

    /// Store a value in the cache.
    pub fn set(&self, key: &str, value: &str) {
        let path = self.key_path(key);
        let _ = std::fs::write(&path, value);
    }

    fn key_path(&self, key: &str) -> PathBuf {
        // Sanitize the key to be filesystem-safe
        let safe_key: String = key
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        self.dir.join(safe_key)
    }
}
