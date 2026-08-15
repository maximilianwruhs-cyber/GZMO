# EML Core — Technische Analyse

**Datum:** 2026-08-15  
**Tests:** crate unit suite (lib + pipeline_hooks) · Clippy `-D warnings` standalone

---

## Inspiration

Prof. Odrzywołeks Paper — `eml(x, y) = exp(x) − ln(y)` als universelles Primitiv,
analog zu NAND in der booleschen Logik.

---

## Architektur (4 Module)

| Modul | Verantwortung | Zeilen |
|--------------------|-----------------------------------------------------------------------|--------|
| `complex_ball.rs`  | `ComplexBall { center, radius }` mit EML-Operation + Fehlerfortpflanzung | ~55    |
| `emitter.rs`       | `EmlExpr`-AST → Post-Order-Traversal → flaches `RpnProgram`            | ~70    |
| `rpn.rs`           | 3 Instruktionen: `PushConstant`, `LoadVariable`, `EvalEml`              | ~55    |
| `executor.rs`      | Zero-Copy Stack-Maschine über das RPN-Programm                         | ~70    |

---

## Stärken

1. **Saubere Trennung:** AST → RPN → Execution ist klassische Compiler-Architektur, jede Stufe testbar.

2. **Zero-Copy Hot Path:** Der Executor allokiert nur einen `Vec` mit `capacity(16)` — keine Heap-Allokationen im Instruction-Loop.

3. **Precision-Drift Tracking:** `ComplexBall` propagiert Fehlergrenzen via
   ```
   Δresult ≈ |exp(x)|·Δx + (1/|y|)·Δy + 1 ULP
   ```
   — elegant und numerisch sinnvoll.

4. **`synth`-Modul:** Sämtliche elementaren Funktionen (exp, ln, +, −, ×, ÷, sqrt, pow, inv) sind aus _einem_ Operator synthetisiert und getestet.

5. **Unit tests + Criterion** — arity (`v(0)` → 1, constants → 0, sparse), empty args, NaN, final overflow, `synth::neg`, constant-fold of closed terms, `Display`/parse. `vs_f64` bench group exists; run it before claiming speed.

---

## Schwächen / TODO

1. **`ln(0)` is IEEE continuation, not a refuse.** `synth::neg` / `add` pass
   `ln(0) = -inf` as the left operand. `ComplexBall::eml` may return an infinite
   center. `execute` refuses a non-finite *final* result as `EmlError::Overflow`.
   Real-fast-path `exp(1000)` now overflows instead of dying as complex `NaN`.

2. **Radius is not rigorous.** `RADIUS_IS_RIGOROUS == false`. First-order plus
   `1e-15`. Do not treat a small radius as enclosure.

3. **Fold is closed-term evaluation, not a CAS.** No `eml(ln(x), exp(y)) → x-y`
   rewrite. Partial trees with variables are only recursively folded in children.

4. **`USE_CASES.md` is not a roadmap.** 44 rows are speculative. No `gzmo-core`
   callers. Do not wire honeypot/RRF/orchestrator from that table.

5. **`vs_f64` kill condition fired (2026-08-15, this host, Criterion medians,
   `black_box` on live inputs — first run was invalid: `f64` was const-folded
   to ~1.55 ns for exp/ln/mul).**

   | Op | EML | `f64` | Multiplier |
   |----|-----|-------|------------|
   | `exp(2)` | 237 ns | 17.0 ns | **14×** |
   | `ln(5)` | 560 ns | 19.2 ns | **29×** |
   | `mul(2,3)` | 2.90 µs | 2.38 ns | **1200×** |

   EML is not a faster float. Do not replace honeypot/RRF/`f64` scores with it.

---

## Performance-Profil (exakte RPN-Instruktionen)

| Operation    | RPN-Länge (Instruktionen) | Relative Kosten (Basis `exp`) |
|--------------|---------------------------|-------------------------------|
| `exp(x)`     | 3                         | 1.0×                          |
| `ln(x)`      | 7                         | ~2.3×                         |
| `sub(x, y)`  | 11                        | ~3.7×                         |
| `add(x, y)`  | 21                        | ~7.0×                         |
| `div(x, y)`  | 25                        | ~8.3×                         |
| `inv(x)`     | 25                        | ~8.3×                         |
| `mul(x, y)`  | 35                        | ~11.7×                        |
| `square(x)`  | 35                        | ~11.7×                        |
| `pow(x, y)`  | 43                        | ~14.3×                        |
| `sqrt(x)`    | 67                        | ~22.3×                        |

---

## Einschätzung

Workspace R&D math crate: AST→RPN→stack is real. It is **not** a Keep organ.

Next only if a measured win exists: algebraic rewrite that beats `f64`, or one
serialized formula-IR call site that can refuse a wide radius. Otherwise extract
as a paper crate and stop growing it inside GZMO.
