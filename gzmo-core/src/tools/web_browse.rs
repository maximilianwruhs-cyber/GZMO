//! # Web Browse Tool
//!
//! Fetches a URL and extracts readable text content.
//! Strips HTML tags, scripts, styles, and navigation boilerplate
//! to produce clean markdown-like text suitable for LLM consumption.
//!
//! This is the lightweight version — pure HTTP with no JavaScript execution.
//! For JS-heavy SPAs, use the Puppeteer MCP server (cloud mode).

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

use crate::tools::{ToolDef, ToolHandler};

/// Fetches a web page and extracts readable text.
pub struct WebBrowseTool {
    http: reqwest::Client,
}

impl Default for WebBrowseTool {
    fn default() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (compatible; GZMO/1.0; +https://gzmo.dev)")
                .timeout(std::time::Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }
}

#[async_trait]
impl ToolHandler for WebBrowseTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "web_read".to_string(),
            description: "Fetch a web page and extract its readable text content. Use this to read documentation, articles, API references, or any URL. Returns clean text stripped of HTML.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch and read"
                    },
                    "max_chars": {
                        "type": "integer",
                        "description": "Maximum characters to return (default: 12000)"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let url = args["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' argument"))?;

        let max_chars = args["max_chars"]
            .as_u64()
            .unwrap_or(12000) as usize;

        tracing::info!(url = %url, "Fetching web page");

        let resp = self.http
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch {}: {}", url, e))?;

        let status = resp.status();
        if !status.is_success() {
            return Ok(format!("HTTP {} — failed to fetch {}", status, url));
        }

        let content_type = resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let body = resp.text().await
            .map_err(|e| anyhow::anyhow!("Failed to read body: {}", e))?;

        // If it's not HTML, return as-is (truncated)
        if !content_type.contains("html") {
            let mut text = body;
            if text.len() > max_chars {
                text.truncate(max_chars);
                text.push_str("\n\n... [content truncated]");
            }
            return Ok(text);
        }

        // Extract readable text from HTML
        let text = Self::extract_text(&body);

        let mut output = format!("# Content from: {}\n\n{}", url, text);

        if output.len() > max_chars {
            output.truncate(max_chars);
            output.push_str("\n\n... [content truncated]");
        }

        tracing::info!(chars = output.len(), "Web page extracted");
        Ok(output)
    }
}

impl WebBrowseTool {
    /// Extract readable text from HTML by removing scripts, styles, nav, and tags.
    fn extract_text(html: &str) -> String {
        let mut text = html.to_string();

        // Remove script and style blocks entirely
        let remove_blocks = ["script", "style", "noscript", "nav", "header", "footer", "iframe"];
        for tag in &remove_blocks {
            let pattern_open = format!("<{}", tag);
            let pattern_close = format!("</{}>", tag);
            while let Some(start) = text.to_lowercase().find(&pattern_open) {
                if let Some(end) = text.to_lowercase()[start..].find(&pattern_close) {
                    text.replace_range(start..start + end + pattern_close.len(), "");
                } else {
                    // No closing tag — remove to end of opening tag
                    if let Some(gt) = text[start..].find('>') {
                        text.replace_range(start..start + gt + 1, "");
                    } else {
                        break;
                    }
                }
            }
        }

        // Convert common block elements to newlines
        let block_tags = ["br", "p", "div", "li", "h1", "h2", "h3", "h4", "h5", "h6", "tr", "td", "th", "article", "section"];
        for tag in &block_tags {
            text = text.replace(&format!("<{}", tag), &format!("\n<{}", tag));
            text = text.replace(&format!("</{}>", tag), &format!("</{}>\n", tag));
        }

        // Strip all remaining HTML tags
        let mut result = String::with_capacity(text.len());
        let mut in_tag = false;
        for ch in text.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }

        // Decode HTML entities
        result = result
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ");

        // Collapse whitespace: multiple spaces → single, multiple newlines → double
        let lines: Vec<String> = result
            .lines()
            .map(|l| {
                l.split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .filter(|l| !l.is_empty())
            .collect();

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_basic() {
        let html = "<html><head><title>Test</title></head><body><p>Hello <b>world</b></p></body></html>";
        let text = WebBrowseTool::extract_text(html);
        assert!(text.contains("Hello world"));
        assert!(!text.contains("<p>"));
    }

    #[test]
    fn test_extract_text_strips_scripts() {
        let html = r#"<p>Before</p><script>var x = 1;</script><p>After</p>"#;
        let text = WebBrowseTool::extract_text(html);
        assert!(text.contains("Before"));
        assert!(text.contains("After"));
        assert!(!text.contains("var x"));
    }

    #[test]
    fn test_extract_text_entities() {
        let html = "<p>A &amp; B &lt; C</p>";
        let text = WebBrowseTool::extract_text(html);
        assert!(text.contains("A & B < C"));
    }
}
