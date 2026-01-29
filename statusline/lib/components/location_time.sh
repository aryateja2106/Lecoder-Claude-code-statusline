#!/bin/bash
# ============================================================================
# Claude Code Statusline - Location/Timezone Component
# ============================================================================
# Displays configurable timezone information
# Format: 🌍 PST 8:45PM • NYC 11:45PM
# ============================================================================

# Component data storage
COMPONENT_PRIMARY_TIME=""
COMPONENT_SECONDARY_TIME=""

# ============================================================================
# CONFIGURATION DEFAULTS
# ============================================================================

# These can be overridden via Config.toml:
# features.show_location_time = true
# location.primary_timezone = "America/Los_Angeles"
# location.primary_label = "PST"
# location.secondary_timezone = "America/New_York"
# location.secondary_label = "NYC"
# location.time_format = "12h"  # or "24h"

LOCATION_PRIMARY_TZ="${LOCATION_PRIMARY_TZ:-America/Los_Angeles}"
LOCATION_PRIMARY_LABEL="${LOCATION_PRIMARY_LABEL:-PST}"
LOCATION_SECONDARY_TZ="${LOCATION_SECONDARY_TZ:-America/New_York}"
LOCATION_SECONDARY_LABEL="${LOCATION_SECONDARY_LABEL:-NYC}"
LOCATION_TIME_FORMAT="${LOCATION_TIME_FORMAT:-12h}"

# ============================================================================
# CROSS-PLATFORM DATE HANDLING
# ============================================================================

get_time_in_timezone() {
    local timezone="$1"
    local format="$2"

    # Determine time format string
    local time_fmt
    if [[ "$format" == "24h" ]]; then
        time_fmt="%H:%M"
    else
        time_fmt="%I:%M%p"
    fi

    # Cross-platform timezone handling
    if [[ "$(uname -s)" == "Darwin" ]]; then
        # macOS
        TZ="$timezone" date +"$time_fmt" 2>/dev/null | sed 's/^0//'
    else
        # Linux
        TZ="$timezone" date +"$time_fmt" 2>/dev/null | sed 's/^0//'
    fi
}

# ============================================================================
# DATA COLLECTION
# ============================================================================

collect_location_time_data() {
    debug_log "Collecting location_time component data" "INFO" 2>/dev/null || true

    COMPONENT_PRIMARY_TIME=""
    COMPONENT_SECONDARY_TIME=""

    # Load config values if available
    if declare -f get_config &>/dev/null; then
        local show_location
        show_location=$(get_config "features.show_location_time" "false" 2>/dev/null)

        if [[ "$show_location" != "true" ]]; then
            debug_log "Location time disabled in config" "INFO" 2>/dev/null || true
            return 0
        fi

        LOCATION_PRIMARY_TZ=$(get_config "location.primary_timezone" "$LOCATION_PRIMARY_TZ" 2>/dev/null)
        LOCATION_PRIMARY_LABEL=$(get_config "location.primary_label" "$LOCATION_PRIMARY_LABEL" 2>/dev/null)
        LOCATION_SECONDARY_TZ=$(get_config "location.secondary_timezone" "$LOCATION_SECONDARY_TZ" 2>/dev/null)
        LOCATION_SECONDARY_LABEL=$(get_config "location.secondary_label" "$LOCATION_SECONDARY_LABEL" 2>/dev/null)
        LOCATION_TIME_FORMAT=$(get_config "location.time_format" "$LOCATION_TIME_FORMAT" 2>/dev/null)
    fi

    # Get times for both timezones
    COMPONENT_PRIMARY_TIME=$(get_time_in_timezone "$LOCATION_PRIMARY_TZ" "$LOCATION_TIME_FORMAT")
    COMPONENT_SECONDARY_TIME=$(get_time_in_timezone "$LOCATION_SECONDARY_TZ" "$LOCATION_TIME_FORMAT")

    debug_log "location_time: primary=${LOCATION_PRIMARY_LABEL}:${COMPONENT_PRIMARY_TIME}, secondary=${LOCATION_SECONDARY_LABEL}:${COMPONENT_SECONDARY_TIME}" "INFO" 2>/dev/null || true
}

# ============================================================================
# RENDERING
# ============================================================================

render_location_time() {
    local theme_enabled="${1:-true}"

    # Skip if no time data
    if [[ -z "$COMPONENT_PRIMARY_TIME" ]]; then
        return 1  # No content
    fi

    local emoji="🌍"
    local output="${emoji} ${LOCATION_PRIMARY_LABEL}: ${COMPONENT_PRIMARY_TIME}"

    if [[ -n "$COMPONENT_SECONDARY_TIME" ]]; then
        output="${output} • ${LOCATION_SECONDARY_LABEL}: ${COMPONENT_SECONDARY_TIME}"
    fi

    echo "$output"
}

# ============================================================================
# COMPONENT REGISTRATION
# ============================================================================

# Only register if the function exists (may not be loaded in standalone tests)
if declare -f register_component &>/dev/null; then
    register_component \
        "location_time" \
        "Configurable timezone display" \
        "" \
        "true"
fi

debug_log "Location time component loaded" "INFO" 2>/dev/null || true
