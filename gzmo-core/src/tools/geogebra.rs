//! # GeoGebra Plot Stub Tool
//!
//! Generates a markdown link to a GeoGebra Graphing Calculator worksheet.
//! Gated strictly to `ops_mode` to ensure safety.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;

use super::{ToolDef, ToolHandler};
use crate::config::PedagogyConfig;
use crate::pedagogy::PedagogySession;

pub struct GeoGebraPlotTool {
    config: PedagogyConfig,
}

impl GeoGebraPlotTool {
    pub fn new(config: &PedagogyConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

#[async_trait]
impl ToolHandler for GeoGebraPlotTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "geogebra_plot".to_string(),
            description: "Generate a clickable GeoGebra graphing worksheet link for a math expression. Requires ops mode (/ops).".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "The math expression to plot (e.g. y=x^2 or y=sin(x))"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["2d", "3d"],
                        "description": "The plotting mode: 2d (Graphing Calculator) or 3d (3D Calculator)"
                    }
                },
                "required": ["expression", "mode"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let session = PedagogySession::load(&self.config).await?;
        if !session.ops_mode {
            return Err(anyhow!(
                "Security Block: geogebra_plot tool is disabled outside ops mode. \
                 Please run /ops to enable execution-first tools."
            ));
        }

        let expr = args["expression"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'expression' argument"))?
            .trim();
        let mode = args["mode"].as_str().unwrap_or("2d").trim();

        if expr.is_empty() {
            return Err(anyhow!("Expression cannot be empty"));
        }

        let encoded_expr = url_encode(expr);

        let url = if mode == "3d" {
            format!("https://www.geogebra.org/3d?expr={}", encoded_expr)
        } else {
            format!("https://www.geogebra.org/graphing?expr={}", encoded_expr)
        };

        Ok(format!("[Open GeoGebra worksheet]({})", url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pedagogy::PedagogySession;

    #[tokio::test]
    async fn test_geogebra_plot_tool_gates() {
        let mut config = PedagogyConfig::default();
        let temp_dir =
            std::env::temp_dir().join(format!("gzmo-test-geogebra-{}", uuid::Uuid::new_v4()));
        config.learner_data_dir = temp_dir.to_string_lossy().to_string();

        let tool = GeoGebraPlotTool::new(&config);

        // Fails closed outside ops mode
        let res = tool
            .execute(json!({ "expression": "y=x^2", "mode": "2d" }))
            .await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("disabled outside ops mode"));

        // Enable ops mode
        let mut session = PedagogySession::default();
        session.ops_mode = true;
        session.save(&config).await.expect("save session");

        // Succeeds in ops mode
        let res = tool
            .execute(json!({ "expression": "y=x^2", "mode": "2d" }))
            .await;
        assert!(res.is_ok());
        let output = res.unwrap();
        assert!(output.contains(
            "[Open GeoGebra worksheet](https://www.geogebra.org/graphing?expr=y%3Dx%5E2)"
        ));

        // 3D mode
        let res = tool
            .execute(json!({ "expression": "z=x^2+y^2", "mode": "3d" }))
            .await;
        assert!(res.is_ok());
        let output = res.unwrap();
        assert!(output.contains("https://www.geogebra.org/3d?expr=z%3Dx%5E2%2By%5E2"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
