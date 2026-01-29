#!/bin/bash
# ============================================================================
# Claude Code Statusline - Container Stats Component
# ============================================================================
# Displays Docker/Podman container resource usage
# Format: 🐳 3 running • CPU:45% MEM:2.1G
# ============================================================================

# Component data storage
COMPONENT_CONTAINER_COUNT=""
COMPONENT_CONTAINER_CPU=""
COMPONENT_CONTAINER_MEM=""

# Cache TTL for container stats (30 seconds)
CONTAINER_STATS_CACHE_TTL="${CONTAINER_STATS_CACHE_TTL:-30}"

# ============================================================================
# RUNTIME DETECTION
# ============================================================================

detect_container_runtime() {
    # Docker first (most common), then Podman
    if command -v docker &>/dev/null && docker info &>/dev/null 2>&1; then
        echo "docker"
    elif command -v podman &>/dev/null && podman info &>/dev/null 2>&1; then
        echo "podman"
    else
        echo ""
    fi
}

# ============================================================================
# DATA COLLECTION
# ============================================================================

collect_container_stats_data() {
    debug_log "Collecting container_stats component data" "INFO"

    COMPONENT_CONTAINER_COUNT=""
    COMPONENT_CONTAINER_CPU=""
    COMPONENT_CONTAINER_MEM=""

    local runtime
    runtime=$(detect_container_runtime)

    if [[ -z "$runtime" ]]; then
        debug_log "No container runtime detected" "INFO"
        return 0
    fi

    # Get running container count and stats
    local running_count=0
    local total_cpu=0
    local total_mem_bytes=0

    if [[ "$runtime" == "docker" ]]; then
        # Get running containers count
        running_count=$(docker ps -q 2>/dev/null | wc -l | tr -d ' ')

        if [[ "$running_count" -gt 0 ]]; then
            # Get stats (CPU% and Memory)
            while IFS=$'\t' read -r cpu mem; do
                [[ -z "$cpu" ]] && continue
                cpu_num="${cpu%\%}"
                total_cpu=$(echo "$total_cpu + $cpu_num" | bc 2>/dev/null || echo "$total_cpu")

                # Parse memory (e.g., "100MiB")
                local mem_value="${mem%%[A-Za-z]*}"
                local mem_unit="${mem##*[0-9.]}"
                local mem_bytes=0
                case "$mem_unit" in
                    GiB|GB) mem_bytes=$(echo "$mem_value * 1073741824" | bc 2>/dev/null) ;;
                    MiB|MB) mem_bytes=$(echo "$mem_value * 1048576" | bc 2>/dev/null) ;;
                    KiB|KB) mem_bytes=$(echo "$mem_value * 1024" | bc 2>/dev/null) ;;
                    *) mem_bytes=0 ;;
                esac
                total_mem_bytes=$((total_mem_bytes + ${mem_bytes:-0}))
            done < <(docker stats --no-stream --format "{{.CPUPerc}}\t{{.MemUsage}}" 2>/dev/null | cut -d'/' -f1)
        fi
    elif [[ "$runtime" == "podman" ]]; then
        running_count=$(podman ps -q 2>/dev/null | wc -l | tr -d ' ')

        if [[ "$running_count" -gt 0 ]]; then
            while IFS=$'\t' read -r cpu mem; do
                [[ -z "$cpu" ]] && continue
                cpu_num="${cpu%\%}"
                total_cpu=$(echo "$total_cpu + $cpu_num" | bc 2>/dev/null || echo "$total_cpu")
            done < <(podman stats --no-stream --format "{{.CPU}}\t{{.MemUsage}}" 2>/dev/null | cut -d'/' -f1)
        fi
    fi

    COMPONENT_CONTAINER_COUNT="$running_count"
    COMPONENT_CONTAINER_CPU=$(printf "%.0f" "$total_cpu" 2>/dev/null || echo "$total_cpu")

    # Format memory
    if [[ "$total_mem_bytes" -gt 1073741824 ]]; then
        COMPONENT_CONTAINER_MEM=$(echo "scale=1; $total_mem_bytes / 1073741824" | bc)G
    elif [[ "$total_mem_bytes" -gt 1048576 ]]; then
        COMPONENT_CONTAINER_MEM=$(echo "scale=0; $total_mem_bytes / 1048576" | bc)M
    else
        COMPONENT_CONTAINER_MEM="0M"
    fi

    debug_log "container_stats: count=${COMPONENT_CONTAINER_COUNT}, cpu=${COMPONENT_CONTAINER_CPU}%, mem=${COMPONENT_CONTAINER_MEM}" "INFO"
}

# ============================================================================
# RENDERING
# ============================================================================

render_container_stats() {
    local theme_enabled="${1:-true}"

    # Skip if no container data
    if [[ -z "$COMPONENT_CONTAINER_COUNT" || "$COMPONENT_CONTAINER_COUNT" == "0" ]]; then
        return 1  # No content
    fi

    local emoji="🐳"
    local output="${emoji} ${COMPONENT_CONTAINER_COUNT} running"

    if [[ -n "$COMPONENT_CONTAINER_CPU" && "$COMPONENT_CONTAINER_CPU" != "0" ]]; then
        output="${output} • CPU:${COMPONENT_CONTAINER_CPU}%"
    fi

    if [[ -n "$COMPONENT_CONTAINER_MEM" && "$COMPONENT_CONTAINER_MEM" != "0M" ]]; then
        output="${output} MEM:${COMPONENT_CONTAINER_MEM}"
    fi

    echo "$output"
}

# ============================================================================
# COMPONENT REGISTRATION
# ============================================================================

# Only register if the function exists (may not be loaded in standalone tests)
if declare -f register_component &>/dev/null; then
    register_component \
        "container_stats" \
        "Docker/Podman container resource usage" \
        "cache" \
        "true"
fi

debug_log "Container stats component loaded" "INFO" 2>/dev/null || true
