# GZMO Extension Research — Deep Web Survey

**Date:** 2026-06-03T18:00 UTC  
**Controller:** AI Agent (pi) — full operational control  
**Method:** GitHub API + README analysis + ecosystem mapping

---

## 1. Current State

**Active MCP servers:** 1
- `memory` → `mcp-neo4j-memory@0.4.5` (stdio) — Neo4j KG, 9 tools

**Gap analysis:**
- ❌ **No web search** — GZMO has no live internet access
- ❌ **No deep research** — SparkEngine hypotheses rely only on existing knowledge
- ❌ **No browser automation** — Cannot verify live web claims
- ❌ **No code search** — Cannot search repos for context
- ❌ **No workflow orchestration** — Manual cron scheduling only
- ❌ **No memory augmentation** — SQLite-only, no cross-session semantic recall

---

## 2. Top Recommendations (by impact on GZMO)

### 2.1 🔍 Web Search & Research (Highest Impact)

| Server | Stars | Type | Install |
|--------|-------|------|---------|
| **Exa MCP** | ⭐4,521 | HTTP hosted | `https://mcp.exa.ai/mcp` (API key needed) |
| **Brave Search MCP** | ⭐1,135 | Local stdio | `uvx mcp-server-brave-search` |
| **BrightData MCP** | ⭐2,430 | HTTP hosted | `npm i @brightdata/mcp` (5K free requests/mo) |

**Why:** SparkEngine hypotheses currently have zero real-world grounding. Web search enables live fact-checking, news research, and source verification.

**Exa advantages:**
- Web search + code search + company research in one
- Hosted (no local setup)
- 4,521 stars, actively maintained (updated today)
- Supports Cursor, VS Code, Claude Desktop, MCP clients

**Brave advantages:**
- Free tier available
- Web + local + video + image + news search
- Local stdio (air-gapped friendly)
- Freshness controls (pd/pw/pm/py)

### 2.2 🧠 Memory Augmentation

| Server | Stars | Type | Install |
|--------|-------|------|---------|
| **Mem0** | ⭐57,588 | Python/Local | `pip install mem0ai` |

**Why:** Mem0 provides cross-session semantic memory that complements GZMO's SQLite vault. It learns preferences and patterns over time.

**Use case:** Enhance DreamEngine's cross-day recall by adding Mem0's persistent memory layer on top of SQLite vault.

### 2.3 🌐 Web Scraping & Data Extraction

| Server | Stars | Type | Install |
|--------|-------|------|---------|
| **Apify Actors MCP** | ⭐1,300 | HTTP hosted | `npx -y @apify/actors-mcp-server` |
| **Playwright MCP** | ⭐33,414 | Local stdio | `npx -y @anthropic/mcp-server-playwright` |

**Why:** GZMO's IngestEngine could leverage Apify's 3,000+ actors for structured web data extraction. Playwright enables browser automation for verification.

**Apify advantages:**
- 3,000+ pre-built scrapers (social media, search, maps, e-commerce)
- Cloud execution (no local resource cost)
- Ideal for periodic data collection tasks

**Playwright advantages:**
- Full browser automation (navigation, screenshots, form filling)
- 33,414 stars — Microsoft-backed
- Local execution (air-gapped friendly)

### 2.4 🔄 Agent Framework & Orchestration

| Server | Stars | Type | Install |
|--------|-------|------|---------|
| **MCP Agent Framework** | ⭐8,352 | Python | `pip install mcp-agent` |
| **MCP Proxy** | ⭐240 | Go local | `mcpproxy-go` |

**Why:** GZMO's daemon has basic cron scheduling. MCP Agent Framework provides composable workflows, durable agents, and pattern library.

**MCP Agent advantages:**
- Implements Anthropic's "Building Effective Agents" patterns
- Temporal integration for pause/resume/recovery
- Simple composable patterns vs complex cron
- Full MCP lifecycle management

### 2.5 📊 Code & Documentation

| Server | Stars | Type | Install |
|--------|-------|------|---------|
| **Context7** | ⭐56,653 | HTTP hosted | `npx -y @upstash/context7-mcp` |
| **Chroma MCP** | ⭐554 | Local stdio | `pip install chroma-mcp` |

