# Pi (Coding Agent) — Extension Ecosystem Research

**Date:** 2026-06-03T18:15 UTC  
**Controller:** AI Agent (pi) — full operational control  
**Method:** GitHub API + npm registry + pi.dev package gallery + README analysis

---

## 1. Pi Architecture (Extension Surface)

Pi's extension system is the **most powerful extensibility surface** in the AI coding agent ecosystem:

| Extension Capability | What it enables |
|---------------------|-----------------|
| **Custom tools** | Register tools callable by the LLM via `pi.registerTool()` |
| **Event interception** | Block/modify tool calls, inject context, customize compaction |
| **Custom UI** | Full TUI components with keyboard input via `ctx.ui.custom()` |
| **Custom providers** | Register new LLM providers (proxies, custom APIs, OAuth) |
| **Session management** | Persist state, fork sessions, tree navigation |
| **Commands** | Register `/command` handlers with autocompletion |
| **Shortcut binding** | Register keyboard shortcuts |
| **System prompt** | Modify or append to system prompt per-turn |
| **RPC mode** | JSON-RPC for subprocess integration |

### Extension Locations
| Location | Scope |
|----------|-------|
| `~/.pi/agent/extensions/*.ts` | Global (all projects) |
| `.pi/extensions/*.ts` | Project-local |

### Pi Packages
| Source | Install |
|--------|---------|
| npm | `pi install npm:@scope/pkg@version` |
| git | `pi install git:github.com/user/repo@v1` |
| local | `pi install /path/to/package` |

---

## 2. Built-in Extensions (from examples/)

| Category | Extension | Purpose |
|----------|-----------|---------|
| **Safety** | `permission-gate.ts` | Confirm before `rm -rf`, `sudo` |
| **Safety** | `protected-paths.ts` | Block writes to `.env`, `.git/` |
| **Safety** | `dirty-repo-guard.ts` | Prevent session changes with uncommitted git |
| **Safety** | `sandbox/` | OS-level sandboxing with `@anthropic-ai/sandbox-runtime` |
| **Git** | `git-checkpoint.ts` | Git stash at each turn for code restoration |
| **Git** | `auto-commit-on-exit.ts` | Auto-commits on exit |
| **Git** | `git-merge-and-resolve.ts` | Merge workflow with conflict resolution |
| **Git** | `github-issue-autocomplete.ts` | `#1234` issue completions from `gh issue list` |
| **Tools** | `todo.ts` | Todo list tool + `/todos` command |
| **Tools** | `question.ts` | `ctx.ui.select()` for user questions |
| **Tools** | `questionnaire.ts` | Multi-question input with tab navigation |
| **Tools** | `dynamic-tools.ts` | Register tools after startup dynamically |
| **Tools** | `tool-override.ts` | Override built-in tools with logging/access control |
| **Tools** | `ssh.ts` | Delegate all tools to remote machine via SSH |
| **Tools** | `subagent/` | Delegate to specialized subagents |
| **UI** | `preset.ts` | Named presets for model/thinking/tools/instructions |
| **UI** | `plan-mode/` | Claude Code-style plan mode |
| **UI** | `handoff.ts` | Transfer context to new focused session |
| **UI** | `qna.ts` | Extract questions from last response |
| **UI** | `status-line.ts` | Turn progress in footer |
| **UI** | `custom-footer.ts` | Git branch + token stats in footer |
| **UI** | `custom-header.ts` | Custom header via `ctx.ui.setHeader()` |
| **UI** | `notify.ts` | Desktop notifications via OSC 777 |
| **UI** | `titlebar-spinner.ts` | Braille spinner while agent works |
| **UI** | `snake.ts` | Snake game with custom UI |
| **UI** | `tic-tac-toe.ts` | Tic-tac-toe vs agent |
| **UI** | `doom-overlay/` | DOOM game as overlay at 35 FPS |
| **Session** | `summarize.ts` | Summarize conversation with GPT-5.2 |
| **Session** | `custom-compaction.ts` | Custom compaction that summarizes entire conversation |
| **Session** | `trigger-compact.ts` | Auto-compact at 100k tokens |
| **Session** | `bookmark.ts` | Bookmark entries with labels |
| **Session** | `session-name.ts` | Name sessions for session selector |
| **System Prompt** | `pirate.ts` | Dynamically modify system prompt |
| **System Prompt** | `claude-rules.ts` | Scan `.claude/rules/` and list in system prompt |
| **System Prompt** | `prompt-customizer.ts` | Custom prompt customization |
| **Input** | `inline-bash.ts` | Expand `!{command}` patterns in prompts |
| **Input** | `input-transform-streaming.ts` | Skip expensive preprocessing for mid-stream steering |
| **Resources** | `dynamic-resources/` | Load skills/prompts/themes dynamically |
| **Messages** | `message-renderer.ts` | Custom message rendering with colors |
| **Messages** | `event-bus.ts` | Inter-extension communication |
| **Providers** | `custom-provider-anthropic/` | Custom Anthropic provider with OAuth |
| **Providers** | `custom-provider-gitlab-duo/` | GitLab Duo provider via proxy |

