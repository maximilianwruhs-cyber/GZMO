//! `gzmo cron` — list / preview / edit / wizard for serve builtins + custom jobs.

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::Utc;

use gzmo_core::agent_loop::{run_agent_loop, AgentLoopConfig};
use gzmo_core::config::{CustomCronJob, CustomCronKind, GzmoConfig, TaskKind};
use gzmo_core::cron::{
    self, persist_builtin_enabled, persist_builtin_schedule, persist_custom_job, remove_custom_job,
    validate_custom, CronJobSource,
};
use gzmo_core::gateway::{GatewayRouter, LlmGateway};
use gzmo_core::identity::IdentityEngine;
use gzmo_core::tools::profile::{register_for_profile, CapabilityProfile, ToolRegisterOpts};
use gzmo_core::tools::ToolRegistry;
use gzmo_core::types::{Message, Role};

use crate::{distill_cmd, dream_cmd, embed_cmd, promote_cmd, spark_cmd};

pub async fn run(config: &GzmoConfig, identity: &IdentityEngine, args: &[String]) -> Result<()> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("wizard");
    match sub {
        "list" | "ls" => cmd_list(config),
        "preview" => {
            let id = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Usage: gzmo cron preview <id> [n]"))?;
            let n = args
                .get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(5);
            cmd_preview(config, id, n)
        }
        "set" => {
            // gzmo cron set <id> HH:MM
            let id = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Usage: gzmo cron set <builtin-id> HH:MM"))?;
            let time = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Usage: gzmo cron set <builtin-id> HH:MM"))?;
            cmd_set_builtin(config, id, time)
        }
        "enable" => {
            let id = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Usage: gzmo cron enable <id>"))?;
            cmd_enable(config, id, true)
        }
        "disable" => {
            let id = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Usage: gzmo cron disable <id>"))?;
            cmd_enable(config, id, false)
        }
        "add" => cmd_add(config, &args[1..]),
        "remove" | "rm" => {
            let id = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Usage: gzmo cron remove <custom-id>"))?;
            cmd_remove(config, id)
        }
        "run" => {
            let id = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Usage: gzmo cron run <id>"))?;
            cmd_run(config, identity, id).await
        }
        "wizard" | "ui" => cmd_wizard(config, identity).await,
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => bail!("Unknown cron subcommand '{other}'. Try: gzmo cron help"),
    }
}

fn print_help() {
    println!(
        "\
gzmo cron — manage overnight + custom jobs (app-level; not host crontab)

  gzmo cron                  Interactive wizard
  gzmo cron list             List builtin + custom jobs
  gzmo cron preview <id> [n] Next n UTC fire times
  gzmo cron set <id> HH:MM   Set builtin daily time (dream/distill/promote/embed/wiki_push)
  gzmo cron enable <id>      Enable job
  gzmo cron disable <id>     Disable job
  gzmo cron add ...          Add custom job (see wizard, or flags below)
  gzmo cron remove <id>      Remove custom job
  gzmo cron run <id>         Run job once now

Custom add (non-interactive):
  gzmo cron add --id NAME --schedule \"M H * * *\" --shell \"command\" [--desc TEXT]
  gzmo cron add --id NAME --schedule \"M H * * *\" --prompt \"...\" [--desc TEXT]

After edits: restart `gzmo serve` / `systemctl --user restart gzmo-serve`."
    );
}

fn config_path() -> PathBuf {
    std::env::var("GZMO_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("gzmo.toml")
        })
}

fn reload() -> Result<GzmoConfig> {
    GzmoConfig::load_auto().context("reload config")
}

fn cmd_list(config: &GzmoConfig) -> Result<()> {
    let jobs = cron::list_jobs(config);
    println!("{}", cron::format_job_table(&jobs));
    println!();
    println!(
        "{} job(s). Custom jobs live under [cron.jobs.*] and run inside `gzmo serve`.",
        jobs.len()
    );
    Ok(())
}

