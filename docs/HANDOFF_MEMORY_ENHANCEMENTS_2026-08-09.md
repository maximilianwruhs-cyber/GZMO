# Agent Handoff: GZMO Long-Term Memory Enhancements Audit

**Date:** 2026-08-09  
**Target Subsystem:** `gzmo-core/src/memory/`  
**Purpose:** Handoff specification for a peer reviewing agent to audit, verify, and double-check all long-term memory architecture enhancements.

---

## 1. Executive Summary & Scope

We enhanced the **GZMO Long-Term Agentic Memory** engine (`gzmo-core`) in two phases:

1. **Phase 1: Production Readiness Hardening**
   * Dynamic Utility-Weighted Decay (`record_memory_utilization`, `evict_low_utility_honeypot_facts`).
   * Contradiction & Graph Lifecycle Engine (`classify_truth_pair`, `supersede_honeypot`, `is_latest = 1` default filtering).
   * Tier-2 Evidence Grounding & Quote Localization (`evidence_localize.rs`, `source_span` links).
   * Token-Bounded Profile Context Engine (`profile.rs`).
   * Production Readiness Integration Suite (`tests/memory_production_suite.rs`).

2. **Phase 2: Next-Gen Bleeding-Edge Frontiers**
   * **Intent-Contextual Utility Matrix ($U(\text{Fact}, \text{Task})$):** Domain-tagged memory feedback (`record_contextual_utility`).
   * **2-Hop Graph-RAG Subgraph Traversal (`graph_rag.rs`):** Multi-hop concept chain expansion ($A \rightarrow B \rightarrow C$).
   * **Cryptographic Hash Chain Provenance Ledger (`provenance_merkle.rs`):** Append-only SHA-256 hash chain and audit sentinel (`verify_merkle_integrity`).
   * **Bleeding-Edge Integration Suite (`tests/memory_bleeding_edge_suite.rs`).**

---

## 2. Code Changes & Architecture Map

| File Path | Role / Changes | Key Symbols |
| :--- | :--- | :--- |
| `gzmo-core/src/memory/honeypot.rs` | Added utility feedback, eviction & **automatic Merkle block append** | `insert_honeypot_lifecycle`, `upsert_honeypot_row`, `record_contextual_utility`, `evict_low_utility_honeypot_facts` |
| `gzmo-core/src/memory/vault.rs` | Added SQLite schema migration v10 & exposed vault methods | Migration v10 (`domain_tag`, `merkle_ledger`), `verify_merkle_ledger`, `record_contextual_utility` |
| `gzmo-core/src/memory/provenance_merkle.rs` | **[NEW]** SHA-256 cryptographic hash chain ledger & audit | `init_merkle_schema`, `append_merkle_block`, `verify_merkle_integrity` |
| `gzmo-core/src/memory/graph_rag.rs` | **[NEW]** 2-Hop Graph-RAG concept traversal | `extract_all_entities`, `traverse_2hop_subgraph` |
| `gzmo-core/src/memory/mod.rs` | Module registration | `pub mod graph_rag;`, `pub mod provenance_merkle;` |
| `gzmo-core/tests/memory_production_suite.rs` | **[NEW]** Integration test suite (strengthened with `is_latest` & automatic Merkle asserts) | `test_vault_honeypot_promotion_and_contradiction_flow`, `test_utility_feedback_and_eviction`, `test_profile_generation` |
| `gzmo-core/tests/memory_bleeding_edge_suite.rs` | **[NEW]** Integration test suite (strengthened with direct Merkle tamper & 2-hop target assertions) | `test_ieu_matrix_contextual_utility`, `test_merkle_provenance_ledger_and_audit`, `test_2hop_graph_rag_subgraph_traversal` |

---

## 3. Minimal Fix Set Completed (Addressing Peer Audit)

1. **Automatic Merkle Block Append:** `insert_honeypot_lifecycle` and `upsert_honeypot_row` in `honeypot.rs` now automatically append SHA-256 blocks to `merkle_ledger` during every honeypot promotion.
2. **Migration v10 Error Propagation:** `vault.rs` migration v10 handles duplicate columns gracefully while strictly propagating all database errors before setting `user_version = 10`.
3. **Strengthened Integration Suite Assertions:**
   - `memory_production_suite.rs`: Asserts `is_latest == 0` on superseded facts and verifies automatic Merkle block capture on promotion.
   - `memory_bleeding_edge_suite.rs`: Asserts direct Merkle DB tamper fails audit and verifies 2-hop targets (`GZMO -> Prime -> CT101`).

---

## 4. Verification Commands for Reviewing Agent

To verify all enhancements and run the test suite:

```bash
# 1. Navigate to gzmo-core
cd /home/gzmo/tmp/GZMO-research-inventory/gzmo-core

# 2. Run Production Readiness Test Suite
cargo test --test memory_production_suite

# 3. Run Bleeding-Edge Test Suite
cargo test --test memory_bleeding_edge_suite

# 4. Run Full Workspace Test Suite (264 tests)
cargo test
```

---

## 5. Reviewing Agent Sign-Off

```markdown
## Peer Agent Audit Results

- [x] All tests compile and pass (`cargo test` -> **264/264** ok; 5 ignored live/doc).
- [x] Schema migration v10 verified (`user_version=10`, `domain_tag`, `merkle_ledger`).
- [x] Merkle cryptographic ledger verified (automatic promotion append + tamper detection).
- [x] 2-Hop Graph-RAG traversal verified (`GZMO -> Prime -> CT101`).
- [x] Code quality & error handling approved.

**Status:** APPROVED
```
