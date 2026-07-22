# Wiki / Observatory demo (theater)

**Status:** Demable Unpark Wave 4.3 — **not** living GREEN, **not** auto wiki push  
**Front door:** `bash scripts/wiki-observatory-demo.sh`  
**Mind notes:** [WIKI_OBSERVATORY_MIND.md](./WIKI_OBSERVATORY_MIND.md) · layer: [WIKI_LAYER.md](./WIKI_LAYER.md)

## What you feel

Seeded **wiki search** (git-tracked `wiki/`) plus a **sanitized nightburst scoreboard** (metabolism · wiki sha · Arena · gate · HSP · €/night). Local HTML stands in for the public mind; OKForge `/observatory` stays agent-discovery.

```bash
bash scripts/wiki-observatory-demo.sh
bash scripts/wiki-observatory-check.sh
# open data-next/arena/scoreboard.html
# optional: http://127.0.0.1:3000/observatory  (if okforge.service is up)
```

## Chain

| Step | Script | Output |
|------|--------|--------|
| Search | `wiki-mind-check.sh` | `data-next/wiki-mind/` (seeded `Lint` hits) |
| Scoreboard | `nightburst-scoreboard.sh` | `data-next/arena/scoreboard.{json,html}` |
| Dashboard | `aos-poll-dashboard.sh` | soft AOS poll JSON |
| Felt | (demo) | `data-next/wiki-observatory/felt-latest.md` |

## Hard rules

1. Scoreboard is sanitized — no tokens, no session bodies.  
2. `wiki-push-gated.sh` is a **separate** operator path (concept-gate PASS required).  
3. Never wire wiki-mind / observatory theater into living-readiness overnight GREEN.  
4. OKForge Observatory URL is a link only — this demo does not start forge services.

## Artifacts

- `data-next/wiki-observatory/demo.json` — chain inventory  
- `data-next/wiki-observatory/felt-latest.md` — search hits + scoreboard pills  
- `data-next/wiki-observatory/latest.json` — check verdict  
- `data-next/arena/scoreboard.html` — open in a browser  

See [UNPARK_ROADMAP.md](./UNPARK_ROADMAP.md) Wave 4 · [STACK_OPPORTUNITY_MAP.md](./STACK_OPPORTUNITY_MAP.md) Observatory as public mind.
