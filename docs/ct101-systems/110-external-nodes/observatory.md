# GZMO Observatory — Read-Only Telemetry Dashboard

**Source:** `gzmo-observatory/observatory/*.py`  
**Parent:** [110-external-nodes/SYSTEM.md](./SYSTEM.md)

---

## Capability

Workstation-hosted **FastAPI + WebSocket** dashboard polling CT101 state every 5 seconds: chaos trajectory, vault/honeypot counts, synapse tail, dreams excerpt, Obolus ledger, Qdrant/Neo4j LAN stats, and local Prime GPU. **Read-only** — failures do not affect the daemon.

---

## How it works

### Application loop

```46:53:home/gzmo/gzmo-observatory/observatory/main.py
async def poll_loop() -> None:
    while True:
        try:
            payload = await asyncio.to_thread(collector.collect)
            await broadcast(payload)
        except Exception as exc:
            await broadcast({"error": str(exc), "timestamp": ""})
        await asyncio.sleep(POLL_INTERVAL_S)
```

WebSocket clients receive JSON snapshots; REST `/api/snapshot` returns latest bundle.

### Collector bundle

```43:54:home/gzmo/gzmo-observatory/observatory/collector.py
    def collect(self) -> dict[str, Any]:
        ts = datetime.now(timezone.utc).isoformat()
        bundle = self._fetch_ct101_bundle()
        chaos = bundle.get("chaos")
        heartbeat = self._parse_heartbeat(bundle.get("heartbeat_raw", ""))
        dreams = bundle.get("dreams", [])
        synapse = self._normalize_synapse(bundle.get("synapse", {}))
        memory = bundle.get("memory", {})
        obolus = bundle.get("obolus", {})
        memory = self._enrich_memory(memory)
        prime = self._fetch_prime()
```

CT101 data fetched via `scripts/ct101-snapshot.py` — single SSH round-trip through PVE.

### Remote on-demand queries

```15:29:home/gzmo/gzmo-observatory/observatory/remote.py
def run_remote_py(script: str, args: dict[str, Any] | None = None, timeout: float = 25.0) -> Any:
    """Run a python script inside CT101, passing args as base64 JSON on argv[1]."""
    payload = base64.b64encode(json.dumps(args or {}).encode()).decode()
    remote = f"pct exec 101 -- python3 - {payload}"
    result = subprocess.run(
        [*PVE_SSH, remote],
        input=script,
        // ...
    )
```

TTL-cached vault search, dreams deep-read, wiki browse — no persistent agent on CT101.

### Config

```12:18:home/gzmo/gzmo-observatory/observatory/config.py
PVE_SSH = ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", "pve"]
GZMO_BIN = "/opt/gzmo/survey_GZMO/target/release/gzmo"
GZMO_DATA = "/opt/gzmo/data"
QDRANT_URL = "http://192.168.31.202:6333"
NEO4J_HTTP = "http://192.168.31.202:7474"
PRIME_URL = "http://127.0.0.1:8000"
```

---

## Interfaces

| Interface | Value |
|-----------|-------|
| HTTP port | `7777` (workstation) |
| Poll interval | 5 s |
| SSH path | `ssh pve` → `pct exec 101` |
| Qdrant | Direct LAN to CT101 `:6333` |
| Neo4j | Direct LAN to CT101 `:7474` |
| Séance API | `POST /api/seance` — mentor ping/teach via SSH |
| Static UI | `static/index.html`, JS panels (mind, vault, synapse, dreams, obolus) |

---

## THINKING nodes

> **THINKING — observatory:SSH single point**
> - *Reviewed:* All CT101 bundle data transits PVE SSH batch mode.
> - *Insight:* One round-trip minimizes latency vs multiple execs.
> - *Risk / limitation:* SSH failure → entire dashboard shows error blob — no partial degrade.
> - *Enhancement:* Per-subsystem fetch with last-good cache timestamps. [CT101-safe]

> **THINKING — observatory:LAN sidecar reads**
> - *Reviewed:* Qdrant/Neo4j queried directly from workstation to CT101 Docker ports.
> - *Insight:* Bypasses SSH for vector/graph counts — faster enrichment.
> - *Risk / limitation:* Assumes flat LAN trust; no TLS on sidecar HTTP.
> - *Enhancement:* Read-only API tokens when sidecars exposed beyond LAN. [GZMO-next]

> **THINKING — observatory:remote.py TTL cache**
> - *Reviewed:* On-demand vault/wiki queries cached with TTL + thread lock.
> - *Insight:* Protects CT101 SQLite from hammering on UI search typing.
> - *Risk / limitation:* Stale search results up to TTL after vault writes.
> - *Enhancement:* Cache bust webhook from daemon on major vault sync. [GZMO-next]

---

## Advancement

| CT101 | GZMO-next |
|-------|-----------|
| SSH snapshot of legacy daemon | Poll GZMO-next metrics endpoint or shared Redis |
| Workstation-only UI | Cloud-hosted Observatory read replica |
| Manual `ct101-snapshot.py` | Structured health gRPC from future lab daemon |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Partial degrade when SSH fails (show last snapshot age) | [CT101-safe] |
| 2 | Discovery cycle status panel from `auto-triggers.jsonl` | [CT101-safe] |
| 3 | LAN Prime health alongside loopback | [CT101-safe] |
| 4 | GZMO-next instance switch in config | [GZMO-next] |
| 5 | Auth gate on `:7777` if exposed beyond localhost | [GZMO-next] |