---

## 3. Pi Packages (Community Ecosystem)

### 3.1 Web Access & Search (Highest Impact)

| Package | Version | Description | Install |
|---------|---------|-------------|---------|
| **pi-web-access** | 0.10.7 | Web search, URL fetching, GitHub cloning, PDF extraction, YouTube/Video understanding. Zero-config Exa search. | `pi install npm:pi-web-access` |
| **pi-smart-fetch** | 0.3.9 | Smart web_fetch with browser TLS impersonation, defuddle extraction, batch fetch. | `pi install npm:pi-smart-fetch` |
| **pi-chrome** | 0.15.35 | Use your signed-in Chrome profile. Inspect tabs, snapshot pages, take screenshots. | `pi install npm:pi-chrome` |
| **pi-agent-browser-native** | 0.2.40 | Native `agent_browser` tool for browser automation. Persistent profiles, authenticated web apps. | `pi install npm:pi-agent-browser-native` |
| **@demigodmode/pi-web-agent** | — | Reliable web access with explicit search, fetch, and headless boundaries. | `pi install npm:@demigodmode/pi-web-agent` |
| **@ollama/pi-web-search** | — | Ollama-based local web search. | `pi install npm:@ollama/pi-web-search` |

**Why:** GZMO currently has **no web access at all**. These packages enable live search, content extraction, video understanding, and authenticated browser automation.

### 3.2 Memory & Context

| Package | Version | Description | Install |
|---------|---------|-------------|---------|
| **pi-hermes-memory** | 0.7.14 | Persistent memory + session search + secret scanning. Token-aware policy-only memory, SQLite FTS5 search, auto-consolidation. | `pi install npm:pi-hermes-memory` |
| **@samfp/pi-memory** | — | Memory extension for Pi. | `pi install npm:@samfp/pi-memory` |
| **pi-hermes-memory** | — | 🧠 Persistent memory + 🔍 session search + 🛡️ secret scanning. | `pi install npm:pi-hermes-memory` |

**Why:** GZMO's memory is SQLite-only. Pi-Hermes adds cross-session semantic recall, secret scanning, and auto-consolidation.

### 3.3 Sub-Agents & Workflow

| Package | Version | Description | Install |
|---------|---------|-------------|---------|
| **pi-subagents** | 0.27.0 | Delegate tasks to child agents. Chains, parallel execution, TUI clarification. | `pi install npm:pi-subagents` |
| **pi-crew** | 0.5.25 | Coordinated AI teams. Workflows, worktree isolation, async orchestration. 38 security audits. | `pi install npm:pi-crew` |
| **pi-workflow-engine** | 0.2.0 | Multi-agent workflow orchestration. Fan out to subagents, validated structured data between stages. | `pi install npm:pi-workflow-engine` |
| **@howaboua/pi-explore-subagents** | — | Explore subagents extension. | `pi install npm:@howaboua/pi-explore-subagents` |
| **@a5c-ai/babysitter-pi** | 0.1.3 | Babysitter/orchestration for Pi. | `pi install npm:@a5c-ai/babysitter-pi` |

**Why:** GZMO's daemon runs single-threaded. Sub-agents enable parallel code review, scouting, implementation, and audit.

### 3.4 MCP Integration

| Package | Version | Description | Install |
|---------|---------|-------------|---------|
| **pi-mcp-adapter** | 2.8.0 | MCP (Model Context Protocol) adapter. Token-efficient. Connects MCP servers to pi. | `pi install npm:pi-mcp-adapter` |
| **@victor-software-house/pi-acp** | — | ACP (Agent Client Protocol) adapter for pi. | `pi install npm:@victor-software-house/pi-acp` |
| **pi-acp** | 0.0.27 | ACP adapter for pi coding agent. | `pi install npm:pi-acp` |

