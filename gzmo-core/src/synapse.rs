//! # Synapse Event Bus — Append-Only Observability
//!
//! A lightweight, file-based JSONL log (`data/Synapse/events.jsonl`) that acts as an
//! audit trail for joint GZMO-Pi execution.
//!
//! **The Firewall:** GZMO does not run an active "chemistry consumer" or automated
//! state transitions that subscribe to the Synapse bus. The bus remains strictly
//! append-only for manual audits, telemetry collection, and joint debugging.
//!
//! **The Prime Directive:** This module writes events only. It never reads, never
//! consumes, never triggers state changes based on bus content.
//!
//! **Thread-local source:** `SynapseBus::set_event_source()` sets the `EventSource`
//! for all `append()` calls on the current thread. Callers (daemon, CLI commands)
//! should set it once before using the bus so events carry the correct `source`.
//!
//! **Advisory file locking:** Uses std file locking on Unix to prevent concurrent
//! writes from interleaving JSONL lines when daemon, CLI, and Pi write simultaneously.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Thread-local event source
// ---------------------------------------------------------------------------

thread_local! {
    /// Default EventSource for this thread. Set via `SynapseBus::set_event_source()`.
    /// If unset, events fall back to `EventSource::GzmoDaemon`.
    static SYNAPSE_SOURCE: std::cell::Cell<Option<EventSource>> =
        std::cell::Cell::new(None);
}

/// Set the default EventSource for all `SynapseBus::append()` calls
/// on the current thread.
///
/// Call once from your entry point (daemon, CLI command, TUI) so all
/// events written through any `SynapseBus` instance carry the correct source.
pub fn set_event_source(source: EventSource) {
    SYNAPSE_SOURCE.with(|s| s.set(Some(source)));
}

/// Resolve the effective EventSource: thread-local > fallback.
pub fn resolve_event_source(fallback: EventSource) -> EventSource {
    SYNAPSE_SOURCE.with(|s| s.get().unwrap_or(fallback))
}

// ---------------------------------------------------------------------------
// Event Types
// ---------------------------------------------------------------------------

/// Source of a Synapse event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    /// GZMO daemon tick (scheduled job execution)
    GzmoDaemon,
    /// GZMO CLI invocation (distill, dream, spark, ingest, health)
    GzmoCli,
    /// Pi Agent (via gzmo-integration skill)
    PiAgent,
}

/// Semantic event type identifier.
/// Follows the format `<domain>.<action>` (e.g., `quest.completed`, `health.tick`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // --- Daemon lifecycle ---
    /// Scheduled job tick fired
    DaemonTick,
    /// Scheduled job completed successfully
    DaemonJobComplete,
    /// Scheduled job failed with error
    DaemonJobFail,

    /// Cognition cron lifecycle (dream, spark, qdrant_sync, wiki, kg, …)
    CognitionSchedule,

    // --- Health ---
    /// Health probe cycle completed
    HealthTick,
    /// Health probe detected failure
    HealthFail,

    // --- Session / quest ---
    /// Agent turn / quest completed
    QuestComplete,
    /// Agent turn / quest failed
    QuestFail,
    /// Session started
    SessionStart,
    /// Session ended
    SessionEnd,

    // --- Memory operations ---
    /// DreamEngine consolidation completed
    DreamComplete,
    /// SparkEngine serendipity completed
    SparkComplete,
    /// IngestEngine processed input
    IngestComplete,
    /// Session distillation completed
    DistillComplete,
    /// WikiEngine (Knowledge Gardener) emitted/maintained wiki pages
    WikiComplete,

    // --- Senses (HSP — output-only, never feeds back) ---
    /// System tension metric emitted (no cognitive impact)
    SenseTension,
    /// Chaos ρ accumulator telemetry (PulseLoop snapshot fields)
    #[serde(rename = "chaos.rho_telemetry")]
    SenseChaosRho,

    // --- Pi interactions ---
    /// Pi agent sent a memory chunk to GZMO inbox
    PiMemorySent,
    /// Pi agent requested GZMO health
    PiHealthRequested,
}

