#!/bin/bash
#
# Count Skills, Workflows, and MCPs for statusline display
#
# Outputs: SKILLS_COUNT WORKFLOWS_COUNT MCPS_COUNT
#

claude_dir="${PAI_DIR:-$HOME/.claude}"

# Count Skills (directories in Skills/, excluding hidden)
skills_count=0
if [ -d "$claude_dir/Skills" ]; then
    skills_count=$(find "$claude_dir/Skills" -maxdepth 1 -type d -not -name ".*" -not -path "$claude_dir/Skills" 2>/dev/null | wc -l | tr -d ' ')
fi

# Count Workflows (all .md files in any Skills/*/workflows/ directory)
workflows_count=0
if [ -d "$claude_dir/Skills" ]; then
    workflows_count=$(find "$claude_dir/Skills" -type f -path "*/workflows/*.md" 2>/dev/null | wc -l | tr -d ' ')
fi

# Count MCPs from both settings.json and .mcp.json
mcps_count=0

# Check settings.json for .mcpServers (legacy)
if [ -f "$claude_dir/settings.json" ]; then
    settings_mcps=$(jq -r '.mcpServers | length' "$claude_dir/settings.json" 2>/dev/null)
    if [ -n "$settings_mcps" ] && [ "$settings_mcps" != "null" ]; then
        mcps_count=$((mcps_count + settings_mcps))
    fi
fi

# Check .mcp.json (current Claude Code default)
if [ -f "$claude_dir/.mcp.json" ]; then
    mcp_json_count=$(jq -r '.mcpServers | length' "$claude_dir/.mcp.json" 2>/dev/null)
    if [ -n "$mcp_json_count" ] && [ "$mcp_json_count" != "null" ]; then
        mcps_count=$((mcps_count + mcp_json_count))
    fi
fi

# Output in format: SKILLS WORKFLOWS MCPS
echo "${skills_count} ${workflows_count} ${mcps_count}"
