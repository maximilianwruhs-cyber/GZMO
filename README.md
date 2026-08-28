# GZMO

**Sovereign overnight memory metabolism** — honeypot quality gate, supersession chains, local vault — airgap-capable on one box. Agents attach via **local stdio MCP**.

**USP:** full living Keep on one airgapped machine (local engines + Redis/Qdrant/Neo4j + daemon). Not a cloud notebook. Not Mem0. Not a public webserver. **One product** — [docs/ADR-0007-one-product-living.md](./docs/adr/ADR-0007-one-product-living.md). There is no lite SKU.

Doctrine: [docs/ADR-0004-airgap-living-usp.md](./docs/adr/ADR-0004-airgap-living-usp.md) · path: [docs/AIRGAP_LIVING.md](docs/AIRGAP_LIVING.md).

## Install

### Hero — living Keep (this machine)

From a clone (needs Docker for sidecars):

```bash
cargo build --release -p gzmo-cli
GZMO_BIN=./target/release/gzmo ./scripts/install-living-airgap.sh
```

Then point engines at `127.0.0.1`, enable **one** overnight daemon on this box, merge the printed `gzmo-living` MCP fragment. Quality: `./scripts/keep-quality-gate.sh`.

`curl | bash` → `~/.gzmo` is an **incomplete install** (client scratch), not GZMO. See [docs/PRODUCT_MCP.md](docs/PRODUCT_MCP.md).

### Verify

```bash
# Product quality (living host):
./scripts/keep-quality-gate.sh
./scripts/living-readiness-gate.sh
```

### Attach in the agent

**`gzmo-living`** → `gzmo_memory_status` / `gzmo_memory_search` ([docs/MCP_LOCAL_ATTACH.md](docs/MCP_LOCAL_ATTACH.md)). Cursor / Pi / OpenClaw are hands, not a second brain.

### Pi package (optional UX)

```bash
pi install npm:gzmo-pi   # or: pi install git:github.com/maximilianwruhs-cyber/gzmo-pi
```

In Pi: `/gzmo setup` → `/reload` → `gzmo_memory_status`.  
Package: [gzmo-pi](https://github.com/maximilianwruhs-cyber/gzmo-pi). Point it at the living writer.

### From source

```bash
cargo build --release -p gzmo-cli
GZMO_BIN=./target/release/gzmo ./scripts/install-living-airgap.sh
```

## What you get

| | |
|---|---|
| **Living vault** | On the Keep box — facts stay on disk |
| **Honeypot curation** | Quality-gated recall, not a raw dump |
| **Corpus + hybrid** | Folder searchable this sitting; vectors when local embedder is up |
| **Supersession chains** | `gzmo_memory_chain` shows how facts were replaced |
| **MCP stdio** | `gzmo-living` with Cursor and Pi |

## MCP tools

| Tool | Purpose |
|------|---------|
| `gzmo_memory_status` | Vault path, fact counts, session |
| `gzmo_memory_search` | Search honeypot / vault |
| `gzmo_memory_turn_start` | Clear session scratch |
| `gzmo_memory_recall_pull` | Pull scratch block |
| `gzmo_memory_chain` | Provenance / supersession chain |
| `gzmo_memory_profile` | Cached operator profile |
| `gzmo_wiki_search` | Wiki layer (if enabled) |

Operator-only probes (`gzmo_ops_health`, `gzmo_discovery_status`) require `GZMO_OPS_MCP=1`.

## Non-goals

- A second “lite” GZMO on the laptop ([ADR-0007](./docs/adr/ADR-0007-one-product-living.md))
- Multi-host living topologies and discovery / mentor pedagogy as install steps
- Cloud-hosted memory SaaS / public MCP (leak, not parity)
- Two overnight writers
- SEIP / Foundry-class platforms inside this product surface

## Docs

| Doc | Audience |
|-----|----------|
| [docs/AIRGAP_LIVING.md](docs/AIRGAP_LIVING.md) | Hero install |
| [docs/ADR-0007-one-product-living.md](./docs/adr/ADR-0007-one-product-living.md) | One product doctrine |
| [docs/README.md](docs/README.md) | Operator / architecture index |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Dev setup |

## License

[MIT](LICENSE)
