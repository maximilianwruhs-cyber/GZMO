//! TUI accessibility flags — high contrast and reduced motion.

/// Runtime flags for operator comfort (env-driven, no config file yet).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessibilityFlags {
    pub high_contrast: bool,
    pub reduced_motion: bool,
}

impl AccessibilityFlags {
    pub fn from_env() -> Self {
        Self {
            high_contrast: env_flag("GZMO_TUI_HIGH_CONTRAST"),
            reduced_motion: env_flag("GZMO_TUI_REDUCED_MOTION"),
        }
    }
}

impl Default for AccessibilityFlags {
    fn default() -> Self {
        Self::from_env()
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}
