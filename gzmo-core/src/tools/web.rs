//! # Web Search Tool
//!
//! Gives GZMO the ability to search the internet via DuckDuckGo.
//! Zero API keys required — uses the DuckDuckGo HTML endpoint.
//!
//! Returns a clean list of title + URL + snippet for the top results.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

use crate::tools::{ToolDef, ToolHandler};

/// Web search tool using DuckDuckGo HTML (no API key needed).
pub struct WebSearchTool {
    http: reqwest::Client,
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }
}

#[async_trait]
impl ToolHandler for WebSearchTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "web_search".to_string(),
            description: "Search the internet using DuckDuckGo. Returns titles, URLs, and snippets for the top results. Use this to find current information, documentation, news, or research topics.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 5, max: 10)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' argument"))?;

        let max_results = args["max_results"]
            .as_u64()
            .unwrap_or(5)
            .min(10) as usize;

        tracing::info!(query = %query, max_results, "Executing web search");

        let results = self.search_ddg(query, max_results).await?;

        if results.is_empty() {
            return Ok("No results found for this query.".to_string());
        }

        // Format results as clean text for the LLM
        let mut output = format!("Search results for: \"{}\"\n\n", query);

        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!(
                "{}. {}\n   {}\n   {}\n\n",
                i + 1,
                result.title,
                result.url,
                result.snippet
            ));
        }

        // Truncate if too long
        if output.len() > 8000 {
            output.truncate(8000);
            output.push_str("\n... [results truncated]");
        }

        Ok(output)
    }
}

/// A single search result.
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

impl WebSearchTool {
    /// Search DuckDuckGo via the HTML lite interface.
    /// This endpoint returns simple HTML that's easy to parse
    /// without a full browser or JavaScript execution.
    async fn search_ddg(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        let url = "https://html.duckduckgo.com/html/";

        let resp = self
            .http
            .post(url)
            .form(&[("q", query), ("kl", ""), ("df", "")])
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "DuckDuckGo returned HTTP {}: {}",
                resp.status().as_u16(),
                resp.text().await.unwrap_or_default()
            );
        }

        let html = resp.text().await?;
        let results = Self::parse_ddg_html(&html, max_results);

        tracing::info!(
            query = %query,
            results_found = results.len(),
            "DuckDuckGo search complete"
        );

        Ok(results)
    }

    /// Parse DuckDuckGo HTML lite results.
    /// The HTML structure uses class="result__a" for links and
    /// class="result__snippet" for snippets.
    fn parse_ddg_html(html: &str, max_results: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();

        // Split by result blocks — each result is in a div with class "result"
        // We use simple string scanning since we don't want to add an HTML parser dependency
        let mut pos = 0;

        while results.len() < max_results {
            // Find the next result link
            let link_marker = "class=\"result__a\"";
            let link_pos = match html[pos..].find(link_marker) {
                Some(p) => pos + p,
                None => break,
            };

            // Extract the URL from href="..."
            let url = Self::extract_href(&html[link_pos..]);

            // Extract the title (text between > and </a>)
            let title = Self::extract_tag_text(&html[link_pos..], "result__a");

            // Find the snippet
            let snippet_marker = "class=\"result__snippet\"";
            let snippet = if let Some(sp) = html[link_pos..].find(snippet_marker) {
                Self::extract_tag_text(&html[link_pos + sp..], "result__snippet")
            } else {
                String::new()
            };

            // Skip ad results and empty results
            if !url.is_empty() && !title.is_empty() && !url.contains("duckduckgo.com/y.js") {
                // DuckDuckGo wraps URLs in a redirect — extract the actual URL
                let clean_url = Self::clean_ddg_url(&url);

                results.push(SearchResult {
                    title: Self::strip_html_tags(&title),
                    url: clean_url,
                    snippet: Self::strip_html_tags(&snippet),
                });
            }

            pos = link_pos + link_marker.len();
        }

        results
    }

    /// Extract href value from an HTML tag at the current position.
    fn extract_href(html: &str) -> String {
        if let Some(href_start) = html.find("href=\"") {
            let start = href_start + 6;
            if let Some(end) = html[start..].find('"') {
                return html[start..start + end].to_string();
            }
        }
        String::new()
    }

    /// Extract text content from a tag with a given class.
    fn extract_tag_text(html: &str, _class: &str) -> String {
        // Find the closing > of the opening tag
        if let Some(tag_end) = html.find('>') {
            let after_tag = &html[tag_end + 1..];
            // Find the next closing tag
            if let Some(close) = after_tag.find("</") {
                return after_tag[..close].trim().to_string();
            }
        }
        String::new()
    }

    /// Clean a DuckDuckGo redirect URL to get the actual destination.
    fn clean_ddg_url(url: &str) -> String {
        // DDG HTML lite uses direct URLs or //duckduckgo.com/l/?uddg=ENCODED_URL
        if url.contains("uddg=") {
            if let Some(uddg_start) = url.find("uddg=") {
                let encoded = &url[uddg_start + 5..];
                // Take until the next & or end
                let end = encoded.find('&').unwrap_or(encoded.len());
                let encoded_url = &encoded[..end];
                // URL decode
                return Self::url_decode(encoded_url);
            }
        }

        // Handle protocol-relative URLs
        if url.starts_with("//") {
            return format!("https:{}", url);
        }

        url.to_string()
    }

    /// Basic URL decoding (handles %XX sequences).
    fn url_decode(input: &str) -> String {
        let mut output = String::with_capacity(input.len());
        let mut chars = input.chars();

        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        output.push(byte as char);
                        continue;
                    }
                }
                output.push('%');
                output.push_str(&hex);
            } else if c == '+' {
                output.push(' ');
            } else {
                output.push(c);
            }
        }

        output
    }

    /// Strip basic HTML tags from text.
    fn strip_html_tags(input: &str) -> String {
        let mut output = String::with_capacity(input.len());
        let mut in_tag = false;

        for c in input.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => output.push(c),
                _ => {}
            }
        }

        // Decode common HTML entities
        output
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
            .replace("&nbsp;", " ")
            .replace("&#39;", "'")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_decode() {
        assert_eq!(
            WebSearchTool::url_decode("https%3A%2F%2Fexample.com"),
            "https://example.com"
        );
        assert_eq!(WebSearchTool::url_decode("hello+world"), "hello world");
    }

    #[test]
    fn test_strip_html_tags() {
        assert_eq!(
            WebSearchTool::strip_html_tags("<b>Hello</b> <i>world</i>"),
            "Hello world"
        );
        assert_eq!(
            WebSearchTool::strip_html_tags("plain text"),
            "plain text"
        );
    }

    #[test]
    fn test_clean_ddg_url() {
        let direct = "https://example.com/page";
        assert_eq!(WebSearchTool::clean_ddg_url(direct), "https://example.com/page");

        let redirect = "//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fpage&rut=abc";
        assert_eq!(
            WebSearchTool::clean_ddg_url(redirect),
            "https://example.com/page"
        );
    }

    #[tokio::test]
    async fn test_web_search_live() {
        let tool = WebSearchTool::default();
        let result = tool.execute(json!({"query": "Rust programming language", "max_results": 3})).await;
        // This test requires internet — may fail in CI
        if let Ok(output) = result {
            assert!(output.contains("Rust"), "Should mention Rust in results");
            println!("{}", output);
        }
    }
}
