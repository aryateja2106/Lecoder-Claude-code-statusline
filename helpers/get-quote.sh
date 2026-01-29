#!/bin/bash
#
# Get a random quote for statusline display
#
# Outputs: "Quote text" —Author
#
# Sources quotes from a local file or falls back to built-in quotes
#

claude_dir="${PAI_DIR:-$HOME/.claude}"
quotes_file="$claude_dir/statusline-helpers/quotes.txt"

# If quotes file exists and has content, use it
if [ -f "$quotes_file" ] && [ -s "$quotes_file" ]; then
    # Count total quotes (each quote is on one line)
    total_quotes=$(wc -l < "$quotes_file" | tr -d ' ')

    if [ "$total_quotes" -gt 0 ]; then
        # Get random line number
        random_line=$((RANDOM % total_quotes + 1))

        # Extract the quote
        quote=$(sed -n "${random_line}p" "$quotes_file")

        if [ -n "$quote" ]; then
            echo "$quote"
            exit 0
        fi
    fi
fi

# Fallback to built-in quotes if file doesn't exist or is empty
builtin_quotes=(
    "\"The very essence of instinct is that it's followed independently of reason.\" —Charles Darwin"
    "\"Code is like humor. When you have to explain it, it's bad.\" —Cory House"
    "\"First, solve the problem. Then, write the code.\" —John Johnson"
    "\"Simplicity is the soul of efficiency.\" —Austin Freeman"
    "\"Make it work, make it right, make it fast.\" —Kent Beck"
    "\"The best error message is the one that never shows up.\" —Thomas Fuchs"
    "\"Any fool can write code that a computer can understand. Good programmers write code that humans can understand.\" —Martin Fowler"
    "\"The only way to learn a new programming language is by writing programs in it.\" —Dennis Ritchie"
    "\"Premature optimization is the root of all evil.\" —Donald Knuth"
    "\"Talk is cheap. Show me the code.\" —Linus Torvalds"
)

# Get random index
total_builtin=${#builtin_quotes[@]}
random_index=$((RANDOM % total_builtin))

echo "${builtin_quotes[$random_index]}"
