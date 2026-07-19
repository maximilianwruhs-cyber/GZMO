//! `gzmo init` — product MCP onboarding (default) + optional interactive wizard.
//!
//! Default (`gzmo init`): laptop-safe `~/.gzmo/` — SQLite vault, no LAN sidecars.
//! Wizard (`gzmo init --wizard`): legacy interactive agent setup in the current directory.

use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use gzmo_core::scanner;

// ─── ANSI helpers ───────────────────────────────────────────────────────

const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

#[derive(Debug, Default)]
struct InitArgs {
    wizard: bool,
    force: bool,
    refresh_engine: bool,
    dir: Option<PathBuf>,
    bin: Option<PathBuf>,
}

fn parse_args() -> Result<InitArgs> {
    let mut out = InitArgs::default();
    let mut args = env::args().skip(2);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--wizard" => out.wizard = true,
            "--force" | "-f" => out.force = true,
            "--refresh-engine" => out.refresh_engine = true,
            "--dir" => {
                let p = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--dir requires a path"))?;
                out.dir = Some(PathBuf::from(p));
            }
            "--bin" => {
                let p = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--bin requires a path"))?;
                out.bin = Some(PathBuf::from(p));
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => bail!("unknown init flag: {other} (try --help)"),
        }
    }
    Ok(out)
}

fn print_help() {
    eprintln!(
        "\
Usage:
  gzmo init [--force] [--dir PATH] [--bin PATH]   Product MCP home (~/.gzmo)
  gzmo init --refresh-engine [--dir PATH]          Update [engine] only (non-destructive)
  gzmo init --wizard [--force]                     Interactive agent setup (cwd)

Options:
  --force, -f         Overwrite existing gzmo.toml
  --refresh-engine    Rescan localhost; rewrite only [engine] url/model (keep vault/MCP)
  --dir PATH          Product home (default: ~/.gzmo)
  --bin PATH          Absolute path to gzmo binary for mcp.json (default: this executable)
  --wizard            Legacy interactive onboarding in the current directory
"
    );
}

fn product_home(override_dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(d) = override_dir {
        return Ok(d);
    }
    let home = env::var_os("HOME").context("HOME is unset; pass --dir")?;
    Ok(PathBuf::from(home).join(".gzmo"))
}

fn resolve_bin(override_bin: Option<PathBuf>) -> PathBuf {
    if let Some(b) = override_bin {
        return b;
    }
    env::current_exe().unwrap_or_else(|_| PathBuf::from("gzmo"))
}

/// Entry point for `gzmo init`.
pub async fn run() -> Result<()> {
    let args = parse_args()?;
    if args.wizard && args.refresh_engine {
        bail!("--refresh-engine cannot be combined with --wizard");
    }
    if args.refresh_engine {
        run_refresh_engine(args).await
    } else if args.wizard {
        run_wizard(args.force).await
    } else {
        run_product(args).await
    }
}

/// Non-destructive: rewrite only `[engine] url` / `model` from a localhost scan.
async fn run_refresh_engine(args: InitArgs) -> Result<()> {
    let home = product_home(args.dir)?;
    let config_path = home.join("gzmo.toml");
    if !config_path.exists() {
        bail!(
            "{} missing — run `gzmo init` first (or pass --dir)",
            config_path.display()
        );
    }

    eprintln!();
    eprintln!("  {BOLD}GZMO — refresh product engine{RESET}");
    eprintln!("  {DIM}Non-destructive: only [engine] url/model change{RESET}");
    eprintln!();

    let endpoints = scanner::scan_endpoints().await;
    let Some(ep) = scanner::prefer_product_engine(&endpoints) else {
        bail!("No local OpenAI-compatible engine responding — start Prime on :8000 and retry");
    };
    let model = ep
        .models
        .first()
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    let raw = tokio::fs::read_to_string(&config_path).await?;
    let updated = patch_engine_block(&raw, &ep.url, &model)?;
    if updated == raw {
        eprintln!("  {GREEN}✔{RESET} Already on {} ({})", ep.url, model);
        return Ok(());
    }

    let backup = home.join("gzmo.toml.bak-refresh-engine");
    tokio::fs::write(&backup, &raw).await?;
    tokio::fs::write(&config_path, &updated).await?;
    // Drop stale sibling overlay so product-first-fact rebuilds from fresh toml.
    let overlay = home.join("gzmo-product-engine.toml");
    if overlay.exists() {
        let _ = tokio::fs::remove_file(&overlay).await;
    }

    eprintln!(
        "  {GREEN}✔{RESET} Engine → {BOLD}{}{RESET} — {}",
        ep.name, ep.url
    );
    eprintln!("  {DIM}model={model}{RESET}");
    eprintln!("  {DIM}backup={}{RESET}", backup.display());
    eprintln!();
    Ok(())
}

