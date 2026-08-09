# Public portfolio inventory vs project-catalog

**Date:** 2026-08-09  
**Ticket:** [Inventory public portfolio vs project-catalog](https://github.com/maximilianwruhs-cyber/GZMO/issues/143) (map [#142](https://github.com/maximilianwruhs-cyber/GZMO/issues/142))  
**Question:** Complete public repo inventory for `maximilianwruhs-cyber`, and catalog relation (documented / missing / stale) for each row.  

Facts only — no Canonical/Satellite/Lab/Archive classifications.

---

## Sources

| Source | What was read |
|--------|---------------|
| GitHub REST `GET /users/maximilianwruhs-cyber/repos?per_page=100` (paginated via `gh api`) | Public repo inventory; fetched `2026-08-09T09:22:19Z`; **84** repos (`private=false`, `fork=false` for all rows) |
| `/home/gzmo/project-catalog/README.md` | Ecosystem map, “All GitHub projects (22)” tables, LTL pointer |
| `/home/gzmo/project-catalog/ecosystems/*.md` | Member tables: agenticos (7), gzmo-constellation (10), meta-ops (4), standalone (1); archived note for tinyFolder |
| `/home/gzmo/project-catalog/LITTLE_TOOLS_LAB.md` | 46-tool index + meta repo URL |
| `/home/gzmo/project-catalog/projects/*.md` | Per-repo cards (**68** files excluding `_template-little-tool.md`) |

---

## Catalog coverage rules (this note)

| Status | Meaning |
|--------|---------|
| **documented** | Named in an ecosystem Members table, and/or listed in README “All GitHub projects (22)”, and/or has `projects/<name>.md`, and/or is a row in `LITTLE_TOOLS_LAB.md` |
| **missing** | Public on GitHub; none of the above |
| **stale** | Present in catalog materials, but coverage is incomplete or catalog counts contradict this inventory (called out per row and/or in Catalog-level stale facts) |

Little Tools Lab tools are **documented** via `LITTLE_TOOLS_LAB.md` + `projects/<tool>.md`; they are not expected to appear in the four ecosystem Members tables.

---

## Catalog-level stale facts

1. `README.md` heading **“All GitHub projects (22)”** lists **22** unique project cards under that section; GitHub returns **84** public repos for the user.
2. Ecosystem Members sums: 7 + 10 + 4 + 1 = **22** named members (plus archived `tinyFolder` noted under GZMO constellation). That set does not include the **46** LTL tool repos, the **little-tools-lab** meta repo, or the **15** missing repos below.
3. Arithmetic check: 22 (README core tables) + 46 (LTL tools) + 1 (little-tools-lab meta) = **69** catalog-touched names; GitHub has **84**; gap = **15** missing (matches row table).
4. `LITTLE_TOOLS_LAB.md` claims **46/46** tools with GitHub links; all **46** tool names resolve to public repos in this inventory.
5. `little-tools-lab` is described in README / `LITTLE_TOOLS_LAB.md` as the meta repo but has **no** `projects/little-tools-lab.md` card → row status **stale**.

---

## Summary counts

| Catalog status | Count |
|----------------|------:|
| documented | 68 |
| missing | 15 |
| stale | 1 |
| **Total public repos** | **84** |

---

## Full inventory (GitHub → catalog)

| Repo | Description | Default branch | Last push (UTC) | Catalog status | Project doc | LTL row | Ecosystem membership |
|------|-------------|----------------|-----------------|----------------|-------------|---------|----------------------|
| [adaptive-tempo](https://github.com/maximilianwruhs-cyber/adaptive-tempo) | — | `main` | 2026-07-10 15:28:12 | documented | `projects/adaptive-tempo.md` | yes | little-tools-lab bucket (index only) |
| [ados](https://github.com/maximilianwruhs-cyber/ados) | Agent-Native Development Operating System | `master` | 2026-08-09 08:17:16 | missing | — | — | — |
| [AOS](https://github.com/maximilianwruhs-cyber/AOS) | Sovereign AI layer for Ubuntu — hot-swaps LLMs by energy telemetry (Intelligence per Watt) | `main` | 2026-04-03 07:19:35 | documented | `projects/AOS.md` | — | agenticos |
| [AOS-Customer-Edition](https://github.com/maximilianwruhs-cyber/AOS-Customer-Edition) | Zero-touch AOS deployment — one curl command installs a complete sovereign AI stack | `master` | 2026-03-28 14:22:38 | documented | `projects/AOS-Customer-Edition.md` | — | agenticos |
| [AOS-Intelligence-Dashboard](https://github.com/maximilianwruhs-cyber/AOS-Intelligence-Dashboard) | VS Codium extension — real-time energy monitoring, model benchmarking & LLM leaderboard | `master` | 2026-07-19 06:00:58 | documented | `projects/AOS-Intelligence-Dashboard.md` | — | agenticos |
| [AttractorBench](https://github.com/maximilianwruhs-cyber/AttractorBench) | — | `main` | 2026-07-02 12:21:29 | documented | `projects/AttractorBench.md` | — | gzmo-constellation |
| [baseline-bench](https://github.com/maximilianwruhs-cyber/baseline-bench) | — | `main` | 2026-07-16 10:02:49 | documented | `projects/baseline-bench.md` | yes | little-tools-lab bucket (index only) |
| [cabinet-sim](https://github.com/maximilianwruhs-cyber/cabinet-sim) | — | `main` | 2026-07-21 18:31:54 | documented | `projects/cabinet-sim.md` | yes | little-tools-lab bucket (index only) |
| [config-fuse](https://github.com/maximilianwruhs-cyber/config-fuse) | — | `main` | 2026-07-16 10:02:58 | documented | `projects/config-fuse.md` | yes | little-tools-lab bucket (index only) |
| [context-prune](https://github.com/maximilianwruhs-cyber/context-prune) | — | `main` | 2026-07-16 10:09:17 | documented | `projects/context-prune.md` | yes | little-tools-lab bucket (index only) |
| [devstack_v2](https://github.com/maximilianwruhs-cyber/devstack_v2) | — | `master` | 2026-04-01 11:41:15 | documented | `projects/devstack_v2.md` | — | agenticos |
| [dice-scheduler](https://github.com/maximilianwruhs-cyber/dice-scheduler) | — | `main` | 2026-07-10 15:28:19 | documented | `projects/dice-scheduler.md` | yes | little-tools-lab bucket (index only) |
| [draft-temp-bench](https://github.com/maximilianwruhs-cyber/draft-temp-bench) | — | `main` | 2026-07-10 15:28:20 | documented | `projects/draft-temp-bench.md` | yes | little-tools-lab bucket (index only) |
| [edge-node](https://github.com/maximilianwruhs-cyber/edge-node) | — | `master` | 2026-06-16 07:20:11 | documented | `projects/edge-node.md` | — | gzmo-constellation |
| [endpoint-scan](https://github.com/maximilianwruhs-cyber/endpoint-scan) | — | `main` | 2026-07-10 15:28:21 | documented | `projects/endpoint-scan.md` | yes | little-tools-lab bucket (index only) |
| [escape-loop-bench](https://github.com/maximilianwruhs-cyber/escape-loop-bench) | — | `main` | 2026-07-16 10:09:19 | documented | `projects/escape-loop-bench.md` | yes | little-tools-lab bucket (index only) |
| [etl-cli](https://github.com/maximilianwruhs-cyber/etl-cli) | — | `main` | 2026-07-10 15:28:24 | documented | `projects/etl-cli.md` | yes | little-tools-lab bucket (index only) |
| [evidence-locate](https://github.com/maximilianwruhs-cyber/evidence-locate) | — | `main` | 2026-07-16 10:02:43 | documented | `projects/evidence-locate.md` | yes | little-tools-lab bucket (index only) |
| [export-knowledge](https://github.com/maximilianwruhs-cyber/export-knowledge) | — | `main` | 2026-07-10 15:28:27 | documented | `projects/export-knowledge.md` | yes | little-tools-lab bucket (index only) |
| [faithfulness-judge](https://github.com/maximilianwruhs-cyber/faithfulness-judge) | — | `main` | 2026-07-15 13:23:21 | documented | `projects/faithfulness-judge.md` | yes | little-tools-lab bucket (index only) |
| [forget-lint](https://github.com/maximilianwruhs-cyber/forget-lint) | — | `master` | 2026-07-20 13:53:20 | missing | — | — | — |
| [frankenstein](https://github.com/maximilianwruhs-cyber/frankenstein) | mixture of experiments, pick what you need | `main` | 2026-06-15 18:55:26 | documented | `projects/frankenstein.md` | — | meta-ops |
| [graph-ledger](https://github.com/maximilianwruhs-cyber/graph-ledger) | — | `main` | 2026-07-10 15:28:29 | documented | `projects/graph-ledger.md` | yes | little-tools-lab bucket (index only) |
| [GZMO](https://github.com/maximilianwruhs-cyber/GZMO) | Sovereign curated memory for coding agents — local SQLite, honeypot gate, supersession chains — MCP for Cursor & Pi | `main` | 2026-07-25 13:15:23 | documented | `projects/GZMO.md` | — | gzmo-constellation |
| [gzmo-core-clean](https://github.com/maximilianwruhs-cyber/gzmo-core-clean) | Clean GZMO Core - From-scratch architecture without theatrical language | `master` | 2026-07-14 10:55:11 | documented | `projects/gzmo-core-clean.md` | — | gzmo-constellation |
| [gzmo-fresh-stack](https://github.com/maximilianwruhs-cyber/gzmo-fresh-stack) | Wipe-recovery handoff: fresh Knowledge OS distillation stack + OSS map + implementation plan | `main` | 2026-07-25 12:21:27 | missing | — | — | — |
| [gzmo-observatory](https://github.com/maximilianwruhs-cyber/gzmo-observatory) | Live CT101 mind visualization for GZMO | `master` | 2026-07-16 11:08:35 | missing | — | — | — |
| [GZMO-Pi-](https://github.com/maximilianwruhs-cyber/GZMO-Pi-) | GZMO curated memory for Pi — extension + MCP (pi-mcp-adapter); optional Redis/Neo4j/Qdrant living stack | `main` | 2026-07-18 19:28:14 | missing | — | — | — |
| [gzmo-rebuild](https://github.com/maximilianwruhs-cyber/gzmo-rebuild) | GZMO backup | `main` | 2026-07-02 12:21:58 | documented | `projects/gzmo-rebuild.md` | — | gzmo-constellation |
| [gzmo_skills](https://github.com/maximilianwruhs-cyber/gzmo_skills) | GZMO orchestration scripts, prompts, systemd units, and discovery pipeline | `main` | 2026-07-19 08:14:34 | documented | `projects/gzmo_skills.md` | — | gzmo-constellation |
| [gzmo_tinyFolder](https://github.com/maximilianwruhs-cyber/gzmo_tinyFolder) | — | `main` | 2026-06-16 07:20:12 | documented | `projects/gzmo_tinyFolder.md` | — | gzmo-constellation |
| [herdr](https://github.com/maximilianwruhs-cyber/herdr) | agent multiplexer that lives in your terminal. | `master` | 2026-07-10 21:24:09 | missing | — | — | — |
| [honeypot-gate](https://github.com/maximilianwruhs-cyber/honeypot-gate) | Curated-fact qualification and lifecycle classifier for GZMO honeypot memory | `main` | 2026-07-22 06:57:46 | documented | `projects/honeypot-gate.md` | yes | little-tools-lab bucket (index only) |
| [HSP](https://github.com/maximilianwruhs-cyber/HSP) | Turn your machine's heartbeat into music — real-time Linux telemetry to MIDI | `main` | 2026-07-18 19:40:12 | documented | `projects/HSP.md` | — | agenticos |
| [HSP-Pi-](https://github.com/maximilianwruhs-cyber/HSP-Pi-) | HSP Relay for Pi — silent record, one R2 settle chatter | `main` | 2026-07-18 19:44:15 | missing | — | — | — |
| [hsp-probe](https://github.com/maximilianwruhs-cyber/hsp-probe) | — | `main` | 2026-07-10 15:28:32 | documented | `projects/hsp-probe.md` | yes | little-tools-lab bucket (index only) |
| [HSP-VS-Codium-Extension](https://github.com/maximilianwruhs-cyber/HSP-VS-Codium-Extension) | VS Codium sidebar for live HSP telemetry — CPU, RAM, GPU visualization and control | `master` | 2026-03-29 08:51:01 | documented | `projects/HSP-VS-Codium-Extension.md` | — | agenticos |
| [kg-reconcile](https://github.com/maximilianwruhs-cyber/kg-reconcile) | — | `main` | 2026-07-10 15:28:33 | documented | `projects/kg-reconcile.md` | yes | little-tools-lab bucket (index only) |
| [ki-assessment-tool](https://github.com/maximilianwruhs-cyber/ki-assessment-tool) | — | `master` | 2026-04-07 07:35:53 | documented | `projects/ki-assessment-tool.md` | — | standalone |
| [little-tools-lab](https://github.com/maximilianwruhs-cyber/little-tools-lab) | Meta repo: catalog, scripts, and CI for 46 Little Tools Lab extractable tools | `main` | 2026-07-22 06:57:38 | stale — README + LITTLE_TOOLS_LAB.md meta mention; no projects/*.md | — (stale: no card) | meta (not a tool row) | — |
| [llama-cpp-prime-bench](https://github.com/maximilianwruhs-cyber/llama-cpp-prime-bench) | Local prime-bench patches for llama.cpp | `master` | 2026-07-03 07:33:28 | documented | `projects/llama-cpp-prime-bench.md` | — | gzmo-constellation |
| [lorenz-map](https://github.com/maximilianwruhs-cyber/lorenz-map) | — | `main` | 2026-07-16 10:02:45 | documented | `projects/lorenz-map.md` | yes | little-tools-lab bucket (index only) |
| [maximilianwruhs-cyber](https://github.com/maximilianwruhs-cyber/maximilianwruhs-cyber) | — | `master` | 2026-04-23 10:21:42 | documented | `projects/maximilianwruhs-cyber.md` | — | meta-ops |
| [mcp-neo4j-memory-gzmo](https://github.com/maximilianwruhs-cyber/mcp-neo4j-memory-gzmo) | GZMO backup | `main` | 2026-07-02 12:21:54 | documented | `projects/mcp-neo4j-memory-gzmo.md` | — | gzmo-constellation |
| [mutation-queue](https://github.com/maximilianwruhs-cyber/mutation-queue) | — | `main` | 2026-07-10 15:28:36 | documented | `projects/mutation-queue.md` | yes | little-tools-lab bucket (index only) |
| [neural-finesse](https://github.com/maximilianwruhs-cyber/neural-finesse) | — | `main` | 2026-07-15 13:23:20 | documented | `projects/neural-finesse.md` | yes | little-tools-lab bucket (index only) |
| [Obolus](https://github.com/maximilianwruhs-cyber/Obolus) | Which local model gives you the best answers per joule on your machine. | `main` | 2026-07-18 19:08:00 | documented | `projects/Obolus.md` | — | agenticos |
| [Obolus-Arena](https://github.com/maximilianwruhs-cyber/Obolus-Arena) | Open lab harness for local SLM organs under the Obolus z metric (quality / joules × price) | `main` | 2026-07-21 18:35:55 | missing | — | — | — |
| [okforge](https://github.com/maximilianwruhs-cyber/okforge) | Forgejo fork: OKF Knowledge Bundles by default with OKCP agent REST API | `main` | 2026-07-16 11:14:41 | missing | — | — | — |
| [organ-audit](https://github.com/maximilianwruhs-cyber/organ-audit) | — | `main` | 2026-07-10 15:28:38 | documented | `projects/organ-audit.md` | yes | little-tools-lab bucket (index only) |
| [pdu-reflect](https://github.com/maximilianwruhs-cyber/pdu-reflect) | Prosecutor–Defender–Umpire Hegelian reflection for lab claims | `master` | 2026-07-20 13:53:17 | missing | — | — | — |
| [pedagogy-bench](https://github.com/maximilianwruhs-cyber/pedagogy-bench) | — | `main` | 2026-07-10 15:28:40 | documented | `projects/pedagogy-bench.md` | yes | little-tools-lab bucket (index only) |
| [plan-gate](https://github.com/maximilianwruhs-cyber/plan-gate) | — | `main` | 2026-07-10 15:28:41 | documented | `projects/plan-gate.md` | yes | little-tools-lab bucket (index only) |
| [rapl-route](https://github.com/maximilianwruhs-cyber/rapl-route) | — | `main` | 2026-07-10 15:28:42 | documented | `projects/rapl-route.md` | yes | little-tools-lab bucket (index only) |
| [recall-eval](https://github.com/maximilianwruhs-cyber/recall-eval) | — | `main` | 2026-07-10 15:28:43 | documented | `projects/recall-eval.md` | yes | little-tools-lab bucket (index only) |
| [rem-substrate](https://github.com/maximilianwruhs-cyber/rem-substrate) | — | `main` | 2026-07-10 15:28:45 | documented | `projects/rem-substrate.md` | yes | little-tools-lab bucket (index only) |
| [rerank-probe](https://github.com/maximilianwruhs-cyber/rerank-probe) | — | `main` | 2026-07-10 15:28:46 | documented | `projects/rerank-probe.md` | yes | little-tools-lab bucket (index only) |
| [research-budget](https://github.com/maximilianwruhs-cyber/research-budget) | — | `main` | 2026-07-10 15:28:48 | documented | `projects/research-budget.md` | yes | little-tools-lab bucket (index only) |
| [rrf-recall](https://github.com/maximilianwruhs-cyber/rrf-recall) | — | `main` | 2026-07-16 10:02:56 | documented | `projects/rrf-recall.md` | yes | little-tools-lab bucket (index only) |
| [seed-curator](https://github.com/maximilianwruhs-cyber/seed-curator) | — | `main` | 2026-07-10 15:28:50 | documented | `projects/seed-curator.md` | yes | little-tools-lab bucket (index only) |
| [self-ask](https://github.com/maximilianwruhs-cyber/self-ask) | — | `main` | 2026-07-10 15:28:52 | documented | `projects/self-ask.md` | yes | little-tools-lab bucket (index only) |
| [session-distill](https://github.com/maximilianwruhs-cyber/session-distill) | — | `main` | 2026-07-16 10:02:53 | documented | `projects/session-distill.md` | yes | little-tools-lab bucket (index only) |
| [shadow-note](https://github.com/maximilianwruhs-cyber/shadow-note) | — | `main` | 2026-07-10 15:28:54 | documented | `projects/shadow-note.md` | yes | little-tools-lab bucket (index only) |
| [skill-patch](https://github.com/maximilianwruhs-cyber/skill-patch) | — | `main` | 2026-07-10 15:28:56 | documented | `projects/skill-patch.md` | yes | little-tools-lab bucket (index only) |
| [sovereign-agent](https://github.com/maximilianwruhs-cyber/sovereign-agent) | — | `master` | 2026-04-08 11:13:14 | documented | `projects/sovereign-agent.md` | — | gzmo-constellation |
| [spark-link](https://github.com/maximilianwruhs-cyber/spark-link) | Serendipitous memory linker — stale anchor + recent facts, verify L3 links | `main` | 2026-07-22 06:57:42 | documented | `projects/spark-link.md` | yes | little-tools-lab bucket (index only) |
| [speed-compare](https://github.com/maximilianwruhs-cyber/speed-compare) | — | `main` | 2026-07-10 15:28:58 | documented | `projects/speed-compare.md` | yes | little-tools-lab bucket (index only) |
| [spot-sweep](https://github.com/maximilianwruhs-cyber/spot-sweep) | — | `main` | 2026-07-10 15:29:00 | documented | `projects/spot-sweep.md` | yes | little-tools-lab bucket (index only) |
| [stigmergy-queue](https://github.com/maximilianwruhs-cyber/stigmergy-queue) | Ideas→Architecture→Build→QA stigmergic folder pipeline (lab) | `master` | 2026-07-20 11:28:04 | missing | — | — | — |
| [swap](https://github.com/maximilianwruhs-cyber/swap) | — | `main` | 2026-07-15 13:23:35 | documented | `projects/swap.md` | — | meta-ops |
| [synapse-health](https://github.com/maximilianwruhs-cyber/synapse-health) | — | `main` | 2026-07-10 15:29:01 | documented | `projects/synapse-health.md` | yes | little-tools-lab bucket (index only) |
| [synapse-tail](https://github.com/maximilianwruhs-cyber/synapse-tail) | — | `main` | 2026-07-10 15:29:03 | documented | `projects/synapse-tail.md` | yes | little-tools-lab bucket (index only) |
| [temp-bench](https://github.com/maximilianwruhs-cyber/temp-bench) | Sweep fixed LLM temperatures per task suite; outputs JSON reports and gzmo.toml recommendations | `main` | 2026-07-10 15:29:04 | documented | `projects/temp-bench.md` | yes | little-tools-lab bucket (index only) |
| [tempo-bench](https://github.com/maximilianwruhs-cyber/tempo-bench) | — | `main` | 2026-07-16 10:02:46 | documented | `projects/tempo-bench.md` | yes | little-tools-lab bucket (index only) |
| [tinyFolder](https://github.com/maximilianwruhs-cyber/tinyFolder) | 📁 tinyFolder a local-first, filesystem-driven AI daemon | `main` | 2026-06-19 21:43:22 | documented | `projects/tinyFolder.md` | — | meta-ops; gzmo-constellation:archived |
| [token-economy](https://github.com/maximilianwruhs-cyber/token-economy) | — | `master` | 2026-07-20 13:53:13 | missing | — | — | — |
| [tool-chain](https://github.com/maximilianwruhs-cyber/tool-chain) | Follow vault/fact refs → bounded auto-read expansion (Tools Are Leaves closer) | `master` | 2026-07-20 13:53:23 | missing | — | — | — |
| [top-p-bench](https://github.com/maximilianwruhs-cyber/top-p-bench) | — | `main` | 2026-07-10 15:29:07 | documented | `projects/top-p-bench.md` | yes | little-tools-lab bucket (index only) |
| [trace-memory](https://github.com/maximilianwruhs-cyber/trace-memory) | Cross-task trace retrieve → inject strategies (lab) | `master` | 2026-07-20 09:34:10 | missing | — | — | — |
| [trigger-sim](https://github.com/maximilianwruhs-cyber/trigger-sim) | — | `main` | 2026-07-10 15:29:08 | documented | `projects/trigger-sim.md` | yes | little-tools-lab bucket (index only) |
| [verify-gates](https://github.com/maximilianwruhs-cyber/verify-gates) | — | `master` | 2026-07-20 09:18:24 | missing | — | — | — |
| [verify-suite](https://github.com/maximilianwruhs-cyber/verify-suite) | Verifiable math and code tasks with ground-truth scoring for LLM benchmarks | `main` | 2026-07-10 15:29:09 | documented | `projects/verify-suite.md` | yes | little-tools-lab bucket (index only) |
| [wiki-lint](https://github.com/maximilianwruhs-cyber/wiki-lint) | — | `main` | 2026-07-10 15:29:11 | documented | `projects/wiki-lint.md` | yes | little-tools-lab bucket (index only) |
| [zpd-tutor](https://github.com/maximilianwruhs-cyber/zpd-tutor) | — | `main` | 2026-07-10 15:29:12 | documented | `projects/zpd-tutor.md` | yes | little-tools-lab bucket (index only) |

---

## Missing repos (15)

Present on GitHub; absent from ecosystem Members tables, README “All GitHub projects (22)”, `projects/*.md`, and `LITTLE_TOOLS_LAB.md` tool rows.

| Repo | Description | Default branch | Last push (UTC) |
|------|-------------|----------------|-----------------|
| [ados](https://github.com/maximilianwruhs-cyber/ados) | Agent-Native Development Operating System | `master` | 2026-08-09 08:17:16 |
| [forget-lint](https://github.com/maximilianwruhs-cyber/forget-lint) | — | `master` | 2026-07-20 13:53:20 |
| [gzmo-fresh-stack](https://github.com/maximilianwruhs-cyber/gzmo-fresh-stack) | Wipe-recovery handoff: fresh Knowledge OS distillation stack + OSS map + implementation plan | `main` | 2026-07-25 12:21:27 |
| [gzmo-observatory](https://github.com/maximilianwruhs-cyber/gzmo-observatory) | Live CT101 mind visualization for GZMO | `master` | 2026-07-16 11:08:35 |
| [GZMO-Pi-](https://github.com/maximilianwruhs-cyber/GZMO-Pi-) | GZMO curated memory for Pi — extension + MCP (pi-mcp-adapter); optional Redis/Neo4j/Qdrant living stack | `main` | 2026-07-18 19:28:14 |
| [herdr](https://github.com/maximilianwruhs-cyber/herdr) | agent multiplexer that lives in your terminal. | `master` | 2026-07-10 21:24:09 |
| [HSP-Pi-](https://github.com/maximilianwruhs-cyber/HSP-Pi-) | HSP Relay for Pi — silent record, one R2 settle chatter | `main` | 2026-07-18 19:44:15 |
| [Obolus-Arena](https://github.com/maximilianwruhs-cyber/Obolus-Arena) | Open lab harness for local SLM organs under the Obolus z metric (quality / joules × price) | `main` | 2026-07-21 18:35:55 |
| [okforge](https://github.com/maximilianwruhs-cyber/okforge) | Forgejo fork: OKF Knowledge Bundles by default with OKCP agent REST API | `main` | 2026-07-16 11:14:41 |
| [pdu-reflect](https://github.com/maximilianwruhs-cyber/pdu-reflect) | Prosecutor–Defender–Umpire Hegelian reflection for lab claims | `master` | 2026-07-20 13:53:17 |
| [stigmergy-queue](https://github.com/maximilianwruhs-cyber/stigmergy-queue) | Ideas→Architecture→Build→QA stigmergic folder pipeline (lab) | `master` | 2026-07-20 11:28:04 |
| [token-economy](https://github.com/maximilianwruhs-cyber/token-economy) | — | `master` | 2026-07-20 13:53:13 |
| [tool-chain](https://github.com/maximilianwruhs-cyber/tool-chain) | Follow vault/fact refs → bounded auto-read expansion (Tools Are Leaves closer) | `master` | 2026-07-20 13:53:23 |
| [trace-memory](https://github.com/maximilianwruhs-cyber/trace-memory) | Cross-task trace retrieve → inject strategies (lab) | `master` | 2026-07-20 09:34:10 |
| [verify-gates](https://github.com/maximilianwruhs-cyber/verify-gates) | — | `master` | 2026-07-20 09:18:24 |

---

## Documented coverage breakdown

| Bucket | How catalog covers them | Count in this inventory |
|--------|-------------------------|------------------------:|
| README “All GitHub projects (22)” | Ecosystem + standalone cards | 22 |
| Little Tools Lab tools | `LITTLE_TOOLS_LAB.md` rows 1–46 + `projects/<tool>.md` | 46 |
| little-tools-lab meta | README + LTL.md pointer; **no** project card | 1 (stale) |
| Missing from catalog | — | 15 |

### Ecosystem Members (from catalog files)

- **agenticos** (7): AOS, AOS-Customer-Edition, AOS-Intelligence-Dashboard, Obolus, HSP, HSP-VS-Codium-Extension, devstack_v2
- **gzmo-constellation** (10 Members): GZMO, gzmo_tinyFolder, gzmo_skills, mcp-neo4j-memory-gzmo, llama-cpp-prime-bench, edge-node, sovereign-agent, gzmo-core-clean, AttractorBench, gzmo-rebuild
- **meta-ops** (4): frankenstein, swap, maximilianwruhs-cyber, tinyFolder
- **standalone** (1): ki-assessment-tool
- **gzmo-constellation archived note**: tinyFolder

### README core list (unique)

`AOS`, `AOS-Customer-Edition`, `AOS-Intelligence-Dashboard`, `Obolus`, `HSP`, `HSP-VS-Codium-Extension`, `devstack_v2`, `GZMO`, `gzmo_tinyFolder`, `gzmo_skills`, `mcp-neo4j-memory-gzmo`, `edge-node`, `llama-cpp-prime-bench`, `sovereign-agent`, `gzmo-core-clean`, `AttractorBench`, `gzmo-rebuild`, `frankenstein`, `swap`, `maximilianwruhs-cyber`, `tinyFolder`, `ki-assessment-tool`

### LTL tool names (46)

`adaptive-tempo`, `baseline-bench`, `cabinet-sim`, `config-fuse`, `context-prune`, `dice-scheduler`, `draft-temp-bench`, `endpoint-scan`, `escape-loop-bench`, `etl-cli`, `evidence-locate`, `export-knowledge`, `faithfulness-judge`, `graph-ledger`, `honeypot-gate`, `hsp-probe`, `kg-reconcile`, `lorenz-map`, `mutation-queue`, `neural-finesse`, `organ-audit`, `pedagogy-bench`, `plan-gate`, `rapl-route`, `recall-eval`, `rem-substrate`, `rerank-probe`, `research-budget`, `rrf-recall`, `seed-curator`, `self-ask`, `session-distill`, `shadow-note`, `skill-patch`, `spark-link`, `speed-compare`, `spot-sweep`, `synapse-health`, `synapse-tail`, `temp-bench`, `tempo-bench`, `top-p-bench`, `trigger-sim`, `verify-suite`, `wiki-lint`, `zpd-tutor`

---

## Notes for later grilling (facts, not recommendations)

- Several missing repos have GitHub descriptions that name GZMO/HSP/Obolus relationships (`GZMO-Pi-`, `HSP-Pi-`, `gzmo-observatory`, `gzmo-fresh-stack`, `Obolus-Arena`, `forget-lint`, `pdu-reflect`, `token-economy`, `tool-chain`, `verify-gates`, `trace-memory`, `stigmergy-queue`) but are not indexed in `project-catalog`.
- `ados`, `herdr`, `okforge` are public with descriptions and no catalog index entry.
- No `projects/*.md` orphan: every project card stem matches a public GitHub repo name (case-insensitive).
- This note does not verify local clone freshness under `/home/gzmo/github-clone/` or contents of individual project cards beyond existence and membership links.

---

## Reproduction

```bash
gh api "users/maximilianwruhs-cyber/repos?per_page=100" --paginate
# Diff against: /home/gzmo/project-catalog/{README.md,ecosystems/*.md,projects/*.md,LITTLE_TOOLS_LAB.md}
```