**Why:** GZMO uses MCP for Neo4j. pi-mcp-adapter enables GZMO's MCP servers to be accessible from pi.

### 3.5 UI & Productivity

| Package | Version | Description | Install |
|---------|---------|-------------|---------|
| **pi-studio** | 0.9.25 | Two-pane browser workspace. Prompt/response editing, annotations, critiques, quiz, live previews, tmux integration. | `pi install npm:pi-studio` |
| **pi-lens** | 3.8.47 | Real-time code feedback: LSP, linters, formatters, type-checking, structural analysis. | `pi install npm:pi-lens` |
| **pi-ask-user** | 0.11.1 | Interactive `ask_user` tool with searchable split-pane selection, multi-select, freeform input. | `pi install npm:pi-ask-user` |
| **pi-powerline-footer** | 0.5.6 | Powerline-style status bar with git branch, tokens, etc. | `pi install npm:pi-powerline-footer` |
| **pi-btw** | 0.4.0 | Parallel side conversations with `/btw`. | `pi install npm:pi-btw` |
| **pi-intercom** | 0.6.0 | Intercom extension for pi. | `pi install npm:pi-intercom` |
| **pi-markdown-preview** | 0.10.0 | Rendered markdown + LaTeX preview. Terminal, browser, and PDF output. | `pi install npm:pi-markdown-preview` |
| **pi-agent-flow** | 2.3.5 | Flow-state transition extension. | `pi install npm:pi-agent-flow` |

### 3.6 Security & Policy

| Package | Version | Description | Install |
|---------|---------|-------------|---------|
| **@gotgenes/pi-permission-system** | 10.0.0 | Permission enforcement. Policy, access control, authorization. | `pi install npm:@gotgenes/pi-permission-system` |
| **pi-lean-ctx** | 3.7.1 | Routes bash/read/grep/find/ls through lean-ctx CLI for token savings. Optional MCP bridge. | `pi install npm:pi-lean-ctx` |
| **@aliou/pi-guardrails** | — | Guardrails extension for pi. | `pi install npm:@aliou/pi-guardrails` |

### 3.7 Specialized / Niche

| Package | Version | Description | Install |
|---------|---------|-------------|---------|
| **pi-crew** | 0.5.25 | Coordinated AI teams with worktree isolation. | `pi install npm:pi-crew` |
| **@firstpick/pi-extension-todo-progress** | — | Todo progress extension. | `pi install npm:@firstpick/pi-extension-todo-progress` |
| **@juicesharp/rpiv-web-tools** | — | Web tools for rpiv. | `pi install npm:@juicesharp/rpiv-web-tools` |
| **@juicesharp/rpiv-ask-user-question** | — | Ask user question tool. | `pi install npm:@juicesharp/rpiv-ask-user-question` |
| **@juicesharp/rpiv-todo** | — | Todo tool. | `pi install npm:@juicesharp/rpiv-todo` |
| **@juicesharp/rpiv-advisor** | — | Advisor tool. | `pi install npm:@juicesharp/rpiv-advisor` |
| **@juicesharp/rpiv-i18n** | — | i18n support. | `pi install npm:@juicesharp/rpiv-i18n` |
| **@juicesharp/rpiv-args** | — | Args handling. | `pi install npm:@juicesharp/rpiv-args` |
| **@juicesharp/rpiv-btw** | — | Btw side conversations. | `pi install npm:@juicesharp/rpiv-btw` |
| **@juicesharp/rpiv-pi** | — | General rpiv tools. | `pi install npm:@juicesharp/rpiv-pi` |
| **@llblab/pi-actors** | — | Actor system for pi. | `pi install npm:@llblab/pi-actors` |
| **@llblab/pi-telegram** | — | Telegram integration. | `pi install npm:@llblab/pi-telegram` |
| **@tintinweb/pi-subagents** | — | Subagent extension. | `pi install npm:@tintinweb/pi-subagents` |
| **@nitra/cursor** | — | Cursor integration. | `pi install npm:@nitra/cursor` |
| **@plannotator/pi-extension** | — | Planner extension. | `pi install npm:@plannotator/pi-extension` |
| **@raindrop-ai/pi-agent** | 0.0.4 | Raindrop observability. Automatic tracing via subscriber or extension. | `pi install npm:@raindrop-ai/pi-agent` |
| **@runfusion/fusion** | — | Fusion extension. | `pi install npm:@runfusion/fusion` |
| **@undefineds.co/models** | — | Models management. | `pi install npm:@undefineds.co/models` |
| **@vigolium/piolium** | — | Piolium extension. | `pi install npm:@vigolium/piolium` |
| **@howaboua/pi-codex-conversion** | — | Codex conversion. | `pi install npm:@howaboua/pi-codex-conversion` |
| **@howaboua/pi-extensions** | — | Additional extensions. | `pi install npm:@howaboua/pi-extensions` |
| **@gonrocca/zero-pi** | — | Zero-pi. | `pi install npm:@gonrocca/zero-pi` |
| **@ff-labs/pi-fff** | — | FFF extension. | `pi install npm:@ff-labs/pi-fff` |
| **gentle-engram** | — | Engram extension. | `pi install npm:gentle-engram` |
| **gentle-pi** | — | Gentle-pi. | `pi install npm:gentle-pi` |
| **glimpseui** | — | Glimpse UI. | `pi install npm:glimpseui` |
| **pi-hermes-memory** | 0.7.14 | Persistent memory + session search + secret scanning. | `pi install npm:pi-hermes-memory` |
| **pi-lean-ctx** | 3.7.1 | Token-saving CLI routing. | `pi install npm:pi-lean-ctx` |
| **pi-markdown-preview** | 0.10.0 | Markdown + LaTeX preview. | `pi install npm:pi-markdown-preview` |
| **pi-smart-fetch** | 0.3.9 | Smart web fetch with TLS impersonation. | `pi install npm:pi-smart-fetch` |
| **pi-studio** | 0.9.25 | Two-pane browser workspace. | `pi install npm:pi-studio` |
| **pi-subagents** | 0.27.0 | Sub-agent delegation. | `pi install npm:pi-subagents` |
| **pi-web-access** | 0.10.7 | Web search, fetching, video understanding. | `pi install npm:pi-web-access` |
| **pi-workflow-engine** | 0.2.0 | Multi-agent workflow orchestration. | `pi install npm:pi-workflow-engine` |