fn patch_engine_block(toml: &str, url: &str, model: &str) -> Result<String> {
    let mut out = String::with_capacity(toml.len() + 64);
    let mut in_engine = false;
    let mut saw_url = false;
    let mut saw_model = false;
    let mut engine_present = false;

    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_engine {
                if !saw_url {
                    out.push_str(&format!("url = \"{url}\"\n"));
                }
                if !saw_model {
                    out.push_str(&format!("model = \"{model}\"\n"));
                }
            }
            in_engine = trimmed == "[engine]";
            if in_engine {
                engine_present = true;
                saw_url = false;
                saw_model = false;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_engine {
            if trimmed.starts_with("url") && trimmed.contains('=') {
                out.push_str(&format!("url = \"{url}\"\n"));
                saw_url = true;
                continue;
            }
            if trimmed.starts_with("model") && trimmed.contains('=') {
                out.push_str(&format!("model = \"{model}\"\n"));
                saw_model = true;
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    if in_engine {
        if !saw_url {
            out.push_str(&format!("url = \"{url}\"\n"));
        }
        if !saw_model {
            out.push_str(&format!("model = \"{model}\"\n"));
        }
    }
    if !engine_present {
        out.push_str(&format!(
            "\n[engine]\nprovider = \"local\"\nurl = \"{url}\"\nmodel = \"{model}\"\n"
        ));
    }
    Ok(out)
}

// ─── Product path (default) ─────────────────────────────────────────────

async fn run_product(args: InitArgs) -> Result<()> {
    let home = product_home(args.dir)?;
    let config_path = home.join("gzmo.toml");
    let data_dir = home.join("data");
    let memory_dir = home.join("memory");
    let vault_db = data_dir.join("vault.db");
    let mcp_path = home.join("mcp.json");
    let bin = resolve_bin(args.bin);

    eprintln!();
    eprintln!("  {BOLD}GZMO — product MCP init{RESET}");
    eprintln!("  {DIM}Local SQLite vault · no LAN sidecars · Cursor/Pi ready{RESET}");
    eprintln!();

    if config_path.exists() && !args.force {
        eprintln!(
            "  {YELLOW}{config} already exists. Re-run with --force to overwrite.{RESET}",
            config = config_path.display()
        );
        eprintln!(
            "  {DIM}Existing home kept. MCP fragment:{RESET} {}",
            mcp_path.display()
        );
        if !mcp_path.exists() {
            write_mcp_fragment(&mcp_path, &bin, &config_path)?;
            eprintln!("  {GREEN}✔{RESET} Wrote {}", mcp_path.display());
        }
        print_next_steps(&home, &mcp_path, &bin, &config_path);
        return Ok(());
    }

    tokio::fs::create_dir_all(&data_dir).await?;
    tokio::fs::create_dir_all(&memory_dir).await?;
    tokio::fs::create_dir_all(home.join("skills")).await?;

    eprintln!("  {DIM}Scanning localhost for OpenAI-compatible engines…{RESET}");
    let endpoints = scanner::scan_endpoints().await;
    let (engine_url, engine_model, engine_note) = if let Some(ep) =
        scanner::prefer_product_engine(&endpoints)
    {
        let model = ep
            .models
            .first()
            .cloned()
            .unwrap_or_else(|| "default".to_string());
        eprintln!(
            "  {GREEN}✔{RESET} Engine {BOLD}{}{RESET} — {} {DIM}({}ms){RESET}",
            ep.name, ep.url, ep.latency_ms
        );
        (ep.url.clone(), model, format!("auto-detected {}", ep.name))
    } else {
        eprintln!(
                "  {DIM}No local engine up — defaulting to http://127.0.0.1:1234/v1 (LM Studio).{RESET}"
            );
        eprintln!(
                "  {DIM}Start Prime on :8000 (or any OpenAI-compatible server) and re-init with --force to pick it up.{RESET}"
            );
        (
            "http://127.0.0.1:1234/v1".to_string(),
            "default".to_string(),
            "placeholder until a local engine is detected".to_string(),
        )
    };

    let toml = generate_product_toml(
        &vault_db,
        &memory_dir,
        &engine_url,
        &engine_model,
        &engine_note,
    );
    tokio::fs::write(&config_path, &toml).await?;
    eprintln!("  {GREEN}✔{RESET} {}", config_path.display());

    // Touch empty vault by loading config + opening via sqlite create path:
    // PlatformMemory refuses empty vaults unless lab/product allow — mcp-serve sets that.
    // Ensure parent exists; vault file is created on first open.
    if !vault_db.exists() {
        // Create minimal empty DB via config load smoke + sqlite open through memory API.
        let _ = gzmo_core::config::GzmoConfig::load(&config_path)?;
        match SqliteVaultTouch::touch(&vault_db) {
            Ok(()) => eprintln!(
                "  {GREEN}✔{RESET} {} {DIM}(empty lab vault){RESET}",
                vault_db.display()
            ),
            Err(e) => eprintln!(
                "  {YELLOW}⚠{RESET} vault not pre-created ({e}); mcp-serve will create it"
            ),
        }
    }

    write_mcp_fragment(&mcp_path, &bin, &config_path)?;
    eprintln!("  {GREEN}✔{RESET} {}", mcp_path.display());

    match gzmo_core::config::GzmoConfig::load(&config_path) {
        Ok(cfg) => {
            eprintln!(
                "  {GREEN}✔{RESET} Config loads · vault={}",
                cfg.memory.vault_db.display()
            );
            if cfg.redis.enabled || cfg.qdrant.enabled || cfg.embeddings.enabled {
                eprintln!("  {YELLOW}⚠{RESET} Unexpected sidecar enablement in product defaults");
            }
        }
        Err(e) => bail!("Generated config failed validation: {e}"),
    }

    eprintln!();
    eprintln!("  {GREEN}{BOLD}Done.{RESET}");
    print_next_steps(&home, &mcp_path, &bin, &config_path);
    Ok(())
}

/// Thin touch so we do not pull PlatformMemory (lab gate) during init.
struct SqliteVaultTouch;
impl SqliteVaultTouch {
    fn touch(path: &Path) -> Result<()> {
        use gzmo_core::memory::vault::SqliteVault;
        let _v = SqliteVault::open(path)?;
        Ok(())
    }
}

fn generate_product_toml(
    vault_db: &Path,
    memory_dir: &Path,
    engine_url: &str,
    engine_model: &str,
    engine_note: &str,
) -> String {
    // Absolute paths — no LAN hosts; sidecars off; embeddings off (FTS-only).
    format!(
        r#"# GZMO product config — generated by `gzmo init`
# Laptop-safe: SQLite vault, no Redis/Qdrant/Neo4j. Optional embeddings later.
# Engine: {engine_note}

[identity]
soul_path = "SOUL.md"
persona_name = "GZMO"

[engine]
provider = "local"
url = "{engine_url}"
model = "{engine_model}"
temperature = 0.3
top_p = 0.95
max_tokens = 4096

[agent]
max_tool_iterations = 40
heartbeat_interval_secs = 1800

[memory]
directory = "{memory}"
vault_db = "{vault}"
vault_backend = "sqlite"

[skills]
directory = "skills"
dreams_path = "DREAMS.md"

[dreams]
enabled = false

[wiki]
enabled = false

[embeddings]
enabled = false
# Optional OpenAI-compatible embeddings (leave disabled for offline FTS-only):
# enabled = true
# url = "http://127.0.0.1:8002/v1"
# model = "Qwen3-Embedding-0.6B"

[redis]
enabled = false

[qdrant]
enabled = false

[workflow_skills]
enabled = false
"#,
        engine_note = engine_note,
        engine_url = engine_url,
        engine_model = engine_model,
        memory = memory_dir.display(),
        vault = vault_db.display(),
    )
}

fn write_mcp_fragment(mcp_path: &Path, bin: &Path, config_path: &Path) -> Result<()> {
    let bin_s = bin.display().to_string();
    let cfg_s = config_path.display().to_string();
    let json = serde_json::json!({
        "mcpServers": {
            "gzmo-memory": {
                "command": bin_s,
                "args": ["mcp-serve"],
                "env": {
                    "GZMO_CONFIG": cfg_s,
                    "GZMO_ALLOW_LAB_VAULT": "1",
                    "GZMO_PRODUCT": "1"
                }
            }
        }
    });
    let text = serde_json::to_string_pretty(&json)? + "\n";
    std::fs::write(mcp_path, text).with_context(|| format!("write {}", mcp_path.display()))?;
    Ok(())
}

fn print_next_steps(home: &Path, mcp_path: &Path, bin: &Path, config_path: &Path) {
    eprintln!("  Home:   {}", home.display());
    eprintln!("  Config: {}", config_path.display());
    eprintln!("  Binary: {}", bin.display());
    eprintln!();
    eprintln!("  {BOLD}Next:{RESET}");
    eprintln!("    1. Merge MCP:  {DIM}./scripts/install-product-mcp.sh{RESET}");
    eprintln!("       or paste:  {}", mcp_path.display());
    eprintln!("    2. In Cursor/Pi call {BOLD}gzmo_memory_status{RESET} then {BOLD}gzmo_memory_search{RESET}");
    eprintln!("    3. Docs:       {DIM}docs/PRODUCT_MCP.md{RESET}");
    eprintln!();
    eprintln!("  {DIM}Snippet:{RESET}");
    eprintln!(
        r#"  {{
    "mcpServers": {{
      "gzmo-memory": {{
        "command": "{}",
        "args": ["mcp-serve"],
        "env": {{
          "GZMO_CONFIG": "{}",
          "GZMO_ALLOW_LAB_VAULT": "1",
          "GZMO_PRODUCT": "1"
        }}
      }}
    }}
  }}"#,
        bin.display(),
        config_path.display()
    );
    eprintln!();
}

// ─── Legacy wizard ──────────────────────────────────────────────────────

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
        eprint!(
            "  {CYAN}▸{RESET} Select {DIM}[1-{}]{RESET}: ",
            options.len()
        );
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

async fn run_wizard(force: bool) -> Result<()> {
    eprintln!();
    eprintln!("  {BOLD}╔══════════════════════════════════════════════╗{RESET}");
    eprintln!("  {BOLD}║          GZMO — Onboarding Wizard            ║{RESET}");
    eprintln!("  {BOLD}║     100% Local · Air-Gapped · Sovereign      ║{RESET}");
    eprintln!("  {BOLD}╚══════════════════════════════════════════════╝{RESET}");
    eprintln!();
    eprintln!("  {DIM}Tip: for Cursor/Pi memory MCP use `gzmo init` (no --wizard).{RESET}");
    eprintln!();

    if Path::new("gzmo.toml").exists()
        && !force
        && !prompt_confirm("gzmo.toml already exists. Overwrite?", false)
    {
        eprintln!("  {YELLOW}Aborted.{RESET}");
        return Ok(());
    }

    eprintln!("  {BOLD}Step 1:{RESET} Scanning for local LLM endpoints...");
    eprintln!();

    let endpoints = scanner::scan_endpoints().await;

    let (selected_url, selected_model) = if endpoints.is_empty() {
        eprintln!("  {YELLOW}No running LLM endpoints detected on localhost.{RESET}");
        eprintln!(
            "  {DIM}Start Prime (:8000), LM Studio, Ollama, or vLLM, then re-run 'gzmo init --wizard'.{RESET}"
        );
        eprintln!();

        let url = prompt_text("Enter your LLM endpoint URL", "http://localhost:1234/v1");

        match scanner::probe_endpoint(&url).await {
            Ok(ep) => {
                eprintln!(
                    "  {GREEN}✔{RESET} Connected to {} ({DIM}{}ms{RESET})",
                    ep.name, ep.latency_ms
                );
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
        for ep in &endpoints {
            eprintln!(
                "  {GREEN}✔{RESET} {BOLD}{}{RESET} — {} {DIM}({}ms, {} model{}){RESET}",
                ep.name,
                ep.url,
                ep.latency_ms,
                ep.models.len(),
                if ep.models.len() == 1 { "" } else { "s" }
            );
        }
        eprintln!();

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

        let model = if ep.models.is_empty() {
            prompt_text("Model name", "default")
        } else if ep.models.len() == 1 {
            eprintln!(
                "  {DIM}Auto-selected the only available model: {}{RESET}",
                ep.models[0]
            );
            ep.models[0].clone()
        } else {
            let idx = prompt_select("Select a model:", &ep.models);
            ep.models[idx].clone()
        };

        (ep.url.clone(), model)
    };

    eprintln!();
    eprintln!("  {BOLD}Step 2:{RESET} Agent Identity");
    eprintln!();

    let persona_name = prompt_text("Persona name", "GZMO");
    let soul_tagline = prompt_text(
        "One-line soul directive",
        "Sovereign local agent. Efficient, precise, no fluff.",
    );

    eprintln!();
    eprintln!("  {BOLD}Step 3:{RESET} Generating configuration...");
    eprintln!();

    let config_content = generate_wizard_toml(&selected_url, &selected_model, &persona_name);
    let soul_content = generate_soul(&persona_name, &soul_tagline);

    tokio::fs::create_dir_all("memory").await?;
    tokio::fs::create_dir_all("data").await?;
    tokio::fs::create_dir_all("skills").await?;
    tokio::fs::create_dir_all("models").await?;

    tokio::fs::write("gzmo.toml", &config_content).await?;
    eprintln!("  {GREEN}✔{RESET} gzmo.toml");

    if !Path::new("SOUL.md").exists() {
        tokio::fs::write("SOUL.md", &soul_content).await?;
        eprintln!("  {GREEN}✔{RESET} SOUL.md");
    } else {
        eprintln!("  {DIM}⊘ SOUL.md already exists, skipped{RESET}");
    }

    if !Path::new("models/README.md").exists() {
        tokio::fs::write(
            "models/README.md",
            "# Model Weights\nDrop your `.gguf` files in this folder for portable USB deployment.\n",
        )
        .await?;
    }

    eprintln!("  {GREEN}✔{RESET} memory/");
    eprintln!("  {GREEN}✔{RESET} data/");
    eprintln!("  {GREEN}✔{RESET} skills/");
    eprintln!("  {GREEN}✔{RESET} models/");

    eprintln!();
    eprintln!("  {BOLD}Step 4:{RESET} Health check...");
    eprintln!();

    match gzmo_core::config::GzmoConfig::load(Path::new("gzmo.toml")) {
        Ok(_) => eprintln!("  {GREEN}✔{RESET} Config loads cleanly"),
        Err(e) => {
            eprintln!("  {YELLOW}⚠{RESET} Config load issue: {e}");
            bail!("Generated config failed validation");
        }
    }

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
    eprintln!(
        "  {GREEN}{BOLD}Done!{RESET} Run {BOLD}gzmo{RESET} to start chatting, or {BOLD}gzmo daemon{RESET} for background mode."
    );
    eprintln!("  {DIM}For product MCP memory: gzmo init (without --wizard).{RESET}");
    eprintln!();

    Ok(())
}

fn generate_wizard_toml(url: &str, model: &str, persona: &str) -> String {
    format!(
        r#"# GZMO Configuration — generated by `gzmo init --wizard`

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

[agent]
max_tool_iterations = 40
heartbeat_interval_secs = 1800

[memory]
directory = "memory"
vault_db = "data/vault.db"

[skills]
directory = "skills"
dreams_path = "DREAMS.md"

[redis]
enabled = false

[qdrant]
enabled = false

[embeddings]
enabled = false
"#
    )
}

fn generate_soul(persona: &str, tagline: &str) -> String {
    format!(
        r#"# {persona} — SOUL

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

## Personality

Direct, competent, low-ego. Like a senior engineer who respects your time.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::patch_engine_block;

    #[test]
    fn patch_engine_rewrites_url_and_model_only() {
        let raw = r#"# header
[identity]
persona_name = "GZMO"

[engine]
provider = "local"
url = "http://127.0.0.1:1234/v1"
model = "default"
temperature = 0.3

[memory]
vault_db = "/tmp/v.db"
"#;
        let out = patch_engine_block(raw, "http://127.0.0.1:8000/v1", "ornith").unwrap();
        assert!(out.contains("url = \"http://127.0.0.1:8000/v1\""));
        assert!(out.contains("model = \"ornith\""));
        assert!(out.contains("temperature = 0.3"));
        assert!(out.contains("vault_db = \"/tmp/v.db\""));
        assert!(!out.contains(":1234"));
    }
}