/// A single Synapse event frame — the unit of observability.
///
/// All fields are required except `data`, which is optional for events
/// that carry no payload beyond the event type itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapseEvent {
    /// Unique event identifier
    pub id: Uuid,
    /// Event type (e.g., `quest.completed`)
    pub event_type: EventType,
    /// Source that generated this event
    pub source: EventSource,
    /// ISO 8601 UTC timestamp
    pub timestamp: DateTime<Utc>,
    /// Arbitrary JSON payload (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl SynapseEvent {
    /// Create a new Synapse event with no payload.
    pub fn new(event_type: EventType, source: EventSource) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            source,
            timestamp: Utc::now(),
            data: None,
        }
    }

    /// Create a new Synapse event with a JSON payload.
    pub fn with_data(event_type: EventType, source: EventSource, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            source,
            timestamp: Utc::now(),
            data: Some(data),
        }
    }

    /// Serialize to a single JSONL line (no trailing newline).
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| {
            // Fallback: if serialization fails, emit a minimal error event
            format!(
                r#"{{"id":"{}","event_type":"synapse.error","source":"synapse","timestamp":"{}","data":{{"error":"{}"}}}}"#,
                self.id,
                self.timestamp.to_rfc3339(),
                e.to_string().replace('"', "\\\"")
            )
        })
    }
}

/// Unified cognition cron observability: `engine` + `phase` + optional fields.
pub fn append_cognition_schedule(
    bus: &SynapseBus,
    engine: &str,
    phase: &str,
    extra: serde_json::Value,
) {
    let mut data = serde_json::json!({
        "engine": engine,
        "phase": phase,
    });
    if let Some(obj) = extra.as_object() {
        if let Some(data_obj) = data.as_object_mut() {
            for (k, v) in obj {
                data_obj.insert(k.clone(), v.clone());
            }
        }
    }
    bus.append(&SynapseEvent::with_data(
        EventType::CognitionSchedule,
        resolve_event_source(EventSource::GzmoDaemon),
        data,
    ));
}

// ---------------------------------------------------------------------------
// SynapseBus — Append-Only Writer
// ---------------------------------------------------------------------------

/// The Synapse event bus. Provides `append` and `append_batch` methods
/// that write events to `data/Synapse/events.jsonl` in append-only JSONL format.
///
/// **Event source:** Thread-local — set via `set_event_source()` before
/// using the bus, or pass `EventSource` directly when constructing events.
///
/// **Cross-process safety:** Uses std advisory file locking on the
/// `.lock` file alongside the JSONL log to prevent interleaving when
/// daemon, CLI, and Pi write concurrently.
#[derive(Debug, Clone)]
pub struct SynapseBus {
    /// Path to the JSONL file (default: `data/Synapse/events.jsonl`)
    pub path: PathBuf,
}

impl SynapseBus {
    /// Create a new SynapseBus with the default path.
    pub fn new() -> Self {
        Self {
            path: PathBuf::from("data/Synapse/events.jsonl"),
        }
    }

    /// Create a new SynapseBus with a custom path.
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Append a single event to the bus.
    ///
    /// Acquires an advisory file lock before writing to prevent
    /// interleaving with concurrent writers (daemon, CLI, Pi).
    ///
    /// Errors are logged via tracing but never propagated —
    /// the bus is fire-and-forget observability, not a critical path.
    pub fn append(&self, event: &SynapseEvent) {
        let line = event.to_json_line();
        self.append_line(&line);
    }

