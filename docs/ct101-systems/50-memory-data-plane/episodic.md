# Episodic — Daily Ledger & NO_REPLY Filter

**System:** [50-memory-data-plane](./SYSTEM.md)  
**Sources:** `gzmo-core/src/memory/episodic.rs`, `gzmo-core/src/memory/filter.rs`

---

## Capability

Episodic memory is an **append-only daily markdown ledger** (`memory/YYYY-MM-DD.md`) for user chat, tool runs, heartbeat checks, session distill markers, and internal monologue. The `NoReplyFilter` fail-closed routes `<NO_REPLY>` LLM output to episodic instead of the user stream. Episodic is **provenance context for Dream**, not primary RAG substrate.

---

## How it works

### File-backed store

```29:64:gzmo-core/src/memory/episodic.rs
    pub async fn append(&self, entry: &EpisodicEntry) -> Result<()> {
        let today = Utc::now().date_naive();
        let path = self.path_for_date(today);
        // create_dir_all, OpenOptions append, format_entry
    }

    pub async fn read_day(&self, date: NaiveDate) -> Result<String> { /* ... */ }
```

Entries are formatted with source tags (USER, HEARTBEAT, TOOL, SESSION, INTERNAL):

```86:113:gzmo-core/src/memory/episodic.rs
fn format_entry(entry: &EpisodicEntry) -> String {
    let source_tag = match &entry.source {
        EpisodicSource::UserChat => "💬 USER",
        EpisodicSource::ToolExecution { tool_name } => /* ### 🔧 TOOL */,
        // ...
    };
```

### NO_REPLY filter

```33:61:gzmo-core/src/memory/filter.rs
    pub async fn process(&self, chunk: &str) -> FilterResult {
        if chunk.contains("<NO_REPLY>") || chunk.contains("[NO_REPLY]") {
            let cleaned = chunk.replace("<NO_REPLY>", "")/* ... */;
            if !cleaned.is_empty() {
                self.store.append(&EpisodicEntry {
                    source: EpisodicSource::InternalMonologue,
                    is_silent: true,
                    /* ... */
                }).await.ok();
            }
            FilterResult::Suppressed
        } else {
            FilterResult::Forward(chunk.to_string())
        }
    }
```

Synapse pull also appends Pi summaries via `synapse_reader::pull_and_log_episodic`.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Config | `[memory] directory = "memory"` |
| Paths | `memory/2026-07-10.md`, etc. under project `data/` or `data-next/` |
| Consumers | DreamEngine (context snippet), agent loop filter, synapse pull |
| Not used for | Default honeypot recall / Qdrant sync |

---

## THINKING nodes

> **THINKING — episodic.rs:FileEpisodicStore**
> - *Reviewed:* Async tokio fs append per entry; no indexing.
> - *Insight:* Human-auditable log separate from structured vault — ops-friendly.
> - *Risk / limitation:* No full-text search across days without external grep.
> - *Enhancement:* Optional episodic FTS index for dream prep [GZMO-next].

> **THINKING — filter.rs:NoReplyFilter**
> - *Reviewed:* Fire-and-forget append on suppress; warns on failure only.
> - *Insight:* Fail-closed prevents internal monologue leaking to operators.
> - *Risk / limitation:* Partial `<NO_REPLY>` across streamed chunks could mis-route.
> - *Enhancement:* Buffer until tag complete in streaming mode [CT101-safe].

> **THINKING — synapse → episodic bridge**
> - *Reviewed:* `pull_and_log_episodic` writes Pi pull summaries as InternalMonologue.
> - *Insight:* Closes loop between Pi agent (488k synapse events) and nightly dream input.
> - *Risk / limitation:* Duplicate pulls if state file reset without idempotency key.
> - *Enhancement:* Hash-based dedup for episodic append [CT101-safe].

---

## Advancement

- **CT101:** Keep episodic as audit trail; do not promote raw episodic lines to honeypot without distill/extract.
- **GZMO-next:** Explicit Document layer URI linking episodic files to vault promotions.

---

## Enhancement backlog

1. **[CT101-safe]** Streaming NO_REPLY tag assembly in agent loop.
2. **[CT101-safe]** Episodic rotation/compress policy for multi-year logs.
3. **[CT101-safe]** Synapse pull dedup by event id in episodic content.
4. **[GZMO-next]** Episodic → SessionDistill auto-queue on day rollover.
5. **[GZMO-next]** Cross-day episodic search for dream context window.
