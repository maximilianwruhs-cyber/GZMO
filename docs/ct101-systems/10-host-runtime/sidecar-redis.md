# Subsystem — Sidecar Redis

**Source:** `swap/templates/database-cluster-compose.yml`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Provides an LRU-bounded Redis instance for scratch memory, distill job queues, and hot context injection. When Redis is unreachable, the daemon degrades to in-memory buffers (see `ScratchService`), but startup health probes surface the failure.

**Live (2026-07-14):** Container `sidecar-redis` up 6 days; LAN port **6379**.

---

## 2. How it works

Compose service definition:

```4:14:swap/templates/database-cluster-compose.yml
  redis:
    image: redis:7-alpine
    container_name: sidecar-redis
    restart: always
    ports:
      - "6379:6379"
    command: redis-server --maxmemory 1gb --maxmemory-policy allkeys-lru --save ""
    sysctls:
      - net.core.somaxconn=1024
    volumes:
      - redis_data:/data
```

Key design choices:
- **`maxmemory 1gb`** — caps RAM on 8 GiB CT101 host
- **`allkeys-lru`** — evicts least-recently-used keys under pressure (scratch is ephemeral)
- **`--save ""`** — disables RDB snapshots; scratch data is reconstructible

Health probe in daemon startup (`health.rs`):

```109:127:gzmo-core/src/health.rs
pub async fn probe_redis(cfg: &RedisConfig) -> ProbeResult {
    if !cfg.enabled {
        return ProbeResult::pass("redis", "disabled in config");
    }
    let client = match redis::Client::open(cfg.url.as_str()) {
        Ok(c) => c,
        Err(e) => return ProbeResult::fail("redis", format!("bad url {}: {e}", cfg.url)),
    };
    // ...
    let pong: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut conn).await;
    match pong {
        Ok(_) => ProbeResult::pass("redis", format!("PONG @ {}", cfg.url)),
        Err(e) => ProbeResult::fail("redis", format!("{} PING failed: {e}", cfg.url)),
    }
}
```

---

## 3. Interfaces

| Interface | Value |
|-----------|-------|
| Container name | `sidecar-redis` |
| Host port | `6379` (bound `0.0.0.0` on CT101) |
| CT101 URL (typical) | `redis://127.0.0.1:6379` |
| Config section | `[redis]` in `/opt/gzmo/gzmo.toml` |
| Volume | `redis_data` (local driver) |

---

## 4. THINKING nodes

> **THINKING — database-cluster-compose.yml:redis command**
> - *Reviewed:* 1 GiB cap + LRU + no persistence.
> - *Insight:* Matches scratch semantics — hot working set, not source of truth.
> - *Risk / limitation:* Distill queue file fallback exists but is slower under load.
> - *Enhancement:* Expose `maxmemory` via compose env for host-size tuning. [CT101-safe]

> **THINKING — health.rs:probe_redis**
> - *Reviewed:* 3s connect timeout; auth errors surface in PING reply.
> - *Insight:* Prevents silent in-memory degradation when Redis is misconfigured.
> - *Risk / limitation:* Non-strict startup allows daemon to run with Redis down.
> - *Enhancement:* Optional strict mode for CT101 production. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| GZMO-next workstation | Same sidecar pattern or embedded Redis |
| CT101 | Frozen — upgrade via `docker compose pull && up -d` |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Compose healthcheck `redis-cli ping` | [CT101-safe] |
| 2 | Configurable maxmemory per host RAM | [CT101-safe] |
| 3 | Redis Sentinel for HA | [GZMO-next] |
| 4 | Separate Redis instance for distill vs scratch | [GZMO-next] |
