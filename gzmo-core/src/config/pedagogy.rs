use serde::{Deserialize, Serialize};

use super::defaults::*;

// ─── Pedagogy (Agentic Teacher) ───────────────────────────────────────────

/// Default interaction mode when pedagogy orchestrator is enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PedagogyDefaultMode {
    /// Socratic mentor via internal 4-agent orchestrator.
    #[default]
    Mentor,
    /// Direct execution (legacy ops daemon behavior).
    Ops,
}

/// Settings for the Agentic Teacher stack.
#[derive(Debug, Deserialize, Clone)]
pub struct PedagogyConfig {
    #[serde(default = "default_pedagogy_enabled")]
    pub enabled: bool,

    #[serde(default)]
    pub default_mode: PedagogyDefaultMode,

    #[serde(default = "default_learner_data_dir")]
    pub learner_data_dir: String,

    #[serde(default = "default_prereq_graphs_dir")]
    pub prerequisite_graphs_dir: String,

    #[serde(default = "default_edf_log_path")]
    pub edf_log_path: String,

    #[serde(default = "default_max_hint_level")]
    pub max_hint_level: u8,

    #[serde(default = "default_solution_leakage_penalty")]
    pub solution_leakage_penalty: f64,

    /// Max tokens for internal agent calls (Diagnoser, Planner, etc.).
    #[serde(default = "default_pedagogy_internal_max_tokens")]
    pub internal_max_tokens: u32,

    /// Teaching turns between teachback checkpoints (0 = disabled).
    #[serde(default = "default_teachback_interval")]
    pub teachback_interval: u32,

    /// Active learner ID (set at CLI boot from `--learner` / `GZMO_LEARNER_ID`).
    #[serde(skip)]
    pub active_learner_id: Option<String>,

    /// Unix-socket headless mentor API (daemon + `gzmo mentor` client).
    #[serde(default = "default_mentor_api_enabled")]
    pub mentor_api_enabled: bool,

    #[serde(default = "default_mentor_socket")]
    pub mentor_socket: String,

    #[serde(default)]
    pub sandbox: SandboxConfig,

    /// Autonomous Socratic dialogue when tension drops below threshold.
    #[serde(default)]
    pub low_tension_dialogue: LowTensionDialogueConfig,

    /// Path to `gzmo_skills` (pi-mentor-discovery scripts).
    #[serde(default = "default_discovery_scripts_root")]
    pub discovery_scripts_root: String,

    /// Structured chaos_val oscillation for discovery sprints (0.9→0.5→0.9).
    #[serde(default)]
    pub tension_oscillation: TensionOscillationConfig,
}

/// One phase in a pedagogy tension oscillation cycle.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TensionOscillationStepConfig {
    pub target: f64,
    pub duration_secs: u32,
    #[serde(default)]
    pub label: String,
}

/// Pedagogy chaos_val setpoint controller (PulseLoop integration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TensionOscillationScheduleMode {
    /// CLI / inbox trigger only (v1 default).
    #[default]
    Manual,
    /// Reserved: daemon cron (not wired in v1).
    Cron,
}

/// Pedagogy chaos_val setpoint controller (PulseLoop integration).
#[derive(Debug, Deserialize, Clone)]
pub struct TensionOscillationConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub schedule_mode: TensionOscillationScheduleMode,

    /// Reserved for future daemon cron when `schedule_mode = "cron"`.
    #[serde(default)]
    pub cron_hours: Vec<u32>,

    #[serde(default = "default_tension_oscillation_spawn_discovery")]
    pub spawn_discovery_on_low: bool,

    #[serde(default = "default_tension_oscillation_low_threshold")]
    pub low_phase_threshold: f64,

    #[serde(default = "default_tension_oscillation_cooldown_secs")]
    pub cooldown_secs: u64,

    #[serde(default = "default_tension_oscillation_blend_ticks")]
    pub blend_ticks: u64,

    #[serde(default = "default_tension_oscillation_sequence")]
    pub sequence: Vec<TensionOscillationStepConfig>,
}

impl Default for TensionOscillationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            schedule_mode: TensionOscillationScheduleMode::default(),
            cron_hours: Vec::new(),
            spawn_discovery_on_low: default_tension_oscillation_spawn_discovery(),
            low_phase_threshold: default_tension_oscillation_low_threshold(),
            cooldown_secs: default_tension_oscillation_cooldown_secs(),
            blend_ticks: default_tension_oscillation_blend_ticks(),
            sequence: default_tension_oscillation_sequence(),
        }
    }
}

/// Daemon-initiated mentor turn when chaos tension is very low.
#[derive(Debug, Deserialize, Clone)]
pub struct LowTensionDialogueConfig {
    #[serde(default)]
    pub enabled: bool,

