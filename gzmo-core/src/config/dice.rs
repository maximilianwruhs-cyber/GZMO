use std::collections::HashMap;

use serde::Deserialize;

use super::defaults::*;

/// `/dice` runtime configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct DiceConfig {
    #[serde(default)]
    pub r#loop: DiceLoopConfig,
    #[serde(default)]
    pub cascade: DiceCascadeConfig,
}

impl Default for DiceConfig {
    fn default() -> Self {
        Self {
            r#loop: DiceLoopConfig::default(),
            cascade: DiceCascadeConfig::default(),
        }
    }
}

/// Wild-magic cascade settings.
#[derive(Debug, Deserialize, Clone)]
pub struct DiceCascadeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Extra skills excluded beyond the embedded table exclusions.
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Default for DiceCascadeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            exclude: Vec::new(),
        }
    }
}

/// Optional lab dice-loop scheduling settings. Daemon firing is intentionally deferred.
#[derive(Debug, Deserialize, Clone)]
pub struct DiceLoopConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_dice_loop_min")]
    pub min_minutes: u32,
    #[serde(default = "default_dice_loop_max")]
    pub max_minutes: u32,
    /// Maximum automatic follow-up depth; 0 permits unlimited chaining.
    #[serde(default)]
    pub max_chain_depth: u32,
    /// Cancel the pending loop when a natural 1 is rolled.
    #[serde(default = "default_true")]
    pub cancel_on_nat_1: bool,
}

impl Default for DiceLoopConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_minutes: default_dice_loop_min(),
            max_minutes: default_dice_loop_max(),
            max_chain_depth: 0,
            cancel_on_nat_1: default_true(),
        }
    }
}

/// Custom / wizard-managed cron jobs (app-level, not host crontab).
#[derive(Debug, Deserialize, Clone, Default)]
pub struct CronConfig {
    /// Named custom jobs: `[cron.jobs.<id>]`
    #[serde(default)]
    pub jobs: HashMap<String, CustomCronJob>,
}

/// Kind of custom cron job payload.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CustomCronKind {
    #[default]
    Shell,
    Prompt,
}

/// One operator-defined job under `[cron.jobs.<id>]`.
#[derive(Debug, Deserialize, Clone)]
pub struct CustomCronJob {
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Classic 5-field cron: `min hour dom month dow` (UTC).
    pub schedule: String,

    #[serde(default)]
    pub kind: CustomCronKind,

    /// Shell command when `kind = shell`.
    #[serde(default)]
    pub command: String,

    /// Agent prompt when `kind = prompt`.
    #[serde(default)]
    pub prompt: String,

    #[serde(default)]
    pub description: String,
}
