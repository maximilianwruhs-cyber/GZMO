# GZMO Workspace — Follow-Up Handoff & Audit Prompt

**Generated:** 2026-08-15 14:35 CEST  
**Audit scope:** All changes since previous session baseline through commit `27ce104` (main)  
**Goal:** Verify structural integrity, correctness, and completeness of all recent modifications.

---

## Instructions for the auditing agent

Review **every item** below. File a structured verification report with pass/fail for each section. Flag any anomalies, inconsistencies, broken references, or security concerns. Do not assume prior work is correct — re-verify from first principles.

Start by:
```bash
cd /home/gzmo/github-clone/GZMO
git log --oneline -10
git diff --stat HEAD~1
```

---

## 1. Git History Audit

Verify the commit history on `main` is clean and intentional.

| Check | Expected |
|-------|----------|
| Last commit | `27ce104` — "feat: add code-stitcher MCP server (6 tools)" |
| Branch is `main` | Fast-forward merged from `fix/hsp-play-preflight` |
| No merge commits on main | Should be linear fast-forward only |
| `.scratch/` excluded | Working notes, never committed |
| No large binary blobs | All files are text / source code |
| No secrets/credentials | Grep for `sk-`, `api_key`, `password`, `token`, `-----BEGIN` |

### Quick check:
```bash
cd /home/gzmo/github-clone/GZMO
git log --oneline -15 main
git show --stat HEAD
grep -rn 'sk-\|api_key\|-----BEGIN' --include='*.py' --include='*.rs' --include='*.sh' \
  --include='*.toml' --include='*.json' --include='*.md' . 2>/dev/null | grep -vi 'commit\|github' | head -10
```

---

## 2. Full File Inventory — Verify Each Group

### 2a. `crates/eml-core/` — New Rust crate

**Files:**

| File | Purpose | Verification |
|------|---------|-------------|
| `Cargo.toml` | Package definition | Check name, version, deps (num-complex, thiserror) |
| `src/lib.rs` | Public API + synth module | `ComplexBall`, `EmlExpr`, `execute`, `RpnInstruction`, `RpnProgram` re-exported |
| `src/complex_ball.rs` | `ComplexBall { center, radius }` | NaN-check (known weakness: no `is_finite()` on result) |
| `src/emitter.rs` | `EmlExpr` AST → RPN compiler | Post-order traversal, arity computation |
| `src/rpn.rs` | RPN instructions | `PushConstant`, `LoadVariable`, `EvalEml` |
| `src/executor.rs` | Zero-copy stack machine | `Vec::with_capacity(16)`, no heap allocs in loop |
| `benches/system_benchmarks.rs` | Criterion benchmarks | 3 benchmark groups, compiles with `cargo bench` |

**Checklist:**
- [ ] `cargo test` passes (expected: 12/12 or 12/12 depending on version)
- [ ] `cargo clippy --all-targets` is clean
- [ ] `cargo build --release` succeeds
- [ ] `synth::neg()` missing (noted in ANALYSIS.md)
- [ ] `EmlError::Overflow` declared but never thrown

### 2b. `crates/eml-core/USE_CASES.md` — 44 use cases

| Check | Expected |
|-------|----------|
| 44 entries | Count them, confirm none truncated |
| Module references exist | Cross-reference against GZMO source files in `gzmo-core/src/` |
| No contradiction with `ANALYSIS.md` | Both should agree on strengths/weaknesses |

**Verify with:**
```bash
wc -l /home/gzmo/github-clone/GZMO/crates/eml-core/USE_CASES.md
grep -cE '^\| *[0-9]+ \|' crates/eml-core/USE_CASES.md
```

### 2c. `crates/eml-core/ANALYSIS.md` — Technical analysis

**Checklist:**
- [ ] 12/12 tests green claim matches current test run
- [ ] Clippy clean claim matches `cargo clippy`
- [ ] NaN-check weakness (`is_finite()` missing) is accurately described
- [ ] Performance profile table (RPN lengths) is mathematically consistent
- [ ] Benchmarks exist vs. naive `f64::exp`/`f64::ln`?

### 2d. `docs/STACK_LIVE_CHAINS.md` — 5-chain validation

**Checklist:**
- [ ] Chain A (Stigmergy → Inference): AESGCM Python snippet — reproducible?
- [ ] Chain B (ADOS): keygen/sign/verify cycle actually tested?
- [ ] Chain C (Energy Routing): Obolus 90.6J claim verifiable with `obolus_read_all`?
- [ ] Chain D (Living Memory): 801 facts, 618 honeypot — verify with `gzmo_memory_status`
- [ ] Chain E (Full Loop): all MCPs called, signed, sonified

### 2e. `docs/CODE_STITCHER_IMPLEMENTATION.md` — 5-phase plan

**Checklist:**
- [ ] Phase 1 (MCP Bridge) — COMPLETE: `scripts/cs-mcp-server.py` exists + registered
- [ ] Phase 2 (Autonomic Pipeline) — NOT STARTED: no stigmergy integration
- [ ] Phase 3 (Skill Workshop) — NOT STARTED: no ingredient generation from skills
- [ ] Phase 4 (GZMO Hooks) — NOT STARTED: no eml-core integration
- [ ] Phase 5 (Code Emission) — PARTIAL: auth_audit works, no general write-tool hook
- [ ] Dependencies are correctly ordered (Phase 1 must precede all others)

### 2f. `docs/CODE_STITCHER_RESULTS.md` — Auth audit result