    /// Fire when tension crosses below this value (edge-triggered).
    #[serde(default = "default_low_tension_threshold")]
    pub threshold: f64,

    /// Minimum seconds between autonomous dialogue turns.
    #[serde(default = "default_low_tension_cooldown")]
    pub cooldown_secs: u64,

    /// Seed message for bare `maybe_teach` — placeholders: `{tension}`, `{tick}`, `{phase}`.
    /// Ignored when `discovery_cycle` is true.
    #[serde(default = "default_low_tension_opening")]
    pub opening_template: String,

    /// Run full pi-mentor-discovery cycle (pillar probe + cycle report) instead of bare mentor teach.
    #[serde(default = "default_low_tension_discovery_cycle")]
    pub discovery_cycle: bool,

    /// Minimum ticks of low-tension plateau to fire dialogue trigger.
    #[serde(default)]
    pub idle_ticks_threshold: Option<u64>,

    /// Discovery queue config (max pending, max concurrent, session priority).
    #[serde(default)]
    pub discovery_queue: DiscoveryQueueConfig,
}

/// Discovery queue configuration for AUTO Socratic cycles.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct DiscoveryQueueConfig {
    /// Maximum pending discovery cycles in the queue.
    #[serde(default = "default_discovery_max_pending")]
    pub max_pending: usize,

    /// Maximum concurrent discovery cycles.
    #[serde(default = "default_discovery_max_concurrent")]
    pub max_concurrent: usize,

    /// Prioritize session-bound cycles over AUTO cycles.
    #[serde(default = "default_discovery_session_priority")]
    pub session_priority: bool,
}

impl Default for LowTensionDialogueConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: default_low_tension_threshold(),
            cooldown_secs: default_low_tension_cooldown(),
            opening_template: default_low_tension_opening(),
            discovery_cycle: default_low_tension_discovery_cycle(),
            idle_ticks_threshold: None,
            discovery_queue: DiscoveryQueueConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SandboxConfig {
    #[serde(default = "default_sandbox_enabled")]
    pub enabled: bool,
    #[serde(default = "default_sandbox_max_code_chars")]
    pub max_code_chars: usize,
    #[serde(default = "default_sandbox_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_sandbox_max_output_chars")]
    pub max_output_chars: usize,
    #[serde(default = "default_sandbox_blocked_imports")]
    pub blocked_imports: Vec<String>,
    #[serde(default = "default_sandbox_orchestrator_offload")]
    pub orchestrator_offload: bool,
}

impl Default for PedagogyConfig {
    fn default() -> Self {
        Self {
            enabled: default_pedagogy_enabled(),
            default_mode: PedagogyDefaultMode::default(),
            learner_data_dir: default_learner_data_dir(),
            prerequisite_graphs_dir: default_prereq_graphs_dir(),
            edf_log_path: default_edf_log_path(),
            max_hint_level: default_max_hint_level(),
            solution_leakage_penalty: default_solution_leakage_penalty(),
            internal_max_tokens: default_pedagogy_internal_max_tokens(),
            teachback_interval: default_teachback_interval(),
            active_learner_id: None,
            mentor_api_enabled: default_mentor_api_enabled(),
            mentor_socket: default_mentor_socket(),
            sandbox: SandboxConfig::default(),
            low_tension_dialogue: LowTensionDialogueConfig::default(),
            discovery_scripts_root: default_discovery_scripts_root(),
            tension_oscillation: TensionOscillationConfig::default(),
        }
    }
}

impl PedagogyConfig {
    /// Resolve learner ID: `--learner` flag → `GZMO_LEARNER_ID` env → `"operator"`.
    pub fn resolve_learner_id(cli_flag: Option<&str>) -> String {
        if let Some(id) = cli_flag.filter(|s| !s.is_empty()) {
            return id.to_string();
        }
        if let Ok(id) = std::env::var("GZMO_LEARNER_ID") {
            if !id.is_empty() {
                return id;
            }
        }
        "operator".to_string()
    }

    pub fn learner_id(&self) -> &str {
        self.active_learner_id.as_deref().unwrap_or("operator")
    }

    pub fn learner_dir(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.learner_data_dir).join(self.learner_id())
    }

    pub fn profile_path(&self) -> std::path::PathBuf {
        self.learner_dir().join("profile.json")
    }

    pub fn session_path(&self) -> std::path::PathBuf {
        self.learner_dir().join("session.json")
    }

    pub fn episodes_dir(&self) -> std::path::PathBuf {
        self.learner_dir().join("episodes")
    }

    pub fn mentor_socket_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.mentor_socket)
    }
}