    /// Append a raw JSONL line with advisory locking.
    fn append_line(&self, line: &str) {
        // Ensure directory exists
        if let Some(parent) = self.path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                tracing::warn!(
                    synapse_path = %parent.display(),
                    error = %e,
                    "SynapseBus: failed to create directory"
                );
                return;
            }
        }

        let lock_path = self.path.with_file_name(format!(
            "{}.lock",
            self.path
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default()
        ));

        // Acquire advisory lock before writing
        let lock_file = match fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
        {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(
                    synapse_lock = %lock_path.display(),
                    error = %e,
                    "SynapseBus: failed to open lock file"
                );
                // Best-effort: write without lock
                if let Ok(mut file) = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)
                {
                    let _ = writeln!(file, "{}", line);
                }
                return;
            }
        };

        // Exclusive lock blocks until available
        if lock_file.lock().is_err() {
            tracing::warn!(
                synapse_lock = %lock_path.display(),
                "SynapseBus: failed to acquire exclusive lock, writing without lock"
            );
        }

        // Write the event
        match fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            Ok(mut file) => {
                if let Err(e) = writeln!(file, "{}", line) {
                    tracing::warn!(
                        synapse_path = %self.path.display(),
                        error = %e,
                        "SynapseBus: failed to write event"
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    synapse_path = %self.path.display(),
                    error = %e,
                    "SynapseBus: failed to open file for append"
                );
            }
        }

        // Lock is released automatically when `lock_file` drops
    }

    /// Convenience: append a batch of events in one pass.
    ///
    /// Acquires an advisory lock once, writes all lines, releases.
    /// More efficient than N individual appends for batch operations.
    pub fn append_batch(&self, events: &[SynapseEvent]) {
        if events.is_empty() {
            return;
        }

        // Ensure directory exists
        if let Some(parent) = self.path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                tracing::warn!(
                    synapse_path = %parent.display(),
                    error = %e,
                    "SynapseBus: failed to create directory for batch"
                );
                return;
            }
        }

        let lock_path = self.path.with_file_name(format!(
            "{}.lock",
            self.path
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default()
        ));

        // Acquire advisory lock before writing
        let lock_file = match fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
        {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(
                    synapse_lock = %lock_path.display(),
                    error = %e,
                    "SynapseBus: failed to open lock file for batch"
                );
                // Best-effort: write without lock
                if let Ok(mut file) = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)
                {
                    for event in events {
                        let line = event.to_json_line();
                        let _ = writeln!(file, "{}", line);
                    }
                }
                return;
            }
        };

        if lock_file.lock().is_err() {
            tracing::warn!(
                synapse_lock = %lock_path.display(),
                "SynapseBus: failed to acquire exclusive lock for batch, writing without lock"
            );
        }

        // Write all lines
        match fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            Ok(mut file) => {
                for event in events {
                    let line = event.to_json_line();
                    if let Err(e) = writeln!(file, "{}", line) {
                        tracing::warn!(
                            error = %e,
                            "SynapseBus: failed to write batch event"
                        );
                        break; // Stop on first failure
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "SynapseBus: failed to open file for batch append"
                );
            }
        }

        // Lock released when `lock_file` drops
    }
}

impl Default for SynapseBus {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_event_serialization() {
        let event = SynapseEvent::new(EventType::HealthTick, EventSource::GzmoDaemon);
        let json = event.to_json_line();
        let parsed: SynapseEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type, EventType::HealthTick);
        assert_eq!(parsed.source, EventSource::GzmoDaemon);
        assert!(parsed.data.is_none());
    }

    #[test]
    fn test_event_with_data() {
        let payload = serde_json::json!({ "status": "ok", "latency_ms": 42 });
        let event = SynapseEvent::with_data(
            EventType::DaemonJobComplete,
            EventSource::GzmoDaemon,
            payload.clone(),
        );
        let json = event.to_json_line();
        let parsed: SynapseEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.data, Some(payload));
    }

    #[test]
    fn test_append_and_read() {
        let tmp_dir = std::env::temp_dir().join("gzmo_synapse_test");
        let path = tmp_dir.join("events.jsonl");
        let bus = SynapseBus::with_path(path.clone());

        let event1 = SynapseEvent::new(EventType::HealthTick, EventSource::GzmoDaemon);
        let event2 = SynapseEvent::with_data(
            EventType::QuestComplete,
            EventSource::PiAgent,
            serde_json::json!({ "turn": 42 }),
        );

        bus.append(&event1);
        bus.append(&event2);

        let contents = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.trim().split('\n').collect();
        assert_eq!(lines.len(), 2);

        // Verify both events are valid JSON
        let _: SynapseEvent = serde_json::from_str(lines[0]).unwrap();
        let _: SynapseEvent = serde_json::from_str(lines[1]).unwrap();

        // Cleanup
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_batch_append() {
        let tmp_dir = std::env::temp_dir().join("gzmo_synapse_batch_test");
        let path = tmp_dir.join("events.jsonl");
        let bus = SynapseBus::with_path(path.clone());

        let events: Vec<SynapseEvent> = (0..5)
            .map(|_| SynapseEvent::new(EventType::DaemonTick, EventSource::GzmoDaemon))
            .collect();

        bus.append_batch(&events);

        let contents = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.trim().split('\n').collect();
        assert_eq!(lines.len(), 5);

        // Cleanup
        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