---

## 4. Ecosystem Map

### Official Packages (earendil-works)

| Package | Stars | Purpose |
|---------|-------|---------|
| **pi** | ⭐59,329 | Main repository: coding agent CLI, unified LLM API, TUI & web UI, Slack bot, vLLM pods |
| **gondolin** | ⭐1,321 | Experimental Linux microvm with TypeScript Control Plane as Agent Sandbox |
| **absurd** | ⭐1,989 | Experiment in durability |

### Key Pi Packages (npm)

| Package | Stars | Purpose |
|---------|-------|---------|
| **pi-mcp-adapter** | ⭐817 | MCP adapter for pi (token-efficient) |
| **pi-coding-agent** | — | Main pi coding agent package |
| **pi-acp** | — | ACP (Agent Client Protocol) adapter |
| **pi-mcp-adapter** | ⭐817 | Token-efficient MCP adapter |
| **@a5c-ai/babysitter-pi** | — | Orchestration babysitter |
| **pi-agent-flow** | — | Flow-state transition |
| **@gotgenes/pi-permission-system** | — | Permission enforcement |
| **@remnic/plugin-pi** | — | Remnic memory extension |
| **pi-powerline-footer** | — | Powerline status bar |
| **@agentuity/coder-tui** | — | Agentuity Coder Hub |
| **@agentuity/pi** | — | Agentuity AI Gateway provider |
| **@whonixnetworks/pi-mattermost** | — | Mattermost bridge |
| **pi-debug** | — | Debug extension |
| **pi-pipeline** | — | Pipeline extension |
| **@vtstech/pi-shared** | — | Shared utilities |
| **@agegr/pi-web** | — | Web UI for pi |
| **@victor-software-house/pi-acp** | — | ACP adapter |
| **@raindrop-ai/pi-agent** | — | Raindrop observability |
| **pi-langsrv** | — | Language server |
| **pi-scheduler-ext** | — | Scheduler extension |

### Package Gallery (pi.dev)

