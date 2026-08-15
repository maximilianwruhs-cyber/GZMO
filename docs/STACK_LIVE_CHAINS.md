# Stack Live Chains — 2026-08-15

**Alle 5 Chains live durchgespielt, ohne Plan-Updates, nur Execution.**

---

## Chain A: Stigmergy → Inference

```rust
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
```

**Ergebnis:** ✅ 200 Tokens sauber generiert, kein Loop

**Wert:** ⭐⭐⭐⭐⭐ Task-Delegation + Inference = höchste praktische Relevanz für Code-Produktion

---

## Chain B: ADOS Provenance (Keygen → Sign → Verify)

ADOS war schon in Chain A eingebettet — hier isoliert durchgespielt:

1. `generate_keypair` → sk: `ab2cf09a…`, pk: `5d5b8c4f…`
2. `show_trusted_keys` → 2 keys loaded (`9e1cac…`, `b4f346…`)
3. Sign → Verify lifecycle: Sign Task Receipt, Verify gegen Trusted Keys

**Wert:** ⭐⭐⭐⭐ Audit/Provenance. Kritisch für reproduzierbare Task-Pipelines, aber nur Backend

---

## Chain C: Energy-Aware Routing (Obolus RAPL → AOS Gateway → Model)

```
Obolus (90.6 J total)
 → AOS Gateway
   ├─ route_model(budget=5.0, task=code-gen) → qwen3.6-35b-mtp (balanced, 1.0 J/tok)
   ├─ route_model(budget=0.5, task=chat) → gzmo-flash (tiny, 0.02 J/tok)
   └─ Escalation ladder: WeakGenerator → Critic → StrongFallback (jeweils budget-gated)
```

**Wert:** ⭐⭐⭐⭐⭐ Energie-Routing ist einzigartiges USP des Stacks. Kein anderer Stack macht budget-bewusste Model-Selection live.

---

## Chain D: Living Memory + Takeaway

```
GZMO Vault (801 facts, Honeypot 618)
 → memory_search → recall/profile
   ├─ Find: "CT101 gzmo-daemon: authoritative source for ecosystem status"
   ├─ Profile: 995 Tokens aus static+dynamic+preferences+procedures
   └─ Brain Feed: enqueue durable insight (via bin/openclaw-takeaway.sh)
```

**Wert:** ⭐⭐⭐⭐⭐ Overnight-Metabolismus. Einzige Brücke zwischen Operator und CT101-Gehirn.

---

## Chain E: Full Autonomic Loop (alles kombiniert)

```
Stigmergy enqueue (task)
 → route_model (budget=5.0) → qwen3.6-35b-mtp
 → resolve_cast (planner+worker+critic)
 → resolve_escalation (Pass bei Budget > 0)
 → ADOS sign (task receipt)
 → HSP play_success
```

**Wert:** ⭐⭐⭐⭐⭐ Maximaler Stack-Nutzen: Jedes MCP wird genau einmal gebraucht, Output hat kryptographische Provenance.

---

## 🏆 Ranking

| Rang | Chain | Wert | Begründung |
|------|-------|------|------------|
| 1 | E — Full Autonomic Loop | ⭐⭐⭐⭐⭐ | Alle Tools in einem Durchlauf, maximaler Stack-Coverage |
| 2 | A — Stigmergy → Inference | ⭐⭐⭐⭐⭐ | Task-Pipeline + Code-Gen, direkt produktiv nutzbar |
| 3 | C — Energy Routing | ⭐⭐⭐⭐⭐ | Einziges USP des Stacks, kein anderer Stack machts |
| 4 | D — Living Memory | ⭐⭐⭐⭐ | Brücke zu CT101, aber langsam (SSH-Latenz) |
| 5 | B — ADOS Provenance | ⭐⭐⭐ | Wichtig für Audit, aber Overhead für Einmal-Tasks |

---

## Fazit

Die wertvollste Kombination ist **E → A → C**: Autonomic Loop mit Energy-Aware Routing, die ihre Ergebnisse signiert. Das ist der Stack-USP: ein Task-Pipeline, der Energie kostet, bewusst routed und kryptographisch belegt, was passiert ist.
