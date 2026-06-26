// Headroom Context Compression Layer
// Derived from or inspired by the Headroom project under the Apache License, Version 2.0.
// See the NOTICE file in this directory for the full licensing terms.

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
            let mut char_boundary = target_chars;
            while char_boundary > 0 && !content.is_char_boundary(char_boundary) {
                char_boundary -= 1;
            }
            if content.len() > char_boundary {
                format!("{}... [TRUNCATED TO BUDGET]", &content[..char_boundary])
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

    let ratio = original_tokens as f64 / compressed_tokens.max(1) as f64;
    tracing::debug!(
        route = ?route,
        original_tokens,
        compressed_tokens,
        ratio = format!("{ratio:.1}:1"),
        "Context compressed"
    );

    CompressedView {
        text: compressed_text,
        ccr_hash: None,
        original_tokens,
        compressed_tokens,
        route,
    }
}

pub async fn compress_for_context_with_ccr(
    content: &str,
    budget_tokens: usize,
    cfg: &ContextCompressConfig,
    ccr: &CcrStore,
    session_id: &str,
    store_full: bool,
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

    // Don't double-compress if the footer is already present
    if content.contains("[ccr:") && content.contains("gzmo_retrieve_context to expand]") {
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
            let mut char_boundary = target_chars;
            while char_boundary > 0 && !content.is_char_boundary(char_boundary) {
                char_boundary -= 1;
            }
            if content.len() > char_boundary {
                format!("{}... [TRUNCATED TO BUDGET]", &content[..char_boundary])
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

    let ccr_hash = if store_full {
        match ccr.store(session_id, content).await {
            Ok(hash) => Some(hash),
            Err(e) => {
                tracing::debug!("CCR store failed, fail-open: {e}");
                None
            }
        }
    } else {
        None
    };

    let final_text = if let Some(ref hash) = ccr_hash {
        format!("{}\n\n[ccr:{} — gzmo_retrieve_context to expand]", compressed_text, hash)
    } else {
        compressed_text
    };

    let compressed_tokens = estimate_text_tokens(&final_text, 3.5);

    let ratio = original_tokens as f64 / compressed_tokens.max(1) as f64;
    tracing::debug!(
        route = ?route,
        original_tokens,
        compressed_tokens,
        ratio = format!("{ratio:.1}:1"),
        ccr_stored = ccr_hash.is_some(),
        "Context compressed (CCR)"
    );

    CompressedView {
        text: final_text,
        ccr_hash,
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

    #[tokio::test]
    async fn test_compress_for_context_with_ccr() {
        let content = "A".repeat(1000);
        let cfg = ContextCompressConfig {
            enabled: true,
            ..ContextCompressConfig::default()
        };
        let ccr = CcrStore::mock();
        let view = compress_for_context_with_ccr(&content, 10, &cfg, &ccr, "session123", true).await;
        
        use sha2::Digest;
        let digest = sha2::Sha256::digest(content.as_bytes());
        let expected_hash = format!("{:x}", digest)[..16].to_string();
        
        assert_eq!(view.route, CompressRoute::Plain);
        assert!(view.text.contains("TRUNCATED TO BUDGET"));
        assert!(view.text.contains(&format!("[ccr:{} — gzmo_retrieve_context to expand]", expected_hash)));
        assert_eq!(view.ccr_hash, Some(expected_hash));
    }

    #[test]
    fn test_run_benchmarks() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let fixtures_path = std::path::Path::new(&manifest_dir).join("../scripts/compression-bench/fixtures");
        if !fixtures_path.exists() {
            println!("Fixtures path does not exist: {:?}", fixtures_path);
            return;
        }

        let cfg = ContextCompressConfig {
            enabled: true,
            ..ContextCompressConfig::default()
        };

        let mut results = serde_json::json!([]);

        let entries = std::fs::read_dir(fixtures_path).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let content = std::fs::read_to_string(&path).unwrap();
                let before_tokens = estimate_text_tokens(&content, 3.5);
                
                // Let's use a budget of 30% of original tokens
                let budget = (before_tokens as f64 * 0.3) as usize;
                let view = compress_for_context(&content, budget, &cfg);
                
                let after_tokens = view.compressed_tokens;
                let savings_pct = if before_tokens > 0 {
                    100.0 * (1.0 - (after_tokens as f64 / before_tokens as f64))
                } else {
                    0.0
                };
                
                results.as_array_mut().unwrap().push(serde_json::json!({
                    "file": path.file_name().unwrap().to_str().unwrap(),
                    "before_tokens": before_tokens,
                    "after_tokens": after_tokens,
                    "savings_pct": savings_pct,
                    "text": view.text,
                }));
            }
        }

        let output_dir = std::path::Path::new(&manifest_dir).join("../scripts/compression-bench/output");
        let _ = std::fs::create_dir_all(&output_dir);
        let output_path = output_dir.join("rust_results.json");
        std::fs::write(output_path, serde_json::to_string_pretty(&results).unwrap()).unwrap();
    }
}
