# Quality Probe — PRECOG State Injection on Mamba-130m

## Model
- `mamba-130m-q8_0` (155 MB, Q8_0)
- Run concurrently with production `qwen3.8-27b` server on :8000
- GPU: RTX 5070 Ti (16 GB), ~1.7 GB free at run time → only 130m fit (790m needs ~593 MB + compute headroom; 7B out of reach)

## Corpus
- 16,898 tokens from 8 real GZMO docs (ADR-0003 through ADR-0008 + 2 runbooks)
- State file: 2,869,892 bytes (~2.87 MB)

## 5 Questions + Answers

### Q1: What does ADR-0004 describe as the key USP of the airgapped living system?
**Full-prefill answer:**
> ADR-0004 describes the airgapped living system. [repeats "A:"]

**Injection answer:**
> The key USP of the airgapped living system is the ability to detect and track the presence of airgapped living systems.

**Correct answer (from corpus):** Full living Keep on one airgapped box — local Prime/embed, local Redis/Qdrant/Neo4j, overnight writer, agents attach via local MCP.

**Assessment:** Neither answer is correct. The 130m model fails to extract the specific "sovereign overnight memory metabolism on hardware the operator owns" concept. The injection answer at least engages with the question topic ("airgapped living systems") but hallucinates about "detect and track."

---

### Q2: What is the flywheel approach in ADR-0005 replacing?
**Full-prefill answer:**
> The flywheel approach is a good one for the flywheel approach. [repeats]

**Injection answer:**
> The flywheel approach is a very simple approach to the problem. It is a simple method to get the desired output.

**Correct answer (from corpus):** Replaces frozen topology (CT101 permanently the only living host) with a mutex claim model — living host is a claimable role, not a fixed host.

**Assessment:** Neither answer is correct. Both are generic nonsense. The model cannot reason about the flywheel/frozen-topology distinction.

---

### Q3: What process lock mechanism does ADR-0006 use for the living writer?
**Full-prefill answer:**
> The ADR-0006 is a process lock. [echoes the question]

**Injection answer:**
> The living writer is a process that is responsible for writing the live text.

**Correct answer (from corpus):** `{vault_db}.write.lock` + owner socket.

**Assessment:** Neither answer captures the specific lock file + owner socket mechanism. The injection answer is slightly more topically relevant (mentions "living writer" + "writing") but hallucinates.

---

### Q4: What does ADR-0007 say about the lite SKU?
**Full-prefill answer:**
> The lite SKU is the same as the lite version of the ADR-0007. [circular]

**Injection answer:**
> It is a generic ADR-0007, which is a generic ADR-0007. [circular]

**Correct answer (from corpus):** There is no lite SKU. Clients attach to the living writer; `~/.gzmo` is incomplete install / telescope scratch, not a product.

**Assessment:** Neither answer is correct. Both are circular/tautological.

---

### Q5: What does ADR-0008 describe regarding edge SSM memory?
**Full-prefill answer:**
> The ADR-0008 is a new feature in the ADR-0008. It is a new feature that allows for the ADR-0008 to be used in the same way as the ADR-0008. [circular]

**Injection answer:**
> ADR-0008 is a new feature in the ADR-000 series of memory controllers. It is a new feature that allows the use of ADR-0008 memory controllers to supp[ort...]

**Correct answer (from corpus):** Edge SSM (state-space model) memory — using mamba/recurrent architectures at the edge for constant-memory context.

**Assessment:** The injection answer is arguably slightly closer — it mentions "memory controllers" — but still nonsensical.

---

## Honest Note

**Quality at 130m is INDICATIVE ONLY.** The mamba-130m model is a 130-million-parameter general language model — it is far too small to perform document QA or extract specific architectural decisions from technical prose. Both full-prefill and injection answers are low-quality across all 5 questions, with no meaningful quality difference between the two conditions.

**The claim under test is the LATENCY mechanism**, not quality. The bench demonstrates that the PRECOG state-injection mechanism works at the infrastructure level: a 16,898-token corpus state can be saved (2.87 MB file), restored into a fresh context, and queried with a 66.72× TTFT speedup (276 ms → 4.1 ms prefill). Quality at 7B-class (e.g., Mamba-Codestral-7B) is a **separate gate** that requires sufficient free GPU memory or a dedicated benchmark window with the production server stopped.

**No quality parity claim can be made from this data.** The next gate is running the same 5-question probe on a 7B-class mamba model to determine whether state-injection preserves answer quality equivalently to full-prefill.
