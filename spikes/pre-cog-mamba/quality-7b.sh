#!/usr/bin/env bash
# quality-7b.sh — 7B quality probe with judge pass.
# Uses answers from bench-7b.json, sends each to the workstation :8000 judge.
set -euo pipefail
cd "$(dirname "$0")"

JUDGE_URL="http://127.0.0.1:8000/v1/chat/completions"
QUESTIONS=(
  "What does ADR-0004 describe as the key USP of the airgapped living system?"
  "What is the flywheel approach in ADR-0005 replacing?"
  "What process lock mechanism does ADR-0006 use for the living writer?"
  "What does ADR-0007 say about the lite SKU?"
  "What does ADR-0008 describe regarding edge SSM memory?"
)

CORPUS_SUMMARY="The corpus contains 8 GZMO ADRs and runbooks. Key facts:
- ADR-0004: Airgapped living system USP = full living Keep on one airgapped box (local Prime/embed, local Redis/Qdrant/Neo4j, overnight writer, agents attach via local MCP). Sovereign overnight memory metabolism on hardware the operator owns.
- ADR-0005: Flywheel replaces frozen topology (CT101 permanently the only living host) with a mutex claim model — living host is a claimable role, not a fixed host.
- ADR-0006: Process lock = {vault_db}.write.lock + owner socket for the living writer.
- ADR-0007: No lite SKU. Clients attach to the living writer; ~/.gzmo is incomplete install/telescope scratch, not a product.
- ADR-0008: Edge SSM memory = using mamba/recurrent (state-space model) architectures at the edge for constant-memory context."

if [ ! -f bench-7b.json ]; then
  echo "ERROR: bench-7b.json not found"
  exit 1
fi

python3 <<'PYEOF'
import json, sys, os, urllib.request

JUDGE_URL = "http://127.0.0.1:8000/v1/chat/completions"

QUESTIONS = [
    "What does ADR-0004 describe as the key USP of the airgapped living system?",
    "What is the flywheel approach in ADR-0005 replacing?",
    "What process lock mechanism does ADR-0006 use for the living writer?",
    "What does ADR-0007 say about the lite SKU?",
    "What does ADR-0008 describe regarding edge SSM memory?",
]

CORRECT_ANSWERS = [
    "Full living Keep on one airgapped box — local Prime/embed, local Redis/Qdrant/Neo4j, overnight writer, agents attach via local MCP. Sovereign overnight memory metabolism on hardware the operator owns.",
    "Replaces frozen topology (CT101 permanently the only living host) with a mutex claim model — living host is a claimable role, not a fixed host.",
    "{vault_db}.write.lock + owner socket.",
    "There is no lite SKU. Clients attach to the living writer; ~/.gzmo is incomplete install/telescope scratch, not a product.",
    "Edge SSM (state-space model) memory — using mamba/recurrent architectures at the edge for constant-memory context.",
]

CORPUS_SUMMARY = "The corpus contains 8 GZMO ADRs and runbooks with the following correct answers to the questions."

with open("bench-7b.json") as f:
    bench = json.load(f)

full_answers = bench["full_prefill"]["answers"]
inj_answers = bench["injection"]["answers"]

