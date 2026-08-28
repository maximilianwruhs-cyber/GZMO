# Agent Home Secrets Policy

**Status:** Operator policy (2026-07-19)  
**Related:** [CREDENTIAL_ROTATION_CHECKLIST.md](./CREDENTIAL_ROTATION_CHECKLIST.md), [CT101_PATH_AUTHORITY.md](./ops/CT101_PATH_AUTHORITY.md), `.env.template`

## Rule

**Never store plaintext passwords, API keys, or Neo4j credentials in agent homes** (`~/.pi/agent/`, Cursor rules dumps, MEMORY_*.md, HANDOFF_*.md, `settings.json` plaintext fields).

Living secrets live only in:

| Location | Contents |
|----------|----------|
| `/opt/gzmo/.env` | CT101 living |
| Workstation `.env` (gitignored) | Lab / next |
| systemd `EnvironmentFile=` / secret managers | Production |

Agent homes may list **variable names** (`NEO4J_PASSWORD`, `OPENROUTER_API_KEY`) and how to load them — not values.

## Known scar (scrub + rotate)

`~/.pi/agent/MEMORY_REFERENCE.md` historically held plaintext Neo4j and SSH passwords. Treat as **compromised for rotation purposes** whenever that file (or a chat transcript quoting it) existed.

Actions:

1. Rotate Neo4j password on LXC101; update `/opt/gzmo/.env` and MCP env.
2. Rotate any SSH password that appeared (prefer key-only auth).
3. Replace agent-home values with `***ROTATED***` or delete the credential lines.
4. Grep agent homes after every Pi upgrade:

```bash
rg -n -i 'password|passwd|sk-or-|neo4j/|Easycheesy' ~/.pi/agent/ 2>/dev/null || true
```

## Safe agent-home content

- Paths: `/opt/gzmo/current`, `GZMO_CONFIG=/opt/gzmo/gzmo.toml`
- MCP command lines that read env from the living process
- Model aliases (no keys)
- Links into repo docs

## Unsafe

- Passwords next to `neo4j / …`
- OpenRouter / Anthropic keys in markdown
- Copy-pasted `.env` blocks into HANDOFF files
- Committing `settings.json` with embedded secrets into any git repo
