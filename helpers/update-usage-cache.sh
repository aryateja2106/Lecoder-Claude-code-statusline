#!/bin/bash
#
# Update usage cache for statusline display
#
# Usage: ./update-usage-cache.sh [session_percent] [weekly_percent] [session_reset] [weekly_reset]
#
# Example: ./update-usage-cache.sh 16 20 "3 hours" "5 days"
#
# If no arguments provided, prompts for values interactively

claude_dir="${PAI_DIR:-$HOME/.claude}"
cache_file="$claude_dir/usage-cache.json"

if [ $# -ge 2 ]; then
    session_pct="$1"
    weekly_pct="$2"
    session_reset="${3:-}"
    weekly_reset="${4:-}"
else
    echo "Update Usage Cache for Statusline"
    echo "=================================="
    echo ""
    echo "Check /usage output and enter the values:"
    echo ""
    read -p "Session usage % (e.g., 16): " session_pct
    read -p "Weekly usage % (e.g., 20): " weekly_pct
    read -p "Session resets in (e.g., 3 hours): " session_reset
    read -p "Weekly resets in (e.g., 5 days) [optional]: " weekly_reset
fi

# Get current timestamp
timestamp=$(date +%s)

# Write cache file
cat > "$cache_file" << EOF
{
  "session_percent": ${session_pct:-0},
  "weekly_percent": ${weekly_pct:-0},
  "session_reset": "${session_reset}",
  "weekly_reset": "${weekly_reset}",
  "timestamp": ${timestamp}
}
EOF

echo "Usage cache updated!"
echo "  Session: ${session_pct}% (resets ${session_reset})"
echo "  Weekly: ${weekly_pct}%"
echo ""
echo "Statusline will now show usage limits."