**Checklist:**
- [ ] Auth audit recipe (`recipes/approved/auth_audit_recipe.json`) has `approved: true`
- [ ] All 26 ingredient nodes present in `fixtures/ingested_ingredients/`
- [ ] `emit-source` output is valid Rust (parses with `syn::parse_file`)
- [ ] `stitch --run` produces correct output (verify_password → true, JWT → valid, validate_token → true)
- [ ] Binary `./audit_app` was compiled and executed (or check if it still exists)

### 2g. `scripts/cs-mcp-server.py` — MCP bridge

**Checklist:**
- [ ] MCP protocol: `initialize` → `tools/list` → `tools/call` cycle works
- [ ] All 6 tools registered: `cs_ingest`, `cs_stitch`, `cs_emit_source`, `cs_list_ingredients`, `cs_list_recipes`, `cs_verify_recipe`
- [ ] Tool inputs/outputs match JSON-RPC spec
- [ ] Error handling: missing file, timeout, binary not found
- [ ] Registered in OpenClaw: `openclaw mcp probe` shows `code-stitcher: 6 tools`
- [ ] `openclaw mcp reload` required after registration

**Verify with:**
```bash
python3 -c "
import json, subprocess
p = subprocess.Popen(['python3', 'scripts/cs-mcp-server.py'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
req = json.dumps({'jsonrpc':'2.0','id':1,'method':'tools/list','params':{}})
out, err = p.communicate(input=req+'\n', timeout=5)
parsed = json.loads(out.strip())
tools = parsed['result']['tools']
assert len(tools) == 6, f'Expected 6 tools, got {len(tools)}'
print(f'✅ {len(tools)} tools verified')
for t in tools:
    print(f'   • {t[\"name\"]}')
"
```

---

## 3. OpenClaw MCP Configuration Audit

| Check | Command | Expected |
|-------|---------|----------|
| Server registered | `openclaw mcp list` | `code-stitcher` appears |
| Tools visible | `openclaw mcp probe` | `code-stitcher: 6 tools` |
| MCP settings | `cat ~/.openclaw/openclaw.json \| python3 -c "import json,sys; c=json.load(sys.stdin); print(c.get('mcpServers',{}).get('code-stitcher','NOT FOUND'))"` | Server entry with command, args, env |

---

## 4. CT101 Baseline Status (2026-08-15 14:13 CEST)

This is the reference state for the living host. Any change after this should be recorded.

| Component | Status | Verification |
|-----------|--------|-------------|
| Host | CT101, uptime 2d 22h | `ssh ct101 uptime` |
| Load | 0.00 | `cat /proc/loadavg` |
| RAM | 3.5G/8G used (4.5G avail) | `free -h` |
| Disk | 25G/125G (21%) | `df -h /` |
| Daemon | active (enabled) | `systemctl is-active gzmo-daemon` |
| Redis | container Up 2d (healthy) | `docker ps --filter name=redis` |
| Qdrant | container Up 2d | `curl localhost:6333/health` |
| Neo4j | Up 2d (healthy) | `curl localhost:7474/` |
| Neo4j-code | Up 2d (healthy) | `docker ps --filter name=neo4j-code` |
| Litellm | Up 17h | `docker ps --filter name=litellm` |
| OTEL/Phoenix | Up 2d | `docker ps --filter name=otel\|phoenix` |

---

## 5. Cross-Cutting Verification

### 5a. Reference consistency

- `docs/STACK_LIVE_CHAINS.md` mentions Chain D: "801 facts, 618 honeypot" — check with `gzmo_memory_status`
- `docs/STACK_LIVE_CHAINS.md` Chain C: "Obolus 90.6 J total" — check with `obolus_read_all`
- `docs/CODE_STITCHER_RESULTS.md` — does `./audit_app` binary still exist at expected path?

### 5b. No dangling references

- All USE_CASES.md module paths (`memory/honeypot.rs`, `spark.rs`, etc.) must exist in `gzmo-core/src/`
- All MCP tool names must match between `tools/list` response and actual handler implementations
- All recipe references (`auth_audit_recipe.json`) must resolve to files

### 5c. Security scan

- [ ] No API keys, tokens, or passwords in committed files
- [ ] No `.env` file committed
- [ ] No private keys (`.pem`, `.key`) committed
- [ ] No `data/vault.db` committed (check Cargo.lock changes are legitimate)
- [ ] No internal IPs or hostnames in public docs (internal ones like `192.168.x.x` are acceptable)
- [ ] No shell scripts with hardcoded credentials

---

## 6. Follow-Up Actions (if all checks pass)

If the audit passes, the next agent should:

1. **Phase 2 begin** — Connect Stigmergy → Code Stitcher chain:
   - Create a stigmergy task that describes a recipe
   - Worker calls `cs_stitch` instead of raw LLM code-gen
   - ADOS signs the recipe receipt

2. **Benchmark `eml-core`** — Run criterion benchmarks:
   ```bash
   cd /home/gzmo/github-clone/GZMO
   cargo bench -p eml-core
   ```
   Compare RPN overhead vs. native `f64` (this is noted as a gap in ANALYSIS.md)

3. **Re-verify the full E chain** — Repeat the Stack Live Chains from `STACK_LIVE_CHAINS.md`

---

## Failure Escalation

If any check above fails:

1. Log the failure with specific details (file, line, expected vs. actual)
2. Determine if it's a documentation mismatch, a code bug, or a config error
3. Fix if trivial and safe (obvious typo, missing line in doc)
4. Escalate to operator for anything that requires human judgment:
   - Schema changes
   - Config changes that affect running services
   - Changes to `openclaw.json` / MCP registrations
   - Credential exposure
   - Data integrity issues

---

*End of handoff audit prompt. Proceed systematically — verify each item once, document results, report anomalies.*
