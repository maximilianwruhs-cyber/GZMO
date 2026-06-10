#!/usr/bin/env bash
# Generate fixtures for Phase 0 benchmark
set -euo pipefail
mkdir -p scripts/compression-bench/fixtures

# 1. shell_grep_large.txt
grep -rn "pub fn " gzmo-core/src/ > scripts/compression-bench/fixtures/shell_grep_large.txt

# 2. neo4j_read_graph.json
cp logs/mega2-lost-facts-investigation.json scripts/compression-bench/fixtures/neo4j_read_graph.json

# 3. gzmo_memory_search.txt
cat << 'EOF' > scripts/compression-bench/fixtures/gzmo_memory_search.txt
[RECALL] Search Results for query: "Prime infrastructure"
Hit 1: [vault.db:L100] Prime runs on VM200 on LXC101 with fallback to LXC102.
Hit 2: [vault.db:L210] Prime's hot budget is configured to 235929 tokens.
Hit 3: [docs/INFRASTRUCTURE_MAP.md] Section 6 explains the Redis configuration.
Hit 4: [SOUL.md] Prime's core personality constraints.
Hit 5: [memory/facts.db] Historical sessions on VM200.
EOF

# 4. wiki_search.txt
cp wiki/sources/drive-research-hermes-compression-and-bol-architecture.md scripts/compression-bench/fixtures/wiki_search.txt

# 5. read_file_rust.txt
cp gzmo-core/src/agent_loop.rs scripts/compression-bench/fixtures/read_file_rust.txt

# 6. web_search.txt
cat << 'EOF' > scripts/compression-bench/fixtures/web_search.txt
Search Results for "context compression LLM":
1. LLMLingua: Compressing Prompts for Efficient Inference
   Url: https://arxiv.org/abs/2310.05736
   Abstract: This paper presents LLMLingua to compress prompt contexts up to 20x.
2. LongLLMLingua: Accelerating Long-Context LLMs
   Url: https://arxiv.org/abs/2310.06839
   Abstract: Enhancing RAG density and latency by pruning non-essential context.
3. Headroom: Context Compression Layer for AI Agents
   Url: https://github.com/chopratejas/headroom
   Abstract: Local-first, reversible context compression with 6 algorithms.
EOF

# 7. subagent_summary.txt
cat << 'EOF' > scripts/compression-bench/fixtures/subagent_summary.txt
Subagent Task Complete: Research on local Redis integration
We analyzed the connection parameters and benchmarked the pipeline.
Redis is accessible at redis://192.168.31.202:6379.
Tested keys: gzmo:scratch:test and gzmo:distill:queue.
Latency for GET/SET operations is < 1ms.
We recommend using namespace prefix 'gzmo:ccr:' with a 1-hour TTL.
Summary of recommendations:
1. Enable Redis backend via gzmo.toml.
2. Store serialized payloads as binary or JSON strings.
3. Use fail-open fallback in core modules.
EOF

# 8. orchestrator_log.txt
cp logs/antigravity-mega-daemon-grep.txt scripts/compression-bench/fixtures/orchestrator_log.txt

# 9. mcp_status.json
cat << 'EOF' > scripts/compression-bench/fixtures/mcp_status.json
{
  "status": "healthy",
  "mcp_servers": {
    "notebooklm": {
      "status": "connected",
      "latency_ms": 124,
      "authenticated": true,
      "capabilities": ["sources", "notes", "chat"]
    },
    "memory_mcp": {
      "status": "connected",
      "latency_ms": 12,
      "authenticated": true,
      "capabilities": ["vault", "recall"]
    }
  },
  "timestamp": "2026-06-10T15:25:00Z"
}
EOF

# 10. distill_transcript.txt
cp logs/antigravity-spark-m3.log scripts/compression-bench/fixtures/distill_transcript.txt

echo "Fixtures generated successfully."
