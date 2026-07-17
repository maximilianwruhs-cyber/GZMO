//! # Pedagogy Tool Registry Builder
//!
//! Exposes a restricted tool registry builder that constructs only the tools
//! permitted on computational offloading and learner history paths.

use crate::config::PedagogyConfig;
use crate::tools::geogebra::GeoGebraPlotTool;
use crate::tools::learner::{LearnerRecallTool, LearnerUpdateTool};
use crate::tools::python_sandbox::PythonSandboxTool;
use crate::tools::ToolRegistry;

/// Construct a registry containing tools allowed in pedagogy execution modes.
pub fn build_pedagogy_tool_registry(config: &PedagogyConfig) -> ToolRegistry {
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(PythonSandboxTool::new(config)));
    tools.register(Box::new(LearnerRecallTool::new(config)));
    tools.register(Box::new(LearnerUpdateTool::new(config)));
    tools.register(Box::new(GeoGebraPlotTool::new(config)));
    tools
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::ToolCall;

    #[tokio::test]
    async fn test_pedagogy_tool_registry() {
        let mut config = PedagogyConfig::default();
        let temp_dir =
            std::env::temp_dir().join(format!("gzmo-test-learner-{}", uuid::Uuid::new_v4()));
        config.learner_data_dir = temp_dir.to_string_lossy().to_string();

        let registry = build_pedagogy_tool_registry(&config);

        assert!(registry.has_tool("python_sandbox"));
        assert!(registry.has_tool("learner_recall"));
        assert!(registry.has_tool("learner_update"));
        assert!(registry.has_tool("geogebra_plot"));

        let call = ToolCall {
            id: "call-123".to_string(),
            function_name: "learner_recall".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = registry.dispatch(&call).await;
        assert!(result.success);
        assert!(result.output.contains("Learner Profile"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
