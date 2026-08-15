# EML Core — Technische Analyse

**Datum:** 2026-08-15  
**Tests:** 12/12 grün · Clippy sauber

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

5. **12 Tests + Criterion-Benchmarks** — solide Abdeckung.

---

## Schwächen / TODO

1. **`complex_ball.rs` — NaN-Check zu spartanisch:**
   `ln(0.0)` gibt `-inf` (IEEE 754 valide), wird durchgelassen. Aber `exp(-inf)` = 0,
   dann `0 - (-inf)` = `+inf` — das crasht nicht, aber `center.re.is_nan()` fängt
   `inf - inf` nicht. Ein `is_finite()`-Check auf dem finalen Center wäre robuster.

2. **Kein `synth::neg`:** Für `neg(y)` wird inline `sub(c(0.0), y)` in `add` verwendet.
   Ein eigenes `pub fn neg` würde Redundanz vermeiden.

3. **`EmlExpr.compile()` linearisiert nur — keine Optimierung:**
   Constant Folding, Dead Code Elimination oder `eml(x, 1)` → `exp(x)` als pattern match
   fehlen. Für tiefe Bäume (>50 Knoten) wäre Constant Folding ein Gewinn.

4. **Fehlerarten:** `EmlError::Overflow` ist deklariert aber nie gethrowed —
   `exp(1000.0)` gibt `inf` ohne Fehler.

5. **Benchmarks:** Die Criterion-Benchmarks sind da, aber es gibt keine Comparative
   Benchmarks gegen naive `f64::exp` / `f64::ln` Implementierungen — genau das wäre
   spannend für die Softmax-Hypothese.

---

## Performance-Profil (geschätzt)

| Operation | RPN-Länge | Relative Kosten |
|-----------|-----------|-----------------|
| `exp(x)`  | 2         | ~1×             |
| `ln(x)`   | 5         | ~2.5×           |
| `x + y`   | ~11       | ~5.5×           |
| `x × y`   | ~13       | ~7×             |
| `x / y`   | ~17       | ~9×             |
| `pow(x,y)`| ~19       | ~10×            |

---

## Einschätzung

Das Crate ist ein **Proof-of-Concept auf Master-Niveau** — compiler-korrekter
AST→RPN→VM-Stack, präzise Fehlerfortpflanzung.

Für den produktiven Einsatz fehlen:

- NaN/inf-Robustheit in `ComplexBall::eml`
- Constant Folding
- Softmax-Benchmark gegen naive Implementierung
- Makro-DSL (`eml!(x, 1)` → `EmlExpr::eml(EmlExpr::v(0), EmlExpr::c(1.0))`)

Der USP bleibt die Idee: _Ein_ Operator der genügt, um alle Analysis zu bauen.
Der Overhead ist bei reellen Zahlen unter Double-Epsilon (~1e-15), bei komplexen
Zahlen aber noch ungemessen.
