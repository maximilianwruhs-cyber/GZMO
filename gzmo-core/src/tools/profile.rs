//! Capability profiles that gate which tools are registered.

use std::sync::Arc;

use anyhow::{bail, Result};

use crate::config::{GzmoConfig, ToolsConfig};
use crate::memory::scratch::{ScratchScope, ScratchService};
use crate::memory::vault::SqliteVault;
use crate::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use crate::tools::jail::PathJail;
use crate::tools::memory::{MemoryRecordTool, MemorySearchTool};
use crate::tools::shell::ShellExecTool;
use crate::tools::sysadmin::{EcosystemStatusTool, SysKillTool, SysMetricsTool};
use crate::tools::web::WebSearchTool;
use crate::tools::web_browse::WebBrowseTool;
use crate::tools::ToolRegistry;

/// Tool capability profile for interactive agents and subagents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityProfile {
    /// FS read/search + memory search only (no write/shell/sysadmin).
    ReadOnly,
    /// Full coding tools: FS R/W, shell, web, memory — no process kill.
    Developer,
    /// Read + shell (tests) + memory search; no file write / kill.
    Reviewer,
    /// Developer + sysadmin kill.
    Operator,
}

impl CapabilityProfile {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "read_only" | "readonly" | "ro" => Ok(Self::ReadOnly),
            "developer" | "dev" => Ok(Self::Developer),
            "reviewer" | "review" => Ok(Self::Reviewer),
            "operator" | "ops" => Ok(Self::Operator),
            other => bail!("Unknown capability profile: {other}"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Developer => "developer",
            Self::Reviewer => "reviewer",
            Self::Operator => "operator",
        }
    }

    /// Map subagent role names to a profile.
    pub fn for_subagent_role(role: &str) -> Self {
        match role.trim().to_ascii_lowercase().as_str() {
            "architect" | "planner" | "researcher" | "read_only" => Self::ReadOnly,
            "reviewer" | "review" | "critic" => Self::Reviewer,
            "operator" | "ops" | "sysadmin" => Self::Operator,
            _ => Self::Developer,
        }
    }

    pub fn allows_file_write(self) -> bool {
        matches!(self, Self::Developer | Self::Operator)
    }

    pub fn allows_shell(self) -> bool {
        !matches!(self, Self::ReadOnly)
    }

    pub fn allows_web(self) -> bool {
        matches!(self, Self::Developer | Self::Operator | Self::Reviewer)
    }

    pub fn allows_sys_kill(self) -> bool {
        matches!(self, Self::Operator)
    }

    pub fn allows_memory_record(self) -> bool {
        matches!(self, Self::Developer | Self::Operator)
    }
}

/// Optional extras when registering interactive tools.
pub struct ToolRegisterOpts {
    pub vault: Option<Arc<SqliteVault>>,
    pub scratch: Option<Arc<ScratchService>>,
    pub scratch_scope: Option<ScratchScope>,
    pub serpapi_key: Option<String>,
    /// When set, register `activate_workflow_skill`.
    pub workflow: Option<(
        Arc<crate::workflow_skills::WorkflowSkillIndex>,
        crate::workflow_skills::SharedWorkflowSession,
    )>,
    /// When set, register `ecosystem_status` (agent-callable `/status`).
    pub gzmo_config: Option<GzmoConfig>,
}

impl Default for ToolRegisterOpts {
    fn default() -> Self {
        Self {
            vault: None,
            scratch: None,
            scratch_scope: None,
            serpapi_key: None,
            workflow: None,
            gzmo_config: None,
        }
    }
}