**50+ packages** listed on [pi.dev/packages](https://pi.dev/packages) including:
- `@a5c-ai/babysitter-pi`
- `@aliou/pi-guardrails`
- `@firstpick/pi-extension-todo-progress`
- `@firstpick/pi-package-webui`
- `@gotgenes/pi-permission-system`
- `@gotgenes/pi-subagents`
- `@howaboua/pi-codex-conversion`
- `@howaboua/pi-explore-subagents`
- `@howaboua/pi-extensions`
- `@juicesharp/rpiv-advisor`
- `@juicesharp/rpiv-args`
- `@juicesharp/rpiv-ask-user-question`
- `@juicesharp/rpiv-btw`
- `@juicesharp/rpiv-i18n`
- `@juicesharp/rpiv-pi`
- `@juicesharp/rpiv-todo`
- `@juicesharp/rpiv-web-tools`
- `@llblab/pi-actors`
- `@llblab/pi-telegram`
- `@nitra/cursor`
- `@ollama/pi-web-search`
- `@plannotator/pi-extension`
- `@raindrop-ai/pi-agent`
- `@runfusion/fusion`
- `@samfp/pi-memory`
- `@syntesseraai/pi-feature-factory`
- `@tintinweb/pi-subagents`
- `@undefineds.co/models`
- `@vigolium/piolium`
- `context-mode`
- `gentle-engram`
- `gentle-pi`
- `glimpseui`
- `gonrocca/zero-pi`
- `pi-agent-browser-native`
- `pi-agent-flow`
- `pi-ask-user`
- `pi-btw`
- `pi-chrome`
- `pi-crew`
- `pi-hermes-memory`
- `pi-intercom`
- `pi-lean-ctx`
- `pi-lens`
- `pi-markdown-preview`
- `pi-mcp-adapter`
- `pi-monofold`
- `pi-powerline-footer`
- `pi-simplify`
- `pi-smart-fetch`
- `pi-studio`
- `pi-subagents`
- `pi-web-access`
- `pi-workflow-engine`

---

## 5. Recommendation Summary (Pi Extensions for GZMO Integration)

### P0 — Critical Gaps to Fill

| Extension | Why | Install |
|-----------|-----|---------|
| **pi-web-access** | Live web search, YouTube/video understanding, GitHub cloning | `pi install npm:pi-web-access` |
| **pi-hermes-memory** | Cross-session memory + secret scanning + FTS5 search | `pi install npm:pi-hermes-memory` |
| **pi-mcp-adapter** | Connect GZMO's MCP servers to pi | `pi install npm:pi-mcp-adapter` |

### P1 — Major Capability Boosts

| Extension | Why | Install |
|-----------|-----|---------|
| **pi-subagents** | Parallel code review, scouting, audits | `pi install npm:pi-subagents` |
| **pi-chrome** | Use signed-in Chrome profile for authenticated browsing | `pi install npm:pi-chrome` |
| **pi-workflow-engine** | Multi-agent workflow orchestration | `pi install npm:pi-workflow-engine` |
| **pi-smart-fetch** | Browser-like TLS impersonation for web fetching | `pi install npm:pi-smart-fetch` |
| **pi-ask-user** | Interactive user questions with split-pane UI | `pi install npm:pi-ask-user` |

### P2 — Productivity & UX

| Extension | Why | Install |
|-----------|-----|---------|
| **pi-studio** | Two-pane browser workspace with annotations | `pi install npm:pi-studio` |
| **pi-lens** | Real-time LSP/linter/type-checking feedback | `pi install npm:pi-lens` |
| **pi-powerline-footer** | Powerline status bar | `pi install npm:pi-powerline-footer` |
| **pi-btw** | Parallel side conversations | `pi install npm:pi-btw` |
| **pi-markdown-preview** | Markdown/LaTeX preview | `pi install npm:pi-markdown-preview` |

### P3 — Specialized / Niche

| Extension | Why | Install |
|-----------|-----|---------|
| **@gotgenes/pi-permission-system** | Permission enforcement for security | `pi install npm:@gotgenes/pi-permission-system` |
| **pi-agent-browser-native** | Native browser automation tool | `pi install npm:pi-agent-browser-native` |
| **@raindrop-ai/pi-agent** | Observability & tracing | `pi install npm:@raindrop-ai/pi-agent` |
| **pi-lean-ctx** | Token-saving CLI routing | `pi install npm:pi-lean-ctx` |

---

## 6. Implementation Priority Matrix

| Priority | Extension | Impact | Effort | Cost | Notes |
|----------|-----------|--------|--------|------|-------|
| **P0** | pi-web-access | ⭐⭐⭐⭐⭐ | Low | Free | Zero-config Exa, immediate web search |
| **P0** | pi-hermes-memory | ⭐⭐⭐⭐⭐ | Low | Free | Cross-session recall, secret scanning |
| **P0** | pi-mcp-adapter | ⭐⭐⭐⭐⭐ | Low | Free | Connect GZMO's MCP to pi |
| **P1** | pi-subagents | ⭐⭐⭐⭐ | Medium | Free | Parallel code review, audits |
| **P1** | pi-chrome | ⭐⭐⭐⭐ | Medium | Free | Authenticated browser access |
| **P1** | pi-workflow-engine | ⭐⭐⭐⭐ | Medium | Free | Multi-agent orchestration |
| **P1** | pi-smart-fetch | ⭐⭐⭐ | Low | Free | Better web fetching |
| **P1** | pi-ask-user | ⭐⭐⭐ | Low | Free | Interactive Q&A |
| **P2** | pi-studio | ⭐⭐⭐ | Medium | Free | Two-pane workspace |
| **P2** | pi-lens | ⭐⭐⭐ | Low | Free | Real-time code feedback |
| **P2** | pi-powerline-footer | ⭐⭐ | Low | Free | Status bar polish |
| **P2** | pi-btw | ⭐⭐ | Low | Free | Side conversations |
| **P2** | pi-markdown-preview | ⭐⭐ | Low | Free | Markdown/LaTeX preview |

---

## 7. Recommended Stack (Minimal Viable Extension)

For maximum capability improvement with minimal overhead:

```bash
# P0 — Install these first
pi install npm:pi-web-access
pi install npm:pi-hermes-memory
pi install npm:pi-mcp-adapter

# P1 — Add when ready
pi install npm:pi-subagents
pi install npm:pi-chrome
pi install npm:pi-workflow-engine

# P2 — Polish
pi install npm:pi-lens
pi install npm:pi-powerline-footer
```

**Total new packages:** 3 (P0) + 3 (P1) + 2 (P2) = 8 packages
**Estimated monthly cost:** $0 (all free)
**Estimated setup time:** 15 minutes (P0 only)

---

## 8. Cross-Reference: Pi + GZMO Integration

### How Pi Extensions Enhance GZMO

| GZMO Component | Pi Extension | Enhancement |
|---------------|--------------|-------------|
| SparkEngine (hypothesis generation) | pi-web-access | Live web grounding for hypotheses |
| DreamEngine (dreaming) | pi-hermes-memory | Cross-session recall for better dreams |
| IngestEngine (web scraping) | pi-chrome + pi-smart-fetch | Authenticated browsing + TLS impersonation |
| Session Distill | pi-subagents | Parallel distillation of multiple sessions |
| Qdrant Sync | pi-mcp-adapter | Connect GZMO's Neo4j MCP to pi |
| Daemon (cron scheduling) | pi-workflow-engine | Multi-agent orchestration replaces cron |
| Knowledge Base | pi-hermes-memory | FTS5 search on top of semantic vault |
| Security | @gotgenes/pi-permission-system | Permission enforcement for dangerous ops |

### Architecture Integration

```
┌─────────────────────────────────────────────────────────┐
│                        Pi CLI                            │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │ pi-web-access│  │pi-hermes-    │  │ pi-mcp-adapter│ │
│  │ (web search) │  │memory        │  │ (MCP bridge)  │ │
│  └──────────────┘  └──────────────┘  └───────────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │pi-subagents  │  │pi-chrome     │  │pi-workflow    │ │
│  │(parallel     │  │(browser      │  │engine         │ │
│  │ review)      │  │ automation)  │  │(orchestration)│ │
│  └──────────────┘  └──────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────┘
                          │
                    pi-mcp-adapter
                          │
┌─────────────────────────────────────────────────────────┐
│                     GZMO Daemon                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │ SparkEngine  │  │ DreamEngine  │  │ IngestEngine  │ │
│  └──────────────┘  └──────────────┘  └───────────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │SessionDistill│  │Qdrant Sync   │  │ sys_janitor   │ │
│  └──────────────┘  └──────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────┘
                          │
                    ┌───────────┐
                    │ Neo4j MCP │
                    └───────────┘
```

---

## 9. Next Steps

1. **Install P0 extensions** (pi-web-access, pi-hermes-memory, pi-mcp-adapter)
2. **Verify MCP bridge** — test GZMO Neo4j MCP access from pi
3. **Evaluate P1 extensions** — pi-subagents for parallel review, pi-chrome for authenticated browsing
4. **Build custom extension** — consider building a GZMO-specific extension that wraps the daemon's cron engines
5. **Set up observability** — pi-hermes-memory auto-indexes, consider pi-agent for tracing

---

*Research conducted 2026-06-03T18:15 UTC by controller. Data sources: GitHub API, npm registry, pi.dev package gallery, README analysis.*