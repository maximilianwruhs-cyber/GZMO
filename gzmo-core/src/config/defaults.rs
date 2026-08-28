use std::path::PathBuf;

use super::pedagogy::TensionOscillationStepConfig;

pub(super) fn default_dice_loop_min() -> u32 {
    5
}

pub(super) fn default_dice_loop_max() -> u32 {
    60
}

pub(super) fn default_tension_oscillation_spawn_discovery() -> bool {
    true
}

pub(super) fn default_tension_oscillation_low_threshold() -> f64 {
    0.55
}

pub(super) fn default_tension_oscillation_cooldown_secs() -> u64 {
    3600
}

pub(super) fn default_tension_oscillation_blend_ticks() -> u64 {
    8
}

pub(super) fn default_tension_oscillation_sequence() -> Vec<TensionOscillationStepConfig> {
    vec![
        TensionOscillationStepConfig {
            target: 0.9,
            duration_secs: 60,
            label: "High tension — confirmation machine".to_string(),
        },
        TensionOscillationStepConfig {
            target: 0.5,
            duration_secs: 60,
            label: "Low tension — discovery machine".to_string(),
        },
        TensionOscillationStepConfig {
            target: 0.9,
            duration_secs: 60,
            label: "High tension — confirmation machine".to_string(),
        },
    ]
}

pub(super) fn default_discovery_max_pending() -> usize {
    2
}
pub(super) fn default_discovery_max_concurrent() -> usize {
    1
}
pub(super) fn default_discovery_session_priority() -> bool {
    true
}

pub(super) fn default_low_tension_discovery_cycle() -> bool {
    true
}

pub(super) fn default_discovery_scripts_root() -> String {
    std::env::var("GZMO_SKILLS_ROOT").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/maximilian-wruhs".into());
        format!("{home}/gzmo_skills")
    })
}

pub(super) fn default_low_tension_threshold() -> f64 {
    15.0
}

pub(super) fn default_low_tension_cooldown() -> u64 {
    300
}

pub(super) fn default_low_tension_opening() -> String {
    "[AUTONOMOUS — low tension] System tension is very low (τ={tension}%, tick {tick}, phase {phase}). \
     Begin a Socratic dialogue with the learner: ask one inviting question about stillness, dormancy, \
     or what the organism should attend to when the chaos field is calm. Do not lecture; do not give the answer."
        .to_string()
}

pub(super) fn default_sandbox_enabled() -> bool {
    true
}
pub(super) fn default_sandbox_max_code_chars() -> usize {
    2000
}
pub(super) fn default_sandbox_timeout_secs() -> u64 {
    10
}
pub(super) fn default_sandbox_max_output_chars() -> usize {
    4000
}
pub(super) fn default_sandbox_blocked_imports() -> Vec<String> {
    vec![
        "os".to_string(),
        "subprocess".to_string(),
        "socket".to_string(),
        "shutil".to_string(),
        "sys".to_string(),
    ]
}
pub(super) fn default_sandbox_orchestrator_offload() -> bool {
    false
}

pub(super) fn default_pedagogy_enabled() -> bool {
    true
}
pub(super) fn default_learner_data_dir() -> String {
    "data/learner".to_string()
}
pub(super) fn default_prereq_graphs_dir() -> String {
    "data/pedagogy/graphs".to_string()
}
pub(super) fn default_edf_log_path() -> String {
    "data/pedagogy/edf_log.jsonl".to_string()
}
pub(super) fn default_max_hint_level() -> u8 {
    5
}
pub(super) fn default_solution_leakage_penalty() -> f64 {
    1.0
}
pub(super) fn default_pedagogy_internal_max_tokens() -> u32 {
    512
}
pub(super) fn default_teachback_interval() -> u32 {
    8
}
pub(super) fn default_mentor_api_enabled() -> bool {
    true
}
pub(super) fn default_mentor_socket() -> String {
    "data/gzmo_mentor.sock".to_string()
}

pub(super) fn default_dream_honeypot_rem_enabled() -> bool {
    true
}

pub(super) fn default_honeypot_rem_anchor_limit() -> usize {
    4
}

pub(super) fn default_honeypot_rem_associate_k() -> usize {
    6
}

pub(super) fn default_dream_exclude_episodic_substrings() -> Vec<String> {
    vec![
        "sys_janitor".to_string(),
        "[job: sys_janitor]".to_string(),
        "[spark ".to_string(),
        "## spark —".to_string(),
        "[ingest:".to_string(),
        "ingested `".to_string(),
        "filesystem utilization".to_string(),
        "root filesystem".to_string(),
        "[hypothesis ".to_string(),
        "promoted=false".to_string(),
    ]
}

pub(super) fn default_dream_min_consolidation_chars() -> usize {
    400
}

pub(super) fn default_pipeline_chunk_chars() -> usize {
    28_000
}

pub(super) fn default_ingest_max_source_chars() -> usize {
    120_000
}
pub(super) fn default_ingest_inbox() -> String {
    "../data-next/inbox".into()
}
pub(super) fn default_ingest_batch_hour() -> u32 {
    2
}

pub(super) fn default_wiki_enabled() -> bool {
    true
}
pub(super) fn default_wiki_backend() -> String {
    "local".to_string()
}
pub(super) fn default_wiki_directory() -> String {
    "wiki".to_string()
}
pub(super) fn default_wiki_index_path() -> String {
    "wiki/index.md".to_string()
}
pub(super) fn default_wiki_log_path() -> String {
    "wiki/log.md".to_string()
}
pub(super) fn default_wiki_schema_path() -> String {
    "WIKI.md".to_string()
}
pub(super) fn default_wiki_emit_on_ingest() -> bool {
    true
}
pub(super) fn default_wiki_sync_cron_hour() -> u32 {
    5
}
pub(super) fn default_wiki_sync_cron_minute() -> u32 {
    30
}
pub(super) fn default_wiki_lint_cron_dow() -> u32 {
    0
}
pub(super) fn default_wiki_lint_cron_hour() -> u32 {
    6
}
pub(super) fn default_wiki_push_cron_hour() -> u32 {
    5
}
pub(super) fn default_wiki_push_cron_minute() -> u32 {
    30
}
pub(super) fn default_okforge_url() -> String {
    "http://127.0.0.1:3000".into()
}
pub(super) fn default_okforge_owner() -> String {
    "gzmo".into()
}
pub(super) fn default_okforge_repo() -> String {
    "gzmo-next-memory".into()
}
pub(super) fn default_okforge_token_env() -> String {
    "OKFORGE_TOKEN".into()
}
pub(super) fn default_okforge_agent_id() -> String {
    "gzmo-next".into()
}

pub(super) fn default_session_distill_enabled() -> bool {
    true
}

pub(super) fn default_sessions_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("data/sessions")
}

pub(super) fn default_session_distill_max_transcript() -> usize {
    28_000
}

pub(super) fn default_session_distill_use_librarian() -> bool {
    true
}

pub(super) fn default_session_distill_librarian_summary() -> bool {
    true
}

pub(super) fn default_session_distill_daemon_scheduled() -> bool {
    true
}

pub(super) fn default_session_distill_cron_hour() -> u32 {
    2
}

pub(super) fn default_session_distill_cron_minute() -> u32 {
    15
}

pub(super) fn default_metabolism_enabled() -> bool {
    true
}
pub(super) fn default_metabolism_promote_hour() -> u32 {
    2
}
pub(super) fn default_metabolism_promote_minute() -> u32 {
    30
}
pub(super) fn default_metabolism_embed_hour() -> u32 {
    2
}
pub(super) fn default_metabolism_embed_minute() -> u32 {
    45
}

pub(super) fn default_spark_anchor_decay_classes() -> Vec<String> {
    vec!["CuratedVault".to_string(), "SessionDistill".to_string()]
}

pub(super) fn default_spark_anchor_min_stale_days() -> u32 {
    0
}

pub(super) fn default_spark_anchor_max_stale_days() -> u32 {
    60
}

pub(super) fn default_spark_anchor_min_age_hours() -> u32 {
    6
}

pub(super) fn default_spark_recent_max_age_hours() -> u32 {
    72
}

pub(super) fn default_spark_min_anchor_recent_similarity() -> f64 {
    0.35
}

