/// ANSI color theme definitions for the statusline.

#[derive(Debug, Clone)]
pub struct Theme {
    pub red: &'static str,
    pub blue: &'static str,
    pub green: &'static str,
    pub yellow: &'static str,
    pub magenta: &'static str,
    pub cyan: &'static str,
    pub white: &'static str,
    pub orange: &'static str,
    pub light_gray: &'static str,
    pub bright_green: &'static str,
    pub purple: &'static str,
    pub teal: &'static str,
    pub gold: &'static str,
    pub pink: &'static str,
    pub dim: &'static str,
    pub italic: &'static str,
    pub bold: &'static str,
    pub reset: &'static str,
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        match name {
            "classic" => Self::classic(),
            "garden" => Self::garden(),
            "catppuccin" | _ => Self::catppuccin(),
        }
    }

    /// Catppuccin Mocha theme (default).
    pub fn catppuccin() -> Self {
        Self {
            red: "\x1b[38;2;243;139;168m",
            blue: "\x1b[38;2;137;180;250m",
            green: "\x1b[38;2;166;227;161m",
            yellow: "\x1b[38;2;249;226;175m",
            magenta: "\x1b[38;2;203;166;247m",
            cyan: "\x1b[38;2;137;220;235m",
            white: "\x1b[38;2;205;214;244m",
            orange: "\x1b[38;2;250;179;135m",
            light_gray: "\x1b[38;2;166;173;200m",
            bright_green: "\x1b[38;2;166;227;161m",
            purple: "\x1b[38;2;203;166;247m",
            teal: "\x1b[38;2;148;226;213m",
            gold: "\x1b[38;2;249;226;175m",
            pink: "\x1b[38;2;245;194;231m",
            dim: "\x1b[2m",
            italic: "\x1b[3m",
            bold: "\x1b[1m",
            reset: "\x1b[0m",
        }
    }

    /// Classic theme (basic ANSI 256 colors).
    pub fn classic() -> Self {
        Self {
            red: "\x1b[31m",
            blue: "\x1b[34m",
            green: "\x1b[32m",
            yellow: "\x1b[33m",
            magenta: "\x1b[35m",
            cyan: "\x1b[36m",
            white: "\x1b[37m",
            orange: "\x1b[38;5;208m",
            light_gray: "\x1b[38;5;248m",
            bright_green: "\x1b[92m",
            purple: "\x1b[95m",
            teal: "\x1b[38;5;73m",
            gold: "\x1b[38;5;220m",
            pink: "\x1b[38;5;205m",
            dim: "\x1b[2m",
            italic: "\x1b[3m",
            bold: "\x1b[1m",
            reset: "\x1b[0m",
        }
    }

    /// Garden theme (soft pastels).
    pub fn garden() -> Self {
        Self {
            red: "\x1b[38;2;255;182;193m",
            blue: "\x1b[38;2;173;216;230m",
            green: "\x1b[38;2;176;196;145m",
            yellow: "\x1b[38;2;255;218;185m",
            magenta: "\x1b[38;2;230;230;250m",
            cyan: "\x1b[38;2;175;238;238m",
            white: "\x1b[38;2;245;245;245m",
            orange: "\x1b[38;2;255;200;173m",
            light_gray: "\x1b[38;2;169;169;169m",
            bright_green: "\x1b[38;2;189;252;201m",
            purple: "\x1b[38;2;230;230;250m",
            teal: "\x1b[38;2;189;252;201m",
            gold: "\x1b[38;2;255;218;185m",
            pink: "\x1b[38;2;255;182;193m",
            dim: "\x1b[2m",
            italic: "\x1b[3m",
            bold: "\x1b[1m",
            reset: "\x1b[0m",
        }
    }
}
