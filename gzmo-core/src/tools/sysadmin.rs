use crate::config::GzmoConfig;
use crate::ecosystem_status::format_ecosystem_status;
use crate::tools::{ToolDef, ToolHandler};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sysinfo::{Disks, System};
use tracing::{info, warn};

// ============================================================================
// 1. SysMetricsTool
// ============================================================================

pub struct SysMetricsTool;

#[async_trait]
impl ToolHandler for SysMetricsTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "sys_metrics".into(),
            description: "Retrieve native system telemetry including RAM usage, active CPU load, and disk capacities without using bash. Useful for background monitoring jobs.".into(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(&self, _args: Value) -> anyhow::Result<String> {
        info!("Executing sys_metrics tool");
        let mut sys = System::new_all();
        sys.refresh_all();

        // Memory
        let total_mem_mb = sys.total_memory() / 1024 / 1024;
        let used_mem_mb = sys.used_memory() / 1024 / 1024;
        let mem_percent = (used_mem_mb as f64 / total_mem_mb as f64) * 100.0;

        // CPU Global
        sys.refresh_cpu_usage();
        let cpu_global_usage = sys.global_cpu_usage();

        // Top 5 heavy processes
        let mut processes: Vec<_> = sys.processes().values().collect();
        processes.sort_by(|a, b| {
            b.cpu_usage()
                .partial_cmp(&a.cpu_usage())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let top_processes: Vec<Value> = processes
            .iter()
            .take(5)
            .map(|p| {
                json!({
                    "pid": p.pid().as_u32(),
                    "name": p.name().to_string_lossy(),
                    "cpu_usage": format!("{:.1}%", p.cpu_usage()),
                    "mem_mb": p.memory() / 1024 / 1024
                })
            })
            .collect();

        // Disks
        let disks = Disks::new_with_refreshed_list();
        let mut disk_stats = vec![];
        for disk in &disks {
            let total_gb = disk.total_space() / 1024 / 1024 / 1024;
            let free_gb = disk.available_space() / 1024 / 1024 / 1024;
            if total_gb > 0 {
                disk_stats.push(json!({
                    "mount": disk.mount_point().display().to_string(),
                    "total_gb": total_gb,
                    "free_gb": free_gb,
                    "usage_percent": format!("{:.1}%", ((total_gb - free_gb) as f64 / total_gb as f64) * 100.0)
                }));
            }
        }

        let hardware = crate::stealth::discover_hardware_stealthily();

        let report = json!({
            "status": "success",
            "telemetry": {
                "memory_usage_percent": format!("{:.1}%", mem_percent),
                "memory_used_mb": used_mem_mb,
                "memory_total_mb": total_mem_mb,
                "cpu_global_usage": format!("{:.1}%", cpu_global_usage),
                "top_processes_by_cpu": top_processes,
                "disk_health": disk_stats,
                "hardware_fingerprint": hardware
            }
        });

        Ok(serde_json::to_string_pretty(&report)?)
    }
}

// ============================================================================
// 2. EcosystemStatusTool — agent-callable equivalent of `/status`
// ============================================================================

/// Deterministic ecosystem snapshot (same report as operator `/status`).
pub struct EcosystemStatusTool {
    pub config: GzmoConfig,
}

#[async_trait]
impl ToolHandler for EcosystemStatusTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "ecosystem_status".into(),
            description: "Grounded GZMO ecosystem snapshot: config paths, systemd units, \
                health probes, metabolism/overnight status, workflow skills. \
                Prefer this over shell probes or inventing a `/status` slash command \
                (slash commands are operator-only)."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(&self, _args: Value) -> anyhow::Result<String> {
        info!("Executing ecosystem_status tool");
        Ok(format_ecosystem_status(&self.config).await)
    }
}

// ============================================================================
// 3. SysKillTool
// ============================================================================

pub struct SysKillTool;

#[derive(Serialize, Deserialize)]
struct SysKillArgs {
    pid: u32,
    reason: String,
}

#[async_trait]
impl ToolHandler for SysKillTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "sys_kill".into(),
            description: "Terminates a runaway or dangerous process by PID. Always provide a reason for the knowledge vault.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pid": {
                        "type": "integer",
                        "description": "The process ID (PID) to terminate."
                    },
                    "reason": {
                        "type": "string",
                        "description": "Why this process is being killed."
                    }
                },
                "required": ["pid", "reason"]
            }),
        }
    }

    async fn execute(&self, args: Value) -> anyhow::Result<String> {
        let args: SysKillArgs = serde_json::from_value(args)?;
        info!(pid = args.pid, reason = %args.reason, "Executing sys_kill tool");

        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let pid = sysinfo::Pid::from_u32(args.pid);

        if let Some(process) = sys.process(pid) {
            let p_name = process.name().to_string_lossy().to_string();

            // SECURITY: Prevent killing our own agent — check PID directly, not name
            let my_pid = sysinfo::Pid::from_u32(std::process::id());
            if pid == my_pid {
                warn!(
                    "Blocked execution: Attempted to kill own process (PID {})",
                    args.pid
                );
                return Ok(serde_json::to_string(&json!({
                    "status": "error",
                    "error": "SECURITY VIOLATION: Cannot kill the GZMO agent's own process."
                }))?);
            }

            if process.kill() {
                Ok(serde_json::to_string(&json!({
                    "status": "success",
                    "message": format!("Successfully terminated process '{}' (PID {})", p_name, args.pid)
                }))?)
            } else {
                Ok(serde_json::to_string(&json!({
                    "status": "error",
                    "error": format!("Failed to terminate process '{}' (PID {})", p_name, args.pid)
                }))?)
            }
        } else {
            Ok(serde_json::to_string(&json!({
                "status": "error",
                "error": format!("Process with PID {} not found or already terminated.", args.pid)
            }))?)
        }
    }
}
