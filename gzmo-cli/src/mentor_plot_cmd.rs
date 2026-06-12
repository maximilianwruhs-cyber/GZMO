//! `gzmo mentor plot` subcommand implementation.

use anyhow::{bail, Result};
use gzmo_core::config::GzmoConfig;
use gzmo_core::pedagogy::PedagogySession;
use gzmo_core::tools::{geogebra::GeoGebraPlotTool, ToolHandler};

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    let mut expr = String::new();
    let mut mode = "2d".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--expr" => {
                if let Some(val) = args.get(i + 1) {
                    expr = val.clone();
                    i += 2;
                } else {
                    bail!("--expr requires value");
                }
            }
            "--mode" => {
                if let Some(val) = args.get(i + 1) {
                    mode = val.clone();
                    i += 2;
                } else {
                    bail!("--mode requires value");
                }
            }
            other => {
                if expr.is_empty() {
                    expr = args[i..].join(" ");
                    break;
                } else {
                    bail!("Unknown argument: {}", other);
                }
            }
        }
    }

    let expr = expr.trim();
    if expr.is_empty() {
        bail!("Usage: gzmo mentor plot --expr <expression> [--mode 2d|3d]");
    }

    // Check ops mode
    let session = PedagogySession::load(&config.pedagogy).await?;
    if !session.ops_mode {
        bail!("Security Block: geogebra_plot tool is disabled outside ops mode. Run /ops first.");
    }

    let tool = GeoGebraPlotTool::new(&config.pedagogy);
    let output = tool.execute(serde_json::json!({
        "expression": expr,
        "mode": mode
    })).await?;

    println!("{}", output);
    Ok(())
}
