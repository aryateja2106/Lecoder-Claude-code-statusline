#!/bin/bash
# Capture and save statusline JSON for debugging
input=$(cat)
echo "$input" > ~/.claude/statusline-debug.json
echo "$input"
