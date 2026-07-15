# Subsystem — Sidecar Qdrant

**Source:** `swap/templates/database-cluster-compose.yml`, `gzmo-cli/src/daemon_cmd.rs` (sync loop)  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Hosts a Qdrant vector database mirroring honeypot vault facts for hybrid semantic recall. Daily cron sync pushes SQLite vault embeddings to the `honeypot` collection after the dream window.

**Live (2026-07-14):** Container `sidecar-qdrant` up 6 days; **24,322** points; health green; ports **6333** (HTTP), **6334** (gRPC).

---

## 2. How it works

Compose service:

```16:26:swap/templates/database-cluster-compose.yml
  qdrant:
    image: qdrant/qdrant:latest
    container_name: sidecar-qdrant
    restart: always
    ports:
      - "6333:6333"
      - "6334:6334"
    environment:
      - QDRANT__SERVICE__GRPC_PORT=6334
    volumes:
      - qdrant_data:/qdrant/storage
```

Daemon daily sync loop (default 01:45 UTC):

```462:491:gzmo-cli/src/daemon_cmd.rs
    // Qdrant mirror — daily sync after dream window (default 01:45 UTC)
    let qdrant_cfg = config.qdrant.clone();
    let vault_db_path = config.memory.vault_db.clone();
    let project_root = qdrant_sync::discover_project_root();
    let qdrant_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_sync_date: Option<NaiveDate> = None;
        loop {
            interval.tick().await;
            if !qdrant_cfg.sync_enabled {
                continue;
            }
            // ... cron_due_today check ...
            if let Err(e) = sync_vault_to_qdrant(&project_root, &qdrant_cfg, &vault_db_path).await {
                error!("Qdrant vault sync failed: {e}");
            } else {
                last_sync_date = Some(today);
                info!("Qdrant vault sync complete");
            }
        }
    });
```

Health probe:

```136:166:gzmo-core/src/health.rs
pub async fn probe_qdrant(cfg: &QdrantConfig) -> ProbeResult {
    if !cfg.enabled {
        return ProbeResult::pass("qdrant", "disabled in config");
    }
    let base = cfg.url.trim_end_matches('/');
    let url = format!("{base}/collections/{}", cfg.collection);
    // GET collection info → points_count, status
}
```

---

## 3. Interfaces

| Interface | Value |
|-----------|-------|
| HTTP API | `http://192.168.31.202:6333` |
| gRPC | `:6334` |
| Collection (typical) | `honeypot` |
| Config section | `[qdrant]` — `url`, `collection`, `sync_enabled`, `sync_cron_hour/minute` |
| Sync function | `gzmo_core::memory::qdrant_sync::sync_vault_to_qdrant` |

---

## 4. THINKING nodes

> **THINKING — database-cluster-compose.yml:qdrant**
> - *Reviewed:* `qdrant/qdrant:latest` with persistent volume.
> - *Insight:* `:latest` tag can drift on redeploy — point count may jump after image upgrade.
> - *Risk / limitation:* No pinned version; no resource limits in compose.
> - *Enhancement:* Pin Qdrant version digest; add memory limit. [CT101-safe]

> **THINKING — daemon_cmd.rs:qdrant sync cron**
> - *Reviewed:* 60s tick + `cron_due_today` catch-up after restart.
> - *Insight:* Same catch-up semantics as dream/spark — won't miss sync after daemon restart.
> - *Risk / limitation:* Full vault sync is O(n) on 60k+ facts; may spike CPU/RAM.
> - *Enhancement:* Incremental sync by `updated_at` watermark. [GZMO-next]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| VM200 embed layer | Embeddings computed remotely; Qdrant stores vectors only |
| GZMO-next | Could colocate Qdrant on workstation GPU host |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Pin Qdrant image version | [CT101-safe] |
| 2 | Incremental vault→Qdrant sync | [GZMO-next] |
| 3 | Collection healthcheck in compose | [CT101-safe] |
| 4 | Dual-write at ingest time (real-time mirror) | [GZMO-next] |
