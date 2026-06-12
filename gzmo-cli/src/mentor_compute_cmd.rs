//! `gzmo mentor compute` subcommand implementation.

use anyhow::{bail, Result};
use gzmo_core::config::GzmoConfig;
use gzmo_core::tools::{python_sandbox::PythonSandboxTool, ToolHandler};

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    if args.is_empty() {
        bail!("Usage: gzmo mentor compute <expression> | gzmo mentor compute --code <code>");
    }

    let code = if args[0] == "--code" {
        if args.len() < 2 {
            bail!("compute --code requires python code argument");
        }
        args[1..].join(" ")
    } else {
        let expr = args.join(" ");
        format!("print({})", expr)
    };

    let tool = PythonSandboxTool::new(&config.pedagogy);
    let output = tool.execute(serde_json::json!({ "code": code })).await?;
    println!("{}", output);
    Ok(())
}
