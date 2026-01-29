# Cross-Platform Compatibility

This document covers platform-specific considerations for macOS and Linux.

## Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| macOS (Apple Silicon) | ✅ Full support | Requires Homebrew bash 4+ |
| macOS (Intel) | ✅ Full support | Requires Homebrew bash 4+ |
| Ubuntu 20.04+ | ✅ Full support | |
| Debian 11+ | ✅ Full support | |
| Fedora | ✅ Full support | |
| Arch Linux | ✅ Full support | |
| Alpine Linux | ⚠️ Partial | Missing some GNU tools |
| Windows WSL2 | ✅ Full support | Via Ubuntu/Debian |

## Bash Version Requirements

The statusline requires Bash 4.0+ for associative arrays and advanced features.

### macOS

macOS ships with Bash 3.x due to licensing. Install a modern version:

```bash
brew install bash
```

The statusline automatically detects and uses Homebrew bash at:
- `/opt/homebrew/bin/bash` (Apple Silicon)
- `/usr/local/bin/bash` (Intel)

### Linux

Most Linux distributions include Bash 4.0+ by default:
```bash
bash --version
# Should show 4.0 or higher
```

## Platform-Specific Commands

### Date Handling

The statusline uses different date parsing for BSD (macOS) and GNU (Linux):

**macOS (BSD date):**
```bash
date -j -f "%Y-%m-%dT%H:%M:%S%z" "$timestamp" "+%s"
```

**Linux (GNU date):**
```bash
date -d "$timestamp" "+%s"
```

The statusline handles this automatically via wrapper functions.

### File Modification Time

**macOS:**
```bash
stat -f %m "$file"
```

**Linux:**
```bash
stat -c %Y "$file"
```

### Keychain/Secret Storage

**macOS (Keychain):**
```bash
security find-generic-password -s "Claude Code-credentials" -w
```

**Linux (GNOME Keyring):**
```bash
secret-tool lookup service "Claude Code-credentials"
```

**Linux (without secret-tool):**
The statusline gracefully degrades when credentials aren't available.

## Container Runtime Detection

The statusline auto-detects container runtimes:

1. **Docker** - Checked first (most common)
2. **Podman** - Fallback if Docker not available

```bash
# Docker check
docker info &>/dev/null

# Podman check
podman info &>/dev/null
```

## Known Platform Differences

### Alpine Linux

Alpine uses `musl` instead of `glibc`, which may cause issues with some GNU tools. For full compatibility:

```dockerfile
RUN apk add --no-cache bash coreutils jq bc git curl
```

### WSL2

WSL2 runs a full Linux kernel and has full compatibility. Ensure you're using WSL2, not WSL1:

```powershell
wsl --set-version <distro-name> 2
```

## Testing Cross-Platform

### Docker Test Matrix

```bash
# Ubuntu
docker run --rm -v $(pwd):/app ubuntu:22.04 bash -c "cd /app && ./install.sh --no-interactive"

# Debian
docker run --rm -v $(pwd):/app debian:12 bash -c "cd /app && ./install.sh --no-interactive"

# Fedora
docker run --rm -v $(pwd):/app fedora:latest bash -c "cd /app && ./install.sh --no-interactive"

# Alpine (may have issues)
docker run --rm -v $(pwd):/app alpine:latest sh -c "apk add bash && cd /app && ./install.sh --no-interactive"
```

## Troubleshooting

### "Bash version too old"

Install Bash 4+:
- macOS: `brew install bash`
- Linux: `sudo apt install bash`

### "command not found: jq"

Install jq:
- macOS: `brew install jq`
- Linux: `sudo apt install jq`

### Usage limits not showing

This requires OAuth credentials stored in the system keychain:
- macOS: Automatic if logged in via Claude Code
- Linux: Requires `secret-tool` and GNOME Keyring

### Container stats not showing

Ensure Docker/Podman is running and accessible:
```bash
docker ps  # Should work without sudo
```

If permission denied, add user to docker group:
```bash
sudo usermod -aG docker $USER
# Then log out and back in
```