pub(super) fn default_spark_recent_dedupe_similarity() -> f64 {
    0.92
}

pub(super) fn default_spark_exclude_anchor_substrings() -> Vec<String> {
    vec![
        "[Session ".to_string(),
        "Topics discussed: GZMO, open sovereign.toml".to_string(),
        "filesystem utilization".to_string(),
        "sys_janitor".to_string(),
        "[ingest:".to_string(),
        "Root filesystem".to_string(),
        "CPU | RAM".to_string(),
    ]
}

pub(super) fn default_spark_max_session_anchor_age_days() -> u32 {
    14
}

pub(super) fn default_spark_refractory_slots() -> usize {
    48
}

pub(super) fn default_spark_refractory_half_life_hours() -> f64 {
    120.0
}

pub(super) fn default_spark_refractory_strength() -> f64 {
    0.95
}

pub(super) fn default_spark_soft_pick_top_k() -> usize {
    8
}

pub(super) fn default_spark_soft_pick_temperature() -> f64 {
    0.35
}

pub(super) fn default_spark_dice_min() -> u32 {
    20
}
pub(super) fn default_spark_dice_max() -> u32 {
    180
}
pub(super) fn default_spark_max_tokens_hypothesis() -> u32 {
    2048
}
pub(super) fn default_spark_max_tokens_verify() -> u32 {
    1024
}
pub(super) fn default_spark_max_connection_chars() -> usize {
    1200
}
pub(super) fn default_spark_min_citation_chars() -> usize {
    12
}

pub(super) fn default_true() -> bool {
    true
}

pub(super) fn default_embed_cache_ttl_secs() -> u64 {
    86_400
}

pub(super) fn default_embeddings_url() -> String {
    "http://localhost:8002/v1".to_string()
}

pub(super) fn default_embeddings_model() -> String {
    "Qwen3-Embedding-0.6B".to_string()
}

pub(super) fn default_qdrant_url() -> String {
    "http://192.168.31.202:6333".to_string()
}

pub(super) fn default_qdrant_collection() -> String {
    "honeypot".to_string()
}

pub(super) fn default_qdrant_sync_cron_hour() -> u32 {
    1
}

pub(super) fn default_qdrant_sync_cron_minute() -> u32 {
    45
}

pub(super) fn default_platform_search_enabled() -> bool {
    true
}

pub(super) fn default_knowledge_collection() -> String {
    "knowledge".to_string()
}

pub(super) fn default_knowledge_prefetch() -> usize {
    12
}

pub(super) fn default_kg_reconcile_hour() -> u32 {
    4
}
pub(super) fn default_kg_reconcile_minute() -> u32 {
    30
}

pub(super) fn default_synapse_pull_hour() -> u32 {
    2
}

pub(super) fn default_synapse_pull_minute() -> u32 {
    45
}

pub(super) fn default_synapse_pull_max_events() -> usize {
    50
}

pub(super) fn default_synapse_bus_path() -> std::path::PathBuf {
    std::path::PathBuf::from("data/Synapse/events.jsonl")
}

pub(super) fn default_librarian_url() -> String {
    "http://192.168.31.110:8083/v1".to_string()
}

pub(super) fn default_librarian_model() -> String {
    "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf".to_string()
}

pub(super) fn default_rerank_url() -> String {
    "http://192.168.31.110:8082/v1".to_string()
}

pub(super) fn default_rerank_model() -> String {
    "bge-reranker-v2-m3-q8_0.gguf".to_string()
}

pub(super) fn default_rerank_prefetch_multiplier() -> usize {
    4
}

pub(super) fn default_redis_enabled() -> bool {
    true
}
pub(super) fn default_redis_url() -> String {
    "redis://192.168.31.202:6379".to_string()
}
pub(super) fn default_distill_queue() -> String {
    "gzmo:distill:pending".to_string()
}
pub(super) fn default_distill_fallback_dir() -> PathBuf {
    PathBuf::from("data/distill-queue")
}

pub(super) fn default_archive_threshold() -> f64 {
    0.90
}
pub(super) fn default_response_reserve() -> f64 {
    0.10
}
pub(super) fn default_scratch_max_tokens() -> usize {
    2000
}

