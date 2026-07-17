# Agent Skills landscape — most valuable available skills (Jul 2026)

Research date: 2026-07-17  
Scope: open **Agent Skills** (`SKILL.md` standard) for Claude Code / Cursor / Codex — not GZMO pantheon `/joke`-style skills.

## Sources

| Source | What it measures | URL |
|--------|------------------|-----|
| Skillselion State of AI Agent Skills 2026 | Catalog size + most-installed skills (skills.sh + GitHub) | https://skillselion.com/state-of-ai-agent-skills-2026 |
| Skillselion Top 100 | Install-ranked leaderboard (refreshed ~2026-07-16) | https://skillselion.com/leaderboard |
| mattpocock/skills | Engineering skill pack (primary source) | https://github.com/mattpocock/skills |
| Firecrawl “Best Claude Code Skills 2026” | Curated practitioner shortlist | https://firecrawl.dev/blog/best-claude-code-skills |
| DEV / AWS “most popular AI coding skills” | Narrative comparison of packs | https://dev.to/aws/the-most-popular-ai-coding-skills-right-now-4183 |

Install/star numbers below are Skillselion’s as of mid-July 2026; they move daily.

## Ecosystem snapshot

- ~60k agent skills, ~8.4k MCP servers, ~10k marketplaces; ~130M combined installs tracked.
- Distribution hubs: [skills.sh](https://skills.sh) / `npx skills`, Anthropic marketplace, curated packs (Superpowers, mattpocock/skills).
- Two useful skill types (Nate Herk / Firecrawl framing):
  - **Capability uplift** — agent couldn’t do it reliably before (browser control, PDF tooling, scrape CLI).
  - **Encoded preference** — agent already “knows,” but you force a better process (grill, TDD, deep modules, anti-slop UI).

## Tier A — highest leverage (install / consensus / daily use)

These show up across install leaderboards *and* independent “best of” lists.

| Skill | Author | Why it’s valuable | Approx. installs |
|-------|--------|-------------------|------------------|
| **find-skills** | vercel-labs | Meta: discovers/installs the right skill mid-task | ~2M |
| **frontend-design** | anthropics | Anti–AI-slop UI; distinctive production UI | ~657k |
| **vercel-react-best-practices** | vercel-labs | React/Next performance rules during gen/review | ~547k |
| **agent-browser** | vercel-labs | High-fidelity browser/Electron control for agents | ~538k |
| **grill-me** | mattpocock | Adversarial plan interview before code | ~532k |
| **web-design-guidelines** | vercel-labs | A11y/UX audit against Vercel interface guidelines | ~459k |
| **grill-with-docs** | mattpocock | Grill + domain docs / ADRs / shared language | ~447k |

Also Tier A by **starred marketplace / workflow pack** (not a single skill):

| Pack | Why |
|------|-----|
| **obra/superpowers** (marketplace ~255k★) | Full multi-step: plan → TDD → review; heavier than Pocock’s composable style |
| **anthropics** official skills (frontend-design, document skills, webapp-testing, code-simplifier, skill-creator) | First-party, progressive disclosure, low trust friction |

## Tier B — Matt Pocock engineering core (quality over hype)

From [mattpocock/skills](https://github.com/mattpocock/skills) (~175k★ repo). Stance: small composable skills vs process frameworks that “own” the loop (contrast GSD / BMAD / Spec-Kit).

**User-invoked (orchestrators)** — call when you want a workflow:

- `ask-matt` — router over the pack  
- `grill-me` / `grill-with-docs` / `grilling`  
- `handoff` / `claude-handoff` — compress session for next agent  
- `to-issues` / `to-prd` / triage / wayfinder / prototype  
- `improve-codebase-architecture`  
- `setup-matt-pocock-skills`

**Model-invoked (discipline)** — agent should pull in when relevant:

- `tdd`  
- `diagnosing-bugs`  
- `domain-modeling`  
- `codebase-design`  
- `code-review`

Practitioner consensus (Medium / explainx / DEV): **grill-with-docs + handoff** is the highest-ROI pair for multi-session work; **tdd + diagnosing-bugs + codebase-design** for day-to-day engineering.

## Tier C — high value when the job matches

| Skill / pack | When it’s worth it |
|--------------|--------------------|
| **Firecrawl** skill + CLI | Research / scrape / JS-heavy web without drowning context |
| **karpathy-guidelines** | Prefer simplicity, surgical diffs, think-before-code |
| **caveman** | Cut narration tokens (~65% claimed) while keeping facts |
| **Document skills** (Anthropic) | Real PDF/DOCX/XLSX/PPTX create/parse |
| **webapp-testing** (Playwright) | Local UI verification loops |
| **Trail of Bits** security (CodeQL/Semgrep) | Vulnerability-focused review |
| **Remotion best-practices** | Programmatic React video |
| **HashiCorp agent-skills** | Terraform modules/tests |
| **AWS Agent Toolkit skills** | Deploy/serverless/DB on AWS |
| **Microsoft Azure / Foundry** skills | High installs; only valuable if you ship on Azure |
| **Lark/Feishu** skills | High installs; enterprise China/Asia collab — skip unless you use Lark |

## Most-searched (Skillselion signal, not just installs)

`find-skills`, `caveman`, `frontend-design`, `brainstorming`, `systematic-debugging`, `stop-slop`, `emil-design-eng`, `ui-ux-pro-max`, `karpathy-guidelines`, `web-design-guidelines`, `react-best-practices`, `grill-me`, `microsoft-foundry`, `agent-browser`

## How to choose (practical filter)

1. **Don’t mass-install.** Vague/huge `SKILL.md` files burn context and misfire. Prefer one job per skill; lean body + fat references.
2. **Trust first:** Anthropic, Vercel Labs, Matt Pocock, HashiCorp, Trail of Bits, Remotion. Read scripts before random GitHub skills.
3. **Install by failure mode:**
   - Agent codes before understanding → `grill-me` / `grill-with-docs`
   - Context rot across sessions → `handoff`
   - Spaghetti modules → `codebase-design` + `improve-codebase-architecture`
   - Flaky “fix” without evidence → `tdd` / `diagnosing-bugs`
   - Generic UI → `frontend-design` (+ Vercel guidelines if React)
   - Needs real browser → `agent-browser` or Playwright webapp-testing
   - Don’t know what skill exists → `find-skills`
4. **Skip install-inflated stacks you don’t use** (Azure, Lark) even if they dominate raw leaderboards.

## Relevance to this machine (GZMO / local install check)

Already present under `~/.claude/skills` and/or `~/.agents/skills` (Matt + writing/research pack):  
`ask-matt`, `grill-me`, `grill-with-docs`, `handoff`, `tdd`, `diagnosing-bugs`, `codebase-design`, `domain-modeling`, `code-review`, `improve-codebase-architecture`, `prototype`, `triage`, `wayfinder`, `research`, etc.

**Highest-value gaps for a Rust/sovereign-agent codebase** (frontend skills less critical unless you’re shipping UI):

| Gap | Why |
|-----|-----|
| `find-skills` (vercel-labs) | Discovery without hunting blogs |
| `karpathy-guidelines` | Matches “minimize scope / surgical diffs” culture |
| `caveman` (optional) | Cheaper verbose sessions |
| Security pack (Trail of Bits / Cursor `review-security`) | Already partly covered by Cursor team-kit |
| `agent-browser` or Playwright testing | Only if you add web surfaces |
| Firecrawl | Only if research/scrape becomes frequent |

**Not a gap:** installing another full process framework (Superpowers) on top of Pocock — overlapping TDD/plan loops; pick one orchestration style.

## Install pointers

```bash
# Vercel skills CLI (common path)
npx skills@latest add mattpocock/skills
npx skills@latest add vercel-labs/find-skills   # package name may vary; use find-skills via skills.sh

# Claude Code marketplace
# /plugin marketplace add anthropics/skills
```

Always open `SKILL.md` + any bundled scripts before first use.

## Bottom line

**Most valuable skills right now are not niche domain packs — they are meta-discovery (`find-skills`), plan discipline (`grill-*`, `handoff`), engineering loops (`tdd`, `diagnosing-bugs`, `codebase-design`), and quality rails (`frontend-design`, Vercel React/web guidelines, `agent-browser`).** Vendor blobs (Azure, Lark) win install charts but are low value unless that platform is your day job.
