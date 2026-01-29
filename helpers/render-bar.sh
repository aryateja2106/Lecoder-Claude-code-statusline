#!/bin/bash
#
# Render a progress bar with color based on percentage
#
# Usage: render-bar.sh PERCENT [WIDTH]
# Example: render-bar.sh 65 10
# Output: [======----] (with ANSI color codes)
#
# Color thresholds (percentage used, lower is better):
# - Green: < 40%
# - Yellow: 40-59%
# - Orange: 60-79%
# - Red: >= 80%
#

pct="${1:-0}"
width="${2:-10}"

# Validate and clamp percentage
[[ ! "$pct" =~ ^[0-9]+$ ]] && pct=0
(( pct < 0 )) && pct=0
(( pct > 100 )) && pct=100

# Calculate bar segments
filled=$((pct * width / 100))
empty=$((width - filled))

# Determine color based on percentage (lower is better)
if   (( pct >= 80 )); then
  color='\033[38;5;203m'  # coral red
elif (( pct >= 60 )); then
  color='\033[38;5;215m'  # peach/orange
elif (( pct >= 40 )); then
  color='\033[38;5;228m'  # light yellow
else
  color='\033[38;5;158m'  # mint green
fi
reset='\033[0m'

# Check if colors are disabled
if [ -n "$NO_COLOR" ] || [ "${PAI_SIMPLE_COLORS:-0}" = "1" ]; then
  color=""
  reset=""
fi

# Render the bar
printf "${color}["
printf '%*s' "$filled" '' | tr ' ' '='
printf '%*s' "$empty" '' | tr ' ' '-'
printf "]${reset}"
