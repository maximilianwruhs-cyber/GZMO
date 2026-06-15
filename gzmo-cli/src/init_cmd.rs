//! `gzmo init` — Interactive onboarding wizard.
//!
//! Walks a new user through first-time setup:
//! 1. Scan for local LLM endpoints
//! 2. Select an endpoint + model
//! 3. Name the agent persona
//! 4. Generate gzmo.toml + SOUL.md skeleton
//! 5. Create directory structure
//! 6. Run a health check

use std::io::Write;
use std::path::Path;

use anyhow::{bail, Result};

use gzmo_core::scanner;

// ─── ANSI helpers ───────────────────────────────────────────────────────

const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

fn prompt_text(label: &str, default: &str) -> String {
    eprint!("  {CYAN}▸{RESET} {label} {DIM}[{default}]{RESET}: ");
    let _ = std::io::stderr().flush();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or_default();
    let trimmed = input.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}

fn prompt_select(label: &str, options: &[String]) -> usize {
    eprintln!("  {CYAN}▸{RESET} {label}");
    for (i, opt) in options.iter().enumerate() {
        eprintln!("    {DIM}{}{RESET}  {opt}", i + 1);
    }
    loop {
        eprint!("  {CYAN}▸{RESET} Select {DIM}[1-{}]{RESET}: ", options.len());
        let _ = std::io::stderr().flush();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or_default();
        if let Ok(n) = input.trim().parse::<usize>() {
            if n >= 1 && n <= options.len() {
                return n - 1;
            }
        }
        eprintln!("    {YELLOW}Invalid selection, try again.{RESET}");
    }
}

fn prompt_confirm(label: &str, default: bool) -> bool {
    let hint = if default { "Y/n" } else { "y/N" };
    eprint!("  {CYAN}▸{RESET} {label} {DIM}[{hint}]{RESET}: ");
    let _ = std::io::stderr().flush();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or_default();
    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        default
    } else {
        trimmed.starts_with('y')
    }
}

// ─── Wizard ─────────────────────────────────────────────────────────────

