#!/bin/bash
#
# Quick Usage Update - Fast way to update usage cache
#
# Usage: ./quick-update-usage.sh <session%> <weekly%> [reset_time]
#
# Examples:
#   ./quick-update-usage.sh 60 25 "3h42m"
#   ./quick-update-usage.sh 60 25
#
# Or just run without args after checking /usage:
#   ./quick-update-usage.sh

claude_dir="${PAI_DIR:-$HOME/.claude}"
cache_file="$claude_dir/usage-cache.json"

# If no args, prompt for quick input
if [ $# -eq 0 ]; then
    echo "Quick Usage Update"
    echo "=================="
    echo "Format: session% weekly% [reset_time]"
    echo "Example: 60 25 3h42m"
    echo ""
    read -p "> " input
    set -- $input
fi

session_pct="${1:-0}"
weekly_pct="${2:-0}"
reset_time="${3:-}"

# Parse reset time into human readable if provided
if [ -n "$reset_time" ]; then
    # Handle formats like "3h42m" or "3 hr 42 min"
    session_reset="$reset_time"
else
    session_reset=""
fi

timestamp=$(date +%s)

cat > "$cache_file" << EOF
{
  "session_percent": ${session_pct},
  "weekly_percent": ${weekly_pct},
  "session_reset": "${session_reset}",
  "weekly_reset": "",
  "timestamp": ${timestamp}
}
EOF

echo "✓ Updated: Session ${session_pct}%, Weekly ${weekly_pct}%"