**Why:** Context7 provides up-to-date code documentation for LLMs — useful for GZMO's code analysis and verification tasks.

---

## 3. Implementation Priority Matrix

| Priority | Extension | Impact | Effort | Cost |
|----------|-----------|--------|--------|------|
| **P0** | Brave Search MCP | ⭐⭐⭐⭐⭐ | Low | Free |
| **P0** | Exa MCP | ⭐⭐⭐⭐⭐ | Low | Free tier |
| **P1** | Playwright MCP | ⭐⭐⭐⭐ | Medium | Free |
| **P1** | Mem0 | ⭐⭐⭐⭐ | Medium | Free tier |
| **P2** | Apify Actors | ⭐⭐⭐ | Medium | Free tier |
| **P2** | MCP Agent Framework | ⭐⭐⭐ | High | Free |
| **P3** | Context7 | ⭐⭐ | Low | Free tier |
| **P3** | MCP Proxy | ⭐⭐ | Medium | Free |

---

## 4. Recommended Stack (Minimal Viable Extension)

For maximum GZMO capability improvement with minimal overhead:

```toml
# gzmo.toml — proposed additions

[[mcp_servers]]
name = "brave-search"
transport = "stdio"
command = "uvx"
args = ["mcp-server-brave-search"]
# BRAVE_SEARCH_API_KEY env var needed

[[mcp_servers]]
name = "exa"
transport = "http"
url = "https://mcp.exa.ai/mcp"
# EXA_API_KEY env var needed

[[mcp_servers]]
name = "playwright"
transport = "stdio"
command = "npx"
args = ["-y", "@anthropic/mcp-server-playwright"]
```

**Total new MCP servers:** 3  
**Estimated monthly cost:** $0 (all have generous free tiers)  
**Estimated setup time:** 30 minutes

---

## 5. Ecosystem Map

### MCP Registry
- **Registry:** https://registry.modelcontextprotocol.io — Official MCP server registry
- **Awesome Lists:**
  - https://github.com/wong2/awesome-mcp-servers (mcpservers.org)
  - https://github.com/punkpeye/awesome-mcp-servers (glama.ai/mcp/servers)
  - https://github.com/appcypher/awesome-mcp-servers

### MCP Frameworks
- **Anubis MCP** (Elixir) — High-level MCP implementation
- **FastMCP** (TypeScript) — Lightweight MCP server framework
- **MCP-Framework** (TypeScript) — CLI-based MCP creation
- **ToolHive** (Go) — Containerized MCP deployment
- **MCP Proxy** (Go) — Multi-server gateway with quarantine

### MCP Management Tools
- **mcpm** — Homebrew-like MCP server manager
- **mcp-dockmaster** — UI for managing MCP servers
- **mcp-manager** — Web UI for Claude Desktop MCP management
- **MCP Router** — Windows/macOS app for MCP management
- **MCPWatch** — Security scanner for MCP servers

---

## 6. Security Considerations

| Extension | Risk Level | Mitigation |
|-----------|-----------|------------|
| Brave Search | Low | Local stdio, no data leaves machine |
| Exa | Low | API key required, hosted (no local exposure) |
| Playwright | Medium | Local browser, sandbox execution |
| Mem0 | Low | Local storage, optional cloud sync |
| Apify | Medium | Cloud execution, review actor permissions |
| MCP Proxy | Low | Local gateway, isolated server execution |

**Recommendation:** Start with Brave Search (stdio, no cloud) and Exa (hosted, API-key only). Defer Playwright until browser automation is needed.

---

## 7. Next Steps

1. **Install Brave Search MCP** — Zero cost, immediate web search capability
2. **Add Exa API key** — Web + code search for SparkEngine grounding
3. **Evaluate Playwright** — For IngestEngine web verification
4. **Consider MCP Agent Framework** — If daemon orchestration becomes complex
5. **Set up MCP Proxy** — If managing >5 MCP servers

---

*Research conducted 2026-06-03T18:00 UTC by controller. Data sources: GitHub API, MCP official registry, awesome-mcp-servers lists.*