fn cmd_preview(config: &GzmoConfig, id: &str, n: usize) -> Result<()> {
    let runs = cron::preview_job(config, id, n, Utc::now())?;
    if runs.is_empty() {
        println!("No upcoming runs found for '{id}' within scan window.");
        return Ok(());
    }
    println!("Next {n} run(s) for '{id}' (UTC):");
    for (i, t) in runs.iter().enumerate() {
        println!("  {}. {}", i + 1, t.format("%Y-%m-%d %H:%M"));
    }
    Ok(())
}

fn cmd_set_builtin(config: &GzmoConfig, id: &str, time: &str) -> Result<()> {
    let _ = config;
    let (h, m) = parse_hhmm(time)?;
    let path = config_path();
    persist_builtin_schedule(&path, id, h, m)?;
    println!("Updated {id} → {h:02}:{m:02} UTC in {}", path.display());
    println!("Restart gzmo serve for the new schedule to apply.");
    Ok(())
}

fn parse_hhmm(s: &str) -> Result<(u32, u32)> {
    let (h, m) = s
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("Expected HH:MM, got {s}"))?;
    let hour: u32 = h.parse().context("hour")?;
    let minute: u32 = m.parse().context("minute")?;
    if hour > 23 || minute > 59 {
        bail!("Invalid time {s}");
    }
    Ok((hour, minute))
}

fn cmd_enable(config: &GzmoConfig, id: &str, enabled: bool) -> Result<()> {
    let path = config_path();
    let jobs = cron::list_jobs(config);
    let view = jobs
        .iter()
        .find(|j| j.id == id)
        .ok_or_else(|| anyhow::anyhow!("Unknown job '{id}'"))?;
    match view.source {
        CronJobSource::Builtin => {
            persist_builtin_enabled(&path, id, enabled)?;
        }
        CronJobSource::Custom => {
            let mut job = config
                .cron
                .jobs
                .get(id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Custom job '{id}' missing"))?;
            job.enabled = enabled;
            persist_custom_job(&path, id, &job)?;
        }
    }
    println!(
        "{} job '{id}' in {}",
        if enabled { "Enabled" } else { "Disabled" },
        path.display()
    );
    println!("Restart gzmo serve for changes to apply.");
    Ok(())
}

fn cmd_add(config: &GzmoConfig, args: &[String]) -> Result<()> {
    let _ = config;
    let mut id = None;
    let mut schedule = None;
    let mut shell = None;
    let mut prompt = None;
    let mut desc = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                id = Some(args.get(i + 1).cloned().context("--id needs value")?);
                i += 2;
            }
            "--schedule" | "-s" => {
                schedule = Some(args.get(i + 1).cloned().context("--schedule needs value")?);
                i += 2;
            }
            "--shell" => {
                shell = Some(args.get(i + 1).cloned().context("--shell needs value")?);
                i += 2;
            }
            "--prompt" => {
                prompt = Some(args.get(i + 1).cloned().context("--prompt needs value")?);
                i += 2;
            }
            "--desc" | "--description" => {
                desc = args.get(i + 1).cloned().context("--desc needs value")?;
                i += 2;
            }
            other => bail!("Unknown flag '{other}'"),
        }
    }
    let id = id.ok_or_else(|| anyhow::anyhow!("--id required"))?;
    let schedule = schedule.ok_or_else(|| anyhow::anyhow!("--schedule required"))?;
    if cron::BUILTIN_IDS.contains(&id.as_str()) {
        bail!("'{id}' is a reserved builtin id");
    }
    let job = if let Some(command) = shell {
        CustomCronJob {
            enabled: true,
            schedule,
            kind: CustomCronKind::Shell,
            command,
            prompt: String::new(),
            description: desc,
        }
    } else if let Some(prompt) = prompt {
        CustomCronJob {
            enabled: true,
            schedule,
            kind: CustomCronKind::Prompt,
            command: String::new(),
            prompt,
            description: desc,
        }
    } else {
        bail!("Provide --shell or --prompt");
    };
    validate_custom(&job)?;
    let path = config_path();
    persist_custom_job(&path, &id, &job)?;
    println!("Added custom job '{id}' → {}", path.display());
    println!("Restart gzmo serve for it to schedule.");
    Ok(())
}

