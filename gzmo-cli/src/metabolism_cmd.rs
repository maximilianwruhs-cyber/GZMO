//! `gzmo metabolism` — overnight job board (TUI) or headless watchdog probe.

use anyhow::{bail, Result};
use gzmo_core::config::GzmoConfig;
use gzmo_core::metabolism;

use crate::tui::boards::metabolism_board;

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        None | Some("--tui") | Some("tui") => metabolism_board::run(config).await,
        Some("watchdog") | Some("--watchdog") => {
            let wd = metabolism::evaluate_and_write_watchdog(config);
            println!("{}", serde_json::to_string_pretty(&wd)?);
            if wd.stale {
                // Soft-fail: exit 0 so scripts can read JSON; stale is in the payload.
                eprintln!("[watchdog] STALE — {}", wd.detail);
            } else {
                eprintln!("[watchdog] fresh — {}", wd.detail);
            }
            Ok(())
        }
        Some("-h") | Some("--help") | Some("help") => {
            print_help();
            Ok(())
        }
        Some(other) => {
            bail!(
                "unknown metabolism subcommand '{other}'. Try: gzmo metabolism [tui|watchdog|--help]"
            )
        }
    }
}

fn print_help() {
    eprintln!(
        "\
Usage:
  gzmo metabolism              # TUI job board
  gzmo metabolism tui          # same
  gzmo metabolism watchdog     # write + print latest-watchdog.json (soft-fail; exit 0)

Env:
  GZMO_METABOLISM_STALE_SECS   override 26h threshold (seconds) for burst tests
"
    );
}
