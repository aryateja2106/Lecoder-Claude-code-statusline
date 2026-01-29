#!/bin/bash
#
# Get system stats (CPU model, CPU usage, Memory) for statusline display
#
# Outputs: CPU_MODEL|CPU_PERCENT|MEMORY_USED|MEMORY_TOTAL
#

# Detect OS
OS=$(uname -s)

if [ "$OS" = "Darwin" ]; then
    # macOS

    # Get CPU model - extract clean model name
    # Clean up the brand string to be shorter
    cpu_model=$(sysctl -n machdep.cpu.brand_string | sed 's/Apple //; s/ (TM)//; s/ (R)//' | awk '{print $1, $2}')

    # Get CPU usage - use top with 1 sample for speed
    # Format: "CPU usage: 12.34% user, 5.67% sys, 81.99% idle"
    cpu_line=$(top -l 1 -n 0 | grep "CPU usage" | head -1)
    if [ -n "$cpu_line" ]; then
        # Extract user and sys percentages
        user_cpu=$(echo "$cpu_line" | awk '{print $3}' | sed 's/%//')
        sys_cpu=$(echo "$cpu_line" | awk '{print $5}' | sed 's/%//')
        # Calculate total CPU usage (user + sys)
        cpu_percent=$(echo "$user_cpu + $sys_cpu" | bc | awk '{printf "%.0f", $1}')
    else
        cpu_percent="0"
    fi

    # Get memory stats from vm_stat
    # Get actual page size (16384 on Apple Silicon, 4096 on Intel usually)
    if command -v pagesize >/dev/null; then
        page_size=$(pagesize)
    else
        page_size=4096
    fi
    
    vm_output=$(vm_stat)

    # Extract values (they include a trailing period and are in pages)
    pages_active=$(echo "$vm_output" | grep "Pages active:" | awk '{print $3}' | sed 's/\.//')
    pages_wired=$(echo "$vm_output" | grep "Pages wired down:" | awk '{print $4}' | sed 's/\.//')
    pages_compressed=$(echo "$vm_output" | grep "Pages occupied by compressor:" | awk '{print $5}' | sed 's/\.//')
    
    # Use standard "App Memory + Wired + Compressed" approximation for "Used"
    used_pages=$((pages_wired + pages_active + pages_compressed))
    used_bytes=$((used_pages * page_size))
    used_gb=$(echo "scale=1; $used_bytes / 1073741824" | bc)

    # Get total memory from sysctl (in bytes)
    total_bytes=$(sysctl -n hw.memsize)
    total_gb=$(echo "scale=0; $total_bytes / 1073741824" | bc)

elif [ "$OS" = "Linux" ]; then
    # Linux

    # Get CPU model
    cpu_model=$(grep "model name" /proc/cpuinfo | head -1 | cut -d: -f2 | xargs | awk '{print $1, $2, $3}')

    # Get CPU usage from /proc/stat
    # Calculate from two samples 100ms apart for accuracy
    read cpu_prev < <(grep '^cpu ' /proc/stat)
    sleep 0.1
    read cpu_now < <(grep '^cpu ' /proc/stat)

    prev_vals=($cpu_prev)
    now_vals=($cpu_now)

    prev_idle=$((${prev_vals[4]} + ${prev_vals[5]}))
    now_idle=$((${now_vals[4]} + ${now_vals[5]}))

    prev_total=0
    now_total=0
    for val in "${prev_vals[@]:1}"; do prev_total=$((prev_total + val)); done
    for val in "${now_vals[@]:1}"; do now_total=$((now_total + val)); done

    total_diff=$((now_total - prev_total))
    idle_diff=$((now_idle - prev_idle))

    if [ $total_diff -gt 0 ]; then
        cpu_percent=$(echo "scale=0; 100 * ($total_diff - $idle_diff) / $total_diff" | bc)
    else
        cpu_percent=0
    fi

    # Get memory stats from /proc/meminfo
    mem_total=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    mem_available=$(grep MemAvailable /proc/meminfo | awk '{print $2}')

    mem_used=$((mem_total - mem_available))
    used_gb=$(echo "scale=1; $mem_used / 1048576" | bc)
    total_gb=$(echo "scale=0; $mem_total / 1048576" | bc)
else
    # Unknown OS
    cpu_model="Unknown"
    cpu_percent="0"
    used_gb="0"
    total_gb="0"
fi

# Output format: CPU_MODEL|CPU%|USED_GB|TOTAL_GB
# Use pipe delimiter to handle multi-word CPU models
echo "${cpu_model}|${cpu_percent}|${used_gb}|${total_gb}"