fn cmd_remove(config: &GzmoConfig, id: &str) -> Result<()> {
    if cron::BUILTIN_IDS.contains(&id) {
        bail!("Cannot remove builtin '{id}' — use disable instead");
    }
    if !config.cron.jobs.contains_key(id) {
        bail!("Unknown custom job '{id}'");
    }
    remove_custom_job(&config_path(), id)?;
    println!("Removed custom job '{id}'");
    Ok(())
}

async fn cmd_run(config: &GzmoConfig, identity: &IdentityEngine, id: &str) -> Result<()> {
    println!("Running '{id}' once…");
    match id {
        "dream" => dream_cmd::run(config, identity, None).await?,
        "distill" => distill_cmd::run(config, identity, None).await?,
        "promote" => promote_cmd::run(config, None).await?,
        "embed" => embed_cmd::run(config, None).await?,
        "spark" => spark_cmd::run(config, identity, None).await?,
        "wiki_push" => {
            // Soft satellite — reuse serve helper via wiki push CLI if present
            crate::wiki_cmd::run(config, vec!["push".into()]).await?;
        }
        other => {
            let job = config
                .cron
                .jobs
                .get(other)
                .ok_or_else(|| anyhow::anyhow!("Unknown job '{other}'"))?
                .clone();
            run_custom_job(config, identity, other, &job).await?;
        }
    }
    println!("Done.");
    Ok(())
}

pub async fn run_custom_job(
    config: &GzmoConfig,
    identity: &IdentityEngine,
    id: &str,
    job: &CustomCronJob,
) -> Result<()> {
    match job.kind {
        CustomCronKind::Shell => {
            info_shell(id, &job.command);
            let mut cmd = tokio::process::Command::new("/bin/sh");
            cmd.arg("-c")
                .arg(&job.command)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            let out = tokio::time::timeout(Duration::from_secs(300), cmd.output())
                .await
                .context("custom shell job timed out (300s)")?
                .context("spawn shell")?;
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stdout.trim().is_empty() {
                println!("{stdout}");
            }
            if !stderr.trim().is_empty() {
                eprintln!("{stderr}");
            }
            if !out.status.success() {
                bail!(
                    "custom job '{id}' exited {}",
                    out.status.code().unwrap_or(-1)
                );
            }
            Ok(())
        }
        CustomCronKind::Prompt => {
            run_prompt_job(config, identity, id, &job.prompt).await
        }
    }
}

fn info_shell(id: &str, command: &str) {
    eprintln!("⚙ custom/{id}: {command}");
}

async fn run_prompt_job(
    config: &GzmoConfig,
    identity: &IdentityEngine,
    id: &str,
    prompt: &str,
) -> Result<()> {
    let soul = identity.snapshot().await;
    let router = GatewayRouter::new(config);
    let gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::Chat));
    let profile = CapabilityProfile::parse(&config.tools.profile).unwrap_or(CapabilityProfile::Developer);
    let mut tools = ToolRegistry::new();
    register_for_profile(
        &mut tools,
        profile,
        &config.tools,
        ToolRegisterOpts {
            gzmo_config: Some(config.clone()),
            ..Default::default()
        },
    )?;
    let system = format!(
        "{}\n\n---\nYou are running as scheduled cron job '{id}'. \
         Complete the task. Be concise. Do not ask the operator questions.",
        soul.raw_markdown
    );
    let mut messages = vec![
        Message {
            role: Role::System,
            content: system,
            is_meta: true,
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: prompt.to_string(),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        },
    ];
    let loop_config = AgentLoopConfig {
        max_iterations: config.agent.max_tool_iterations.min(20),
        verbose_tool_output: false,
        ..AgentLoopConfig::default()
    };
    let response = run_agent_loop(gateway.as_ref(), &tools, &mut messages, &loop_config).await?;
    println!("{}", response.text);
    Ok(())
}