pub(super) fn default_subagent_enabled() -> bool {
    true
}
pub(super) fn default_subagent_max_concurrent() -> usize {
    5
}
pub(super) fn default_subagent_max_depth() -> u8 {
    2
}
pub(super) fn default_subagent_context_budget() -> usize {
    32_768
}
pub(super) fn default_subagent_summary_max() -> usize {
    800
}

pub(super) fn default_routing_engine() -> String {
    "local".to_string()
}

pub(super) fn default_vault_backend() -> String {
    "sqlite".to_string()
}

pub(super) fn default_workflow_skills_enabled() -> bool {
    true
}
pub(super) fn default_workflow_skills_dir() -> PathBuf {
    PathBuf::from("skills/workflows")
}
pub(super) fn default_workflow_model_can_activate() -> bool {
    true
}
pub(super) fn default_workflow_max_active() -> usize {
    2
}
pub(super) fn default_workflow_handoff_dir() -> PathBuf {
    PathBuf::from("data-next/handoffs")
}
pub(super) fn default_workflow_handoff_to_vault() -> bool {
    true
}

pub(super) fn default_tools_profile() -> String {
    "developer".to_string()
}
pub(super) fn default_tools_audit() -> bool {
    true
}

pub(super) fn default_watcher_debounce_secs() -> u64 {
    3
}

// ─── Defaults ───────────────────────────────────────────────────────────

pub(super) fn default_soul_path() -> PathBuf {
    PathBuf::from("SOUL.md")
}
pub(super) fn default_memory_dir() -> PathBuf {
    PathBuf::from("memory")
}
pub(super) fn default_vault_db() -> PathBuf {
    PathBuf::from("data/vault.db")
}
pub(super) fn default_skills_dir() -> PathBuf {
    PathBuf::from("skills")
}
pub(super) fn default_dreams_path() -> PathBuf {
    PathBuf::from("DREAMS.md")
}
pub(super) fn default_provider() -> String {
    "local".to_string()
}
pub(super) fn default_engine_url() -> String {
    "http://localhost:1234/v1".to_string()
}
pub(super) fn default_model_name() -> String {
    "gemma-4-E4B-it-Q4_K_M.gguf".to_string()
}
pub(super) fn default_temperature() -> f32 {
    0.3
}
pub(super) fn default_top_p() -> f32 {
    0.95
}
pub(super) fn default_max_tokens() -> u32 {
    8192
}
pub(super) fn default_max_iterations() -> usize {
    40
}
pub(super) fn default_heartbeat_secs() -> u64 {
    1800
}
pub(super) fn default_step_iterations() -> usize {
    20
}
pub(super) fn default_dream_enabled() -> bool {
    true
}
pub(super) fn default_dream_verify() -> bool {
    true
}
pub(super) fn default_dream_min_confidence() -> f64 {
    0.85
}
pub(super) fn default_dream_verify_temperature() -> f32 {
    0.1
}
pub(super) fn default_dream_cron_hour() -> u32 {
    1
}
pub(super) fn default_dream_cron_minute() -> u32 {
    0
}
pub(super) fn default_ingest_enabled() -> bool {
    true
}
pub(super) fn default_kg_require_evidence() -> bool {
    true
}
pub(super) fn default_kg_strict() -> bool {
    true
}
pub(super) fn default_spark_enabled() -> bool {
    true
}
pub(super) fn default_spark_hypothesis_temperature() -> f32 {
    0.2
}
pub(super) fn default_spark_candidate_limit() -> usize {
    5
}
pub(super) fn default_spark_recent_limit() -> usize {
    2
}
pub(super) fn default_spark_quarantine_confidence() -> f64 {
    0.6
}
pub(super) fn default_spark_cron_hours() -> Vec<u32> {
    vec![9, 14, 21]
}
pub(super) fn default_spark_cron_minute() -> u32 {
    17
}

pub(super) fn default_cloud_provider() -> String {
    "openrouter".to_string()
}
pub(super) fn default_cloud_url() -> String {
    "https://openrouter.ai/api/v1".to_string()
}
pub(super) fn default_cloud_model() -> String {
    "openrouter/free".to_string()
}