def judge_answer(question, answer, correct_answer):
    """Call the workstation 27B judge to score the answer 0-2."""
    prompt = f"""You are a strict judge evaluating an AI answer to a question about a technical corpus.

Question: {question}

Expected correct answer: {correct_answer}

AI answer to evaluate: {answer}

Score this answer 0-2:
- 0: Completely wrong, irrelevant, or nonsensical
- 1: Partially correct — touches the right topic but misses key specifics
- 2: Correct — captures the key point(s) of the expected answer

Respond in EXACTLY this format:
SCORE: <0 or 1 or 2>
REASON: <one line reason>"""

    payload = json.dumps({
        "model": "qwen3.8-27b",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.0,
        "max_tokens": 100
    }).encode("utf-8")

    req = urllib.request.Request(JUDGE_URL, data=payload, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            data = json.loads(resp.read())
        content = data["choices"][0]["message"]["content"]
        # Parse SCORE and REASON
        score = 0
        reason = ""
        for line in content.split("\n"):
            if line.strip().startswith("SCORE:"):
                try:
                    score = int(line.strip().split(":")[1].strip())
                except:
                    score = 0
            elif line.strip().startswith("REASON:"):
                reason = line.strip().split(":", 1)[1].strip()
        return score, reason
    except Exception as e:
        return -1, f"judge error: {e}"

# Run judge on all answers
results = []
print("=== Judging full-prefill answers ===")
for i, (q, ans) in enumerate(zip(QUESTIONS, full_answers)):
    score, reason = judge_answer(q, ans, CORRECT_ANSWERS[i])
    print(f"  Q{i+1}: score={score} reason={reason}")
    results.append({"question": q, "mode": "full-prefill", "answer": ans, "score": score, "reason": reason, "correct": CORRECT_ANSWERS[i]})

print("\n=== Judging injection answers ===")
for i, (q, ans) in enumerate(zip(QUESTIONS, inj_answers)):
    score, reason = judge_answer(q, ans, CORRECT_ANSWERS[i])
    print(f"  Q{i+1}: score={score} reason={reason}")
    results.append({"question": q, "mode": "injection", "answer": ans, "score": score, "reason": reason, "correct": CORRECT_ANSWERS[i]})

# Build quality-7b.md
md = []
md.append("# Quality Probe — PRECOG State Injection on Mamba-Codestral-7B\n\n")
md.append("## Model\n")
md.append(f"- `Mamba-Codestral-7B-v0.1-Q4_0` ({bench.get('state_size_bytes',0) > 0 and 'state injectable' or 'no state'})\n")
md.append(f"- Run on VM200 GTX 1070 (8 GB, ~5.7 GB free for model)\n")
md.append(f"- llama.cpp build b9018 (rebuilt from pre-refactor commit c84e6d6db)\n")
md.append(f"- Judge: workstation Qwen3.8-27B on :8000 (read-only)\n\n")

md.append("## Corpus\n")
md.append(f"- {bench['corpus_tokens']} tokens from 8 real GZMO ADRs + runbooks\n")
md.append(f"- State file: {bench['state_size_bytes']:,} bytes ({bench['state_size_bytes']/1024/1024:.1f} MB)\n")
md.append(f"- Restore verified: {bench.get('restore_verified', 'unknown')}\n\n")

md.append("## 5 Questions × 2 Modes — Full Answers\n\n")

for i, q in enumerate(QUESTIONS):
    full_r = results[i]
    inj_r = results[i + 5]
    md.append(f"### Q{i+1}: {q}\n\n")
    md.append(f"**Correct answer (from corpus):** {CORRECT_ANSWERS[i]}\n\n")
    md.append(f"**Full-prefill answer:**\n> {full_r['answer'][:300]}\n\n")
    md.append(f"**Injection answer:**\n> {inj_r['answer'][:300]}\n\n")
    md.append(f"**Judge — full-prefill:** score={full_r['score']}/2 — {full_r['reason']}\n\n")
    md.append(f"**Judge — injection:** score={inj_r['score']}/2 — {inj_r['reason']}\n\n")
    md.append("---\n\n")

# Verdict table
md.append("## Judge Verdict Table\n\n")
md.append("| Q# | Question | Full-prefill score | Injection score | Parity? |\n")
md.append("|---|---|---|---|---|\n")
for i, q in enumerate(QUESTIONS):
    fs = results[i]["score"]
    is_ = results[i + 5]["score"]
    parity = "YES" if fs == is_ else "NO"
    md.append(f"| Q{i+1} | {q[:50]} | {fs}/2 | {is_}/2 | {parity} |\n")

full_scores = [results[i]["score"] for i in range(5)]
inj_scores = [results[i + 5]["score"] for i in range(5)]
md.append(f"\n**Total: full-prefill {sum(full_scores)}/10, injection {sum(inj_scores)}/10**\n\n")

parity_count = sum(1 for i in range(5) if full_scores[i] == inj_scores[i])
md.append(f"**Parity: {parity_count}/5 questions have equal scores**\n\n")

# Assessment
md.append("## Assessment\n\n")
full_total = sum(full_scores)
inj_total = sum(inj_scores)

if full_total >= 6 and inj_total >= 6:
    md.append("Both full-prefill and injection produce **adequate quality** answers (≥6/10). ")
    if parity_count >= 4:
        md.append("State injection **preserves answer quality** — scores are equal or near-equal across all questions. ")
        md.append("**Quality parity holds at 7B.**\n\n")
    else:
        md.append(f"State injection shows some quality variation ({parity_count}/5 parity). ")
        md.append("Quality parity is **partial** — injection preserves most but not all quality.\n\n")
elif full_total >= 6 and inj_total < 6:
    md.append(f"Full-prefill produces adequate quality ({full_total}/10) but injection degrades to {inj_total}/10. ")
    md.append("**Quality parity does NOT hold** — state injection loses answer quality.\n\n")
elif full_total < 6:
    md.append(f"Even full-prefill produces low quality ({full_total}/10). The model cannot adequately answer these questions from the corpus. ")
    if parity_count >= 4:
        md.append("However, injection **does not further degrade** quality — parity is maintained at a low level.\n\n")
    else:
        md.append(f"Injection quality ({inj_total}/10) differs from full-prefill ({full_total}/10).\n\n")

with open("quality-7b.md", "w") as f:
    f.write("".join(md))

print(f"\n=== quality-7b.md written ===")
print(f"Full-prefill: {sum(full_scores)}/10")
print(f"Injection: {sum(inj_scores)}/10")
print(f"Parity: {parity_count}/5")
PYEOF