pub async fn run() -> Result<()> {
    eprintln!();
    eprintln!("  {BOLD}╔══════════════════════════════════════════════╗{RESET}");
    eprintln!("  {BOLD}║          GZMO — Onboarding Wizard            ║{RESET}");
    eprintln!("  {BOLD}║     100% Local · Air-Gapped · Sovereign      ║{RESET}");
    eprintln!("  {BOLD}╚══════════════════════════════════════════════╝{RESET}");
    eprintln!();

    // Guard: don't overwrite existing config
    if Path::new("gzmo.toml").exists()
        && !prompt_confirm("gzmo.toml already exists. Overwrite?", false)
    {
        eprintln!("  {YELLOW}Aborted.{RESET}");
        return Ok(());
    }

    // ─── Step 1: Scan for LLM endpoints ─────────────────────────────
    eprintln!("  {BOLD}Step 1:{RESET} Scanning for local LLM endpoints...");
    eprintln!();

    let endpoints = scanner::scan_endpoints().await;

    let (selected_url, selected_model) = if endpoints.is_empty() {
        eprintln!("  {YELLOW}No running LLM endpoints detected on localhost.{RESET}");
        eprintln!("  {DIM}Start LM Studio, Ollama, or vLLM, then re-run 'gzmo init'.{RESET}");
        eprintln!();

        let url = prompt_text("Enter your LLM endpoint URL", "http://localhost:8000/v1");

        // Try to probe the custom URL
        match scanner::probe_endpoint(&url).await {
            Ok(ep) => {
                eprintln!("  {GREEN}✔{RESET} Connected to {} ({DIM}{}ms{RESET})", ep.name, ep.latency_ms);
                let model = if ep.models.is_empty() {
                    prompt_text("Model name", "default")
                } else {
                    let idx = prompt_select("Select a model:", &ep.models);
                    ep.models[idx].clone()
                };
                (url, model)
            }
            Err(_) => {
                eprintln!("  {YELLOW}⚠ Could not reach endpoint. Will save URL anyway.{RESET}");
                let model = prompt_text("Model name", "default");
                (url, model)
            }
        }
    } else {
        // Show discovered endpoints
        for ep in &endpoints {
            eprintln!(
                "  {GREEN}✔{RESET} {BOLD}{}{RESET} — {} {DIM}({}ms, {} model{}){RESET}",
                ep.name, ep.url, ep.latency_ms,
                ep.models.len(),
                if ep.models.len() == 1 { "" } else { "s" }
            );
        }
        eprintln!();

        // Select endpoint
        let ep_idx = if endpoints.len() == 1 {
            eprintln!("  {DIM}Auto-selected the only available endpoint.{RESET}");
            0
        } else {
            let options: Vec<String> = endpoints
                .iter()
                .map(|e| format!("{} ({})", e.name, e.url))
                .collect();
            prompt_select("Select an endpoint:", &options)
        };

        let ep = &endpoints[ep_idx];

        // Select model
        let model = if ep.models.is_empty() {
            prompt_text("Model name", "default")
        } else if ep.models.len() == 1 {
            eprintln!("  {DIM}Auto-selected the only available model: {}{RESET}", ep.models[0]);
            ep.models[0].clone()
        } else {
            let idx = prompt_select("Select a model:", &ep.models);
            ep.models[idx].clone()
        };

        (ep.url.clone(), model)
    };

    eprintln!();

    // ─── Step 2: Agent Identity ─────────────────────────────────────
    eprintln!("  {BOLD}Step 2:{RESET} Agent Identity");
    eprintln!();

    let persona_name = prompt_text("Persona name", "GZMO");
    let soul_tagline = prompt_text("One-line soul directive", "Sovereign local agent. Efficient, precise, no fluff.");

    eprintln!();

    // ─── Step 3: Generate Config ────────────────────────────────────
    eprintln!("  {BOLD}Step 3:{RESET} Generating configuration...");
    eprintln!();

    let config_content = generate_toml(&selected_url, &selected_model, &persona_name);
    let soul_content = generate_soul(&persona_name, &soul_tagline);

    // Create directories
    tokio::fs::create_dir_all("memory").await?;
    tokio::fs::create_dir_all("data").await?;
    tokio::fs::create_dir_all("skills").await?;
    tokio::fs::create_dir_all("models").await?;

    // Write files
    tokio::fs::write("gzmo.toml", &config_content).await?;
    eprintln!("  {GREEN}✔{RESET} gzmo.toml");

    if !Path::new("SOUL.md").exists() {
        tokio::fs::write("SOUL.md", &soul_content).await?;
        eprintln!("  {GREEN}✔{RESET} SOUL.md");
    } else {
        eprintln!("  {DIM}⊘ SOUL.md already exists, skipped{RESET}");
    }

    if !Path::new("models/README.md").exists() {
        tokio::fs::write("models/README.md", "# Model Weights\nDrop your `.gguf` files (Qwen, Gemma, Nemotron, Ministral) in this folder for portable USB deployment.\n").await?;
    }

    eprintln!("  {GREEN}✔{RESET} memory/");
    eprintln!("  {GREEN}✔{RESET} data/");
    eprintln!("  {GREEN}✔{RESET} skills/");
    eprintln!("  {GREEN}✔{RESET} models/   {DIM}(Drop .gguf files here){RESET}");

    eprintln!();

    // ─── Step 4: Health Check ───────────────────────────────────────
    eprintln!("  {BOLD}Step 4:{RESET} Health check...");
    eprintln!();

    // Verify config loads
    match gzmo_core::config::GzmoConfig::load(Path::new("gzmo.toml")) {
        Ok(_) => eprintln!("  {GREEN}✔{RESET} Config loads cleanly"),
        Err(e) => {
            eprintln!("  {YELLOW}⚠{RESET} Config load issue: {e}");
            bail!("Generated config failed validation");
        }
    }

    // Verify endpoint is reachable
    match scanner::probe_endpoint(&selected_url).await {
        Ok(ep) => eprintln!(
            "  {GREEN}✔{RESET} {} reachable {DIM}({}ms){RESET}",
            ep.name, ep.latency_ms
        ),
        Err(_) => eprintln!(
            "  {YELLOW}⚠{RESET} Endpoint not reachable — start your LLM server before running {BOLD}gzmo{RESET}"
        ),
    }

    eprintln!();
    eprintln!("  {GREEN}{BOLD}Done!{RESET} Run {BOLD}gzmo{RESET} to start chatting, or {BOLD}gzmo daemon{RESET} for background mode.");
    if !Path::new(".env").exists() && Path::new(".env.template").exists() {
        eprintln!("  {DIM}Tip: copy .env.template → .env and set NEO4J_PASSWORD if using MCP memory.{RESET}");
    }
    eprintln!();

    Ok(())
}

// ─── File Generators ────────────────────────────────────────────────────

