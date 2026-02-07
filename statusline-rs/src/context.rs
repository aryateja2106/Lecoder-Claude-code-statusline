use crate::stdin_data::StdinData;

/// Context window usage information.
#[derive(Debug, Default)]
pub struct ContextInfo {
    pub usage_percent: Option<f64>,
    pub tokens_used: Option<u64>,
    pub tokens_max: Option<u64>,
}

/// Extract context window info from stdin data.
pub fn from_stdin(data: &StdinData) -> ContextInfo {
    let mut info = ContextInfo::default();

    if let Some(ref ctx) = data.context_window {
        info.usage_percent = ctx.used_percentage;
        info.tokens_max = ctx.context_window_size;

        // Calculate approximate used tokens from percentage
        if let (Some(pct), Some(max)) = (ctx.used_percentage, ctx.context_window_size) {
            if max > 0 {
                info.tokens_used = Some((pct / 100.0 * max as f64) as u64);
            }
        }

        // If no percentage but we have raw tokens, calculate it
        if info.usage_percent.is_none() {
            if let (Some(input), Some(max)) = (ctx.total_input_tokens, ctx.context_window_size) {
                if max > 0 {
                    info.usage_percent = Some((input as f64 / max as f64) * 100.0);
                    info.tokens_used = Some(input);
                }
            }
        }
    }

    info
}

/// Legacy collect function for standalone mode fallback.
pub fn collect() -> ContextInfo {
    from_stdin(&StdinData::default())
}
