//! `gzmo observatory` — ecosystem health LED board (TUI) or `--json` snapshot.

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::observatory_board::collect_health_led_board;

use crate::tui::boards::health_board;

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    if args
        .iter()
        .any(|a| a == "-h" || a == "--help" || a == "help")
    {
        print_help();
        return Ok(());
    }
    if args.iter().any(|a| a == "--json" || a == "json") {
        let board = collect_health_led_board(config).await;
        println!("{}", serde_json::to_string_pretty(&board.snapshot_json())?);
        let wiki_okforge = config.wiki.enabled && config.wiki.backend == "okforge";
        if wiki_okforge && !board.knowledge_plane_down().is_empty() {
            anyhow::bail!("observatory knowledge plane DOWN");
        }
        return Ok(());
    }
    health_board::run(config).await
}

fn print_help() {
    eprintln!(
        "\
Usage:
  gzmo observatory           # TUI health LED board
  gzmo observatory --json    # scriptable snapshot (fail-closed if OKForge wiki plane DOWN)

Units: llama-prime (unit state only), gzmo-serve / gzmo-scheduler expected-offline
on the telescope, okforge.service. Wiki plane LEDs: okforge_http, wiki_push.
Forge UI: http://127.0.0.1:3000/observatory (not the retired :7777 sidecar).
"
    );
}