/// Register tools for a profile into an empty or existing registry.
pub fn register_for_profile(
    registry: &mut ToolRegistry,
    profile: CapabilityProfile,
    tools_cfg: &ToolsConfig,
    opts: ToolRegisterOpts,
) -> Result<Arc<PathJail>> {
    let jail = PathJail::from_roots(&tools_cfg.workspace_roots)?;

    registry.register(Box::new(FileReadTool {
        jail: Some(Arc::clone(&jail)),
    }));
    registry.register(Box::new(DirListTool {
        jail: Some(Arc::clone(&jail)),
    }));
    registry.register(Box::new(FileSearchTool {
        jail: Some(Arc::clone(&jail)),
    }));

    if profile.allows_file_write() {
        registry.register(Box::new(FileWriteTool {
            jail: Some(Arc::clone(&jail)),
        }));
    }

    if profile.allows_shell() {
        registry.register(Box::new(ShellExecTool {
            timeout: std::time::Duration::from_secs(30),
            cwd: None,
            read_only: matches!(profile, CapabilityProfile::Reviewer),
        }));
    }

    if profile.allows_web() {
        if let Some(key) = opts.serpapi_key.filter(|k| !k.is_empty()) {
            registry.register(Box::new(WebSearchTool::with_serpapi_key(key)));
        } else {
            registry.register(Box::new(WebSearchTool::default()));
        }
        registry.register(Box::new(WebBrowseTool::default()));
    }

    registry.register(Box::new(SysMetricsTool));
    if let Some(cfg) = opts.gzmo_config {
        registry.register(Box::new(EcosystemStatusTool { config: cfg }));
    }
    if profile.allows_sys_kill() {
        registry.register(Box::new(SysKillTool));
    }

    if let Some(v) = opts.vault {
        if profile.allows_memory_record() {
            registry.register(Box::new(MemoryRecordTool {
                vault: Arc::clone(&v),
            }));
        }
        registry.register(Box::new(MemorySearchTool {
            vault: v,
            scratch: opts.scratch,
            scope: opts.scratch_scope,
            scope_cell: None,
        }));
    }

    if let Some((index, session)) = opts.workflow {
        if index.model_can_activate && !index.is_empty() {
            registry.register(Box::new(
                crate::workflow_skills::ActivateWorkflowSkillTool { index, session },
            ));
        }
    }

    registry.set_audit(tools_cfg.audit);
    Ok(jail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ToolsConfig;

    #[test]
    fn read_only_excludes_write_and_shell() {
        let mut reg = ToolRegistry::new();
        let cfg = ToolsConfig::default();
        register_for_profile(
            &mut reg,
            CapabilityProfile::ReadOnly,
            &cfg,
            ToolRegisterOpts::default(),
        )
        .unwrap();
        assert!(reg.has_tool("file_read"));
        assert!(!reg.has_tool("file_write"));
        assert!(!reg.has_tool("shell_exec"));
        assert!(!reg.has_tool("sys_kill"));
    }

    #[test]
    fn developer_has_write_shell_not_kill() {
        let mut reg = ToolRegistry::new();
        let cfg = ToolsConfig::default();
        register_for_profile(
            &mut reg,
            CapabilityProfile::Developer,
            &cfg,
            ToolRegisterOpts::default(),
        )
        .unwrap();
        assert!(reg.has_tool("file_write"));
        assert!(reg.has_tool("shell_exec"));
        assert!(!reg.has_tool("sys_kill"));
        assert!(!reg.has_tool("ecosystem_status"));
    }

    #[test]
    fn ecosystem_status_when_config_provided() {
        let mut reg = ToolRegistry::new();
        let cfg = ToolsConfig::default();
        register_for_profile(
            &mut reg,
            CapabilityProfile::ReadOnly,
            &cfg,
            ToolRegisterOpts {
                gzmo_config: Some(GzmoConfig::default()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(reg.has_tool("ecosystem_status"));
        assert!(reg.has_tool("sys_metrics"));
    }

    #[test]
    fn role_mapping() {
        assert_eq!(
            CapabilityProfile::for_subagent_role("architect"),
            CapabilityProfile::ReadOnly
        );
        assert_eq!(
            CapabilityProfile::for_subagent_role("reviewer"),
            CapabilityProfile::Reviewer
        );
        assert_eq!(
            CapabilityProfile::for_subagent_role("developer"),
            CapabilityProfile::Developer
        );
    }
}
