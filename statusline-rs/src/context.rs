use crate::stdin_data::StdinData;

/// Context window usage information.
#[derive(Debug, Default)]
pub struct ContextInfo {
    pub usage_percent: Option<f64>,
    pub tokens_used: Option<u64>,
    pub tokens_max: Option<u64>,
}

/// Extract context window info from stdin data.
///
/// Priority:
/// 1. Use `used_percentage` directly if > 0
/// 2. Fall back to `current_usage` token counts (most accurate at session start)
/// 3. Fall back to `total_input_tokens` as last resort
pub fn from_stdin(data: &StdinData) -> ContextInfo {
    let mut info = ContextInfo::default();

    let ctx = match data.context_window {
        Some(ref ctx) => ctx,
        None => return info,
    };

    info.tokens_max = ctx.context_window_size;

    // Try used_percentage first (reliable after first API response)
    if let Some(pct) = ctx.used_percentage {
        if pct > 0.0 {
            info.usage_percent = Some(pct);
            if let Some(max) = ctx.context_window_size {
                if max > 0 {
                    info.tokens_used = Some((pct / 100.0 * max as f64) as u64);
                }
            }
            return info;
        }
    }

    // used_percentage is 0 or missing — calculate from current_usage
    // This handles the session-start case where system prompt, tools, skills
    // are already loaded but used_percentage hasn't been calculated yet
    if let Some(ref usage) = ctx.current_usage {
        let input = usage.input_tokens.unwrap_or(0);
        let cache_create = usage.cache_creation_input_tokens.unwrap_or(0);
        let cache_read = usage.cache_read_input_tokens.unwrap_or(0);
        let total = input + cache_create + cache_read;

        if total > 0 {
            if let Some(max) = ctx.context_window_size {
                if max > 0 {
                    info.usage_percent = Some((total as f64 / max as f64) * 100.0);
                    info.tokens_used = Some(total);
                    return info;
                }
            }
        }
    }

    // Last resort: use total_input_tokens
    if let (Some(input), Some(max)) = (ctx.total_input_tokens, ctx.context_window_size) {
        if max > 0 && input > 0 {
            info.usage_percent = Some((input as f64 / max as f64) * 100.0);
            info.tokens_used = Some(input);
        }
    }

    info
}

/// Legacy collect function for standalone mode fallback.
pub fn collect() -> ContextInfo {
    from_stdin(&StdinData::default())
}
