//! Sovereign Node compliance gates (ARCH-DIR).
//!
//! Tier 1 (strict): outbound blocked when `mode = "sovereign"` and `allow_cloud_tools = false`.
//! Tier 2 (exceptions): `network_exceptions` always permits web_search, agent-reach, and arxiv.

use crate::config::ComplianceConfig;

/// Network-oriented shell binaries gated under sovereign compliance.
const NETWORK_SHELL_BINARIES: &[&str] = &["curl", "wget"];

/// Substrings that indicate Agent-Reach / session-hijack tooling.
const AGENT_REACH_MARKERS: &[&str] = &[
    "agent-reach",
    "agent_reach",
    ".agent-reach",
    "xreach",
    "twitter-cli",
    "rdt-cli",
    "xiaohongshu-mcp",
    "linkedin-scraper",
    "yt-dlp",
    "cookie-editor",
    "opencli",
];

/// Substrings that indicate arXiv retrieval (OAI-PMH, export API, skill).
const ARXIV_MARKERS: &[&str] = &[
    "arxiv",
    "oaipmh.arxiv.org",
    "export.arxiv.org",
    "skill_arxiv",
    "arxiv_search",
    "semanticscholar.org/graph/v1",
];

/// Whether a named Tier-2 network exception is active.
pub fn has_network_exception(config: &ComplianceConfig, name: &str) -> bool {
    config
        .network_exceptions
        .iter()
        .any(|entry| entry.eq_ignore_ascii_case(name))
}

/// Whether outbound network / cloud tools are permitted (Tier 3 opt-in or non-sovereign).
pub fn allows_network_tools(config: &ComplianceConfig) -> bool {
    config.allow_cloud_tools || config.mode != "sovereign"
}

fn command_matches_any(lower: &str, markers: &[&str]) -> bool {
    markers.iter().any(|m| lower.contains(m))
}

/// Block reason for a shell command, if any.
pub fn shell_command_block_reason(command: &str, config: &ComplianceConfig) -> Option<String> {
    if allows_network_tools(config) {
        return None;
    }

    let lower = command.to_ascii_lowercase();

    if has_network_exception(config, "agent-reach")
        && command_matches_any(&lower, AGENT_REACH_MARKERS)
    {
        return None;
    }

    if has_network_exception(config, "arxiv") && command_matches_any(&lower, ARXIV_MARKERS) {
        return None;
    }

    if command_matches_any(&lower, AGENT_REACH_MARKERS) {
        return Some(format!(
            "Agent-Reach capability blocked: add \"agent-reach\" to compliance.network_exceptions \
             or set compliance.allow_cloud_tools = true (mode={})",
            config.mode
        ));
    }

    let binary = crate::tools::shell::shell_command_binary(command);
    if NETWORK_SHELL_BINARIES.contains(&binary) {
        return Some(format!(
            "Network shell command '{binary}' blocked under sovereign compliance \
             (not covered by network_exceptions; allow_cloud_tools = false)"
        ));
    }

    None
}

/// Block reason for web_search / web_read tools.
pub fn web_tool_block_reason(config: &ComplianceConfig) -> Option<String> {
    if allows_network_tools(config)
        || has_network_exception(config, "web_search")
        || has_network_exception(config, "web_read")
    {
        return None;
    }
    Some(format!(
        "Web tool blocked under sovereign compliance (mode={}, allow_cloud_tools=false, \
         web_search/web_read not in network_exceptions)",
        config.mode
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sovereign() -> ComplianceConfig {
        ComplianceConfig::default()
    }

    fn sovereign_no_exceptions() -> ComplianceConfig {
        ComplianceConfig {
            network_exceptions: vec![],
            ..ComplianceConfig::default()
        }
    }

    fn cloud_allowed() -> ComplianceConfig {
        ComplianceConfig {
            allow_cloud_tools: true,
            ..ComplianceConfig::default()
        }
    }

    #[test]
    fn blocks_generic_curl_under_sovereign() {
        assert!(shell_command_block_reason("curl -s https://example.com", &sovereign()).is_some());
    }

    #[test]
    fn allows_arxiv_curl_with_default_exceptions() {
        assert!(shell_command_block_reason(
            "curl -s 'https://oaipmh.arxiv.org/oai?verb=Identify'",
            &sovereign()
        )
        .is_none());
    }

    #[test]
    fn allows_curl_when_cloud_enabled() {
        assert!(shell_command_block_reason("curl -s https://example.com", &cloud_allowed()).is_none());
    }

    #[test]
    fn allows_agent_reach_with_default_exceptions() {
        assert!(shell_command_block_reason("agent-reach doctor", &sovereign()).is_none());
    }

    #[test]
    fn blocks_agent_reach_without_exceptions() {
        assert!(shell_command_block_reason("agent-reach doctor", &sovereign_no_exceptions()).is_some());
    }

    #[test]
    fn web_tools_allowed_with_default_exceptions() {
        assert!(web_tool_block_reason(&sovereign()).is_none());
    }

    #[test]
    fn web_tools_blocked_without_exceptions() {
        assert!(web_tool_block_reason(&sovereign_no_exceptions()).is_some());
    }
}