async fn cmd_wizard(config: &GzmoConfig, identity: &IdentityEngine) -> Result<()> {
    let mut config = config.clone();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    loop {
        config = reload().unwrap_or(config);
        println!();
        println!("══ GZMO Cron Wizard ══");
        println!("{}", cron::format_job_table(&cron::list_jobs(&config)));
        println!();
        println!("Commands: [l]ist  [p]review  [s]et time  [e]nable  [d]isable");
        println!("          [a]dd custom  [r]emove custom  [n] run now  [q]uit");
        print!("cron> ");
        io::stdout().flush()?;
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cmd = line.chars().next().unwrap_or(' ');
        match cmd {
            'q' | 'Q' => break,
            'l' | 'L' => cmd_list(&config)?,
            'p' | 'P' => {
                let id = prompt_line(&mut stdin, "Job id: ")?;
                let _ = cmd_preview(&config, &id, 5);
            }
            's' | 'S' => {
                let id = prompt_line(&mut stdin, "Builtin id (dream/distill/promote/embed/wiki_push): ")?;
                let time = prompt_line(&mut stdin, "Time UTC HH:MM: ")?;
                if let Err(e) = cmd_set_builtin(&config, &id, &time) {
                    eprintln!("Error: {e}");
                }
            }
            'e' | 'E' => {
                let id = prompt_line(&mut stdin, "Job id to enable: ")?;
                if let Err(e) = cmd_enable(&config, &id, true) {
                    eprintln!("Error: {e}");
                }
            }
            'd' | 'D' => {
                let id = prompt_line(&mut stdin, "Job id to disable: ")?;
                if let Err(e) = cmd_enable(&config, &id, false) {
                    eprintln!("Error: {e}");
                }
            }
            'a' | 'A' => {
                if let Err(e) = wizard_add(&mut stdin) {
                    eprintln!("Error: {e}");
                }
            }
            'r' | 'R' => {
                let id = prompt_line(&mut stdin, "Custom job id to remove: ")?;
                if let Err(e) = cmd_remove(&config, &id) {
                    eprintln!("Error: {e}");
                }
            }
            'n' | 'N' => {
                let id = prompt_line(&mut stdin, "Job id to run now: ")?;
                if let Err(e) = cmd_run(&config, identity, &id).await {
                    eprintln!("Error: {e}");
                }
            }
            'h' | '?' => print_help(),
            _ => eprintln!("Unknown command. Type h for help, q to quit."),
        }
    }
    Ok(())
}

fn wizard_add(stdin: &mut impl BufRead) -> Result<()> {
    println!("Schedule presets:");
    for (k, v) in cron::schedule_presets() {
        println!("  {k:16} → {v}");
    }
    let id = prompt_line(stdin, "New job id: ")?;
    if cron::BUILTIN_IDS.contains(&id.as_str()) {
        bail!("'{id}' is reserved");
    }
    let sched_in = prompt_line(stdin, "Schedule (preset name or 5-field cron): ")?;
    let schedule = cron::schedule_presets()
        .get(sched_in.as_str())
        .copied()
        .map(|s| s.to_string())
        .unwrap_or(sched_in);
    let kind = prompt_line(stdin, "Kind [shell/prompt]: ")?;
    let desc = prompt_line(stdin, "Description (optional): ").unwrap_or_default();
    let job = match kind.to_ascii_lowercase().as_str() {
        "shell" | "s" | "" => {
            let command = prompt_line(stdin, "Command: ")?;
            CustomCronJob {
                enabled: true,
                schedule,
                kind: CustomCronKind::Shell,
                command,
                prompt: String::new(),
                description: desc,
            }
        }
        "prompt" | "p" => {
            let prompt = prompt_line(stdin, "Prompt: ")?;
            CustomCronJob {
                enabled: true,
                schedule,
                kind: CustomCronKind::Prompt,
                command: String::new(),
                prompt,
                description: desc,
            }
        }
        other => bail!("Unknown kind '{other}'"),
    };
    validate_custom(&job)?;
    persist_custom_job(&config_path(), &id, &job)?;
    println!("Added '{id}'. Restart gzmo serve to schedule it.");
    // Preview
    let cfg = reload()?;
    let _ = cmd_preview(&cfg, &id, 3);
    Ok(())
}

fn prompt_line(stdin: &mut impl BufRead, label: &str) -> Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    stdin.read_line(&mut line)?;
    Ok(line.trim().to_string())
}
