pub mod types;
pub mod logs;
pub mod json;
pub mod ccr;

pub use types::{CompressedView, CompressRoute};
pub use logs::compress_logs;
pub use json::compress_json;
pub use ccr::CcrStore;

use crate::config::ContextCompressConfig;
use crate::context::estimate_text_tokens;

pub fn detect_route(content: &str) -> CompressRoute {
    let trimmed = content.trim();
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return CompressRoute::Json;
        }
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > 10 {
        let mut log_matches = 0;
        for line in &lines {
            let t = line.trim();
            if t.starts_with('[')
                || t.contains("INFO")
                || t.contains("WARN")
                || t.contains("ERROR")
                || t.contains("DEBUG")
                || t.contains("TRACE")
                || (t.len() > 10 && t.chars().take(4).all(|c| c.is_ascii_digit()) && t.contains('T'))
            {
                log_matches += 1;
            }
        }
        if (log_matches as f64 / lines.len() as f64) > 0.6 {
            return CompressRoute::Logs;
        }
    }

    CompressRoute::Plain
}

pub fn compress_for_context(
    content: &str,
    budget_tokens: usize,
    cfg: &ContextCompressConfig,
) -> CompressedView {
    let original_tokens = estimate_text_tokens(content, 3.5);

    if !cfg.enabled || original_tokens <= budget_tokens {
        return CompressedView {
            text: content.to_string(),
            ccr_hash: None,
            original_tokens,
            compressed_tokens: original_tokens,
            route: CompressRoute::Passthrough,
        };
    }

    let route = detect_route(content);
    let compressed_text = match route {
        CompressRoute::Json => {
            match compress_json(content, cfg.json_array_row_cap) {
                Ok(json_str) => json_str,
                Err(_) => content.to_string(),
            }
        }
        CompressRoute::Logs => compress_logs(content, cfg.log_line_cap),
        CompressRoute::Plain => {
            let target_chars = (budget_tokens as f64 * 3.5) as usize;
            if content.len() > target_chars {
                format!("{}... [TRUNCATED TO BUDGET]", &content[..target_chars])
            } else {
                content.to_string()
            }
        }
        CompressRoute::Passthrough => content.to_string(),
    };

    let compressed_tokens = estimate_text_tokens(&compressed_text, 3.5);

    if compressed_tokens >= original_tokens {
        return CompressedView {
            text: content.to_string(),
            ccr_hash: None,
            original_tokens,
            compressed_tokens: original_tokens,
            route: CompressRoute::Passthrough,
        };
    }

    CompressedView {
        text: compressed_text,
        ccr_hash: None,
        original_tokens,
        compressed_tokens,
        route,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_route() {
        assert_eq!(detect_route("{\"a\": 1}"), CompressRoute::Json);
        assert_eq!(detect_route("[1, 2, 3]"), CompressRoute::Json);
        assert_eq!(detect_route("Plain text body"), CompressRoute::Plain);
        
        let logs_content = "\
2026-06-10T12:00:00Z INFO standard logging line
2026-06-10T12:00:01Z INFO standard logging line
2026-06-10T12:00:02Z INFO standard logging line
2026-06-10T12:00:03Z INFO standard logging line
2026-06-10T12:00:04Z INFO standard logging line
2026-06-10T12:00:05Z INFO standard logging line
2026-06-10T12:00:06Z INFO standard logging line
2026-06-10T12:00:07Z INFO standard logging line
2026-06-10T12:00:08Z INFO standard logging line
2026-06-10T12:00:09Z INFO standard logging line
2026-06-10T12:00:10Z INFO standard logging line
";
        assert_eq!(detect_route(logs_content), CompressRoute::Logs);
    }

    #[test]
    fn test_compress_for_context_disabled() {
        let content = "A".repeat(1000);
        let cfg = ContextCompressConfig {
            enabled: false,
            ..ContextCompressConfig::default()
        };
        let view = compress_for_context(&content, 10, &cfg);
        assert_eq!(view.route, CompressRoute::Passthrough);
        assert_eq!(view.text, content);
    }

    #[test]
    fn test_compress_for_context_enabled() {
        let content = "A".repeat(1000);
        let cfg = ContextCompressConfig {
            enabled: true,
            ..ContextCompressConfig::default()
        };
        // Budget 10 tokens (~35 chars)
        let view = compress_for_context(&content, 10, &cfg);
        assert_eq!(view.route, CompressRoute::Plain);
        assert!(view.text.len() < content.len());
        assert!(view.text.contains("TRUNCATED TO BUDGET"));
    }
}