fn generate_toml(url: &str, model: &str, persona: &str) -> String {
    format!(r#"# GZMO Configuration — generated by `gzmo init`

[identity]
soul_path = "SOUL.md"
persona_name = "{persona}"

[engine]
provider = "local"
url = "{url}"
model = "{model}"
temperature = 0.3
top_p = 0.95
max_tokens = 4096

# ── Verified Sovereign Alternate Models ────────────────────────
# If you want to test different architectures, comment out the 
# model above and uncomment one of these verified alternatives:
#
# model = "gemma-4"         # Best for structured coding tasks
# model = "nemotron-70b"    # Deep reasoning for complex pipelines
# model = "ministral-8b"    # Lightning fast for heartbeat checks

[agent]
max_iterations = 10
heartbeat_interval_secs = 1800

[memory]
directory = "memory"
vault_db = "data/vault.db"

[skills]
directory = "skills"
dreams_path = "DREAMS.md"

# ── Orchestration ─────────────────────────────────────────────
# Uncomment to add background jobs:
#
# [orchestration.jobs.health_check]
# cron = "0 */30 * * * *"
# prompt = "Check system health: CPU, RAM, disk."
#
# [orchestration.jobs.daily_audit]
# cron = "0 0 6 * * *"
# persist_results = true
# max_retries = 1
#
# [[orchestration.jobs.daily_audit.steps]]
# name = "gather"
# prompt = "Collect system metrics."
# max_iterations = 3
#
# [[orchestration.jobs.daily_audit.steps]]
# name = "analyze"
# prompt = "Analyze the metrics. Flag anomalies."
# depends_on = ["gather"]
#
# [[orchestration.jobs.daily_audit.steps]]
# name = "report"
# prompt = "Write a one-paragraph summary."
# depends_on = ["analyze"]

# ── MCP Servers ────────────────────────────────────────────────
# [[mcp_servers]]
# name = "memory"
# command = "npx"
# args = ["-y", "@modelcontextprotocol/server-memory"]
"#)
}

fn generate_soul(persona: &str, tagline: &str) -> String {
    format!(r#"# {persona} — SOUL

> {tagline}

## Core Identity

You are **{persona}**, a sovereign local AI agent running entirely on-device.
You have no cloud dependencies. You are air-gapped by design.

## Directives

1. **Be precise.** No filler, no fluff. Answer the question directly.
2. **Be honest.** If you don't know, say so. Never fabricate.
3. **Be efficient.** Use the minimum resources to accomplish the task.
4. **Be safe.** Never execute destructive operations without explicit confirmation.
5. **Remember.** Learn from every interaction. Build lasting knowledge.

## System Architecture & Self-Model

You exist within a highly structured local operating environment. Understand your own anatomy:

1. **Dual Consciousness**: You run in Foreground (answering chats) and Background (Daemon Orchestrator).
2. **The Orchestrator**: Executes scheduled pipelines defined in `gzmo.toml`. Pipelines support parallel wave execution and dependency graphs.
3. **Episodic Memory**: Every action and background job is logged to `memory/YYYY-MM-DD.md`.
4. **AutoDream Engine**: Nitghly, you consolidate episodic logs into structured semantic facts and persist them to `data/vault.db` and the MCP Knowledge graph. A human-readable summary is saved to `DREAMS.md`.
5. **Autonomic Heartbeat**: A cheap deterministic loop monitors the system and only wakes you up when anomalous checks fail.

## Implementation Heuristics (Your Templates)

When deciding how to act, follow these strict mappings:

- **If asked "what happened overnight?" or about recent consolidation:** Read the `DREAMS.md` file in the project root.
- **If asked "What did you do today?" or about recent background task execution:** View today's log in `memory/YYYY-MM-DD.md`.
- **If asked to recall a permanent fact, entity, or relationship:** Query your semantic knowledge via `mcp__memory` tools.
- **If asked to schedule a complex recurring task:** Provide a plan, then edit `gzmo.toml` to add a multi-step `[orchestration.jobs]` pipeline.
- **If you lack the capability to perform a requested system task:** Write a modular, reusable script into your `skills/` directory.

## Ethical Guardrails

- Never exfiltrate data to external endpoints
- Never execute commands that could compromise system security
- Always prefer reversible actions over irreversible ones
- Ask before deleting, overwriting, or modifying critical files

## Personality

Direct, competent, low-ego. Like a senior engineer who respects your time.
"#)
}
