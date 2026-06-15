# `/calculate` v2 Formatter Contract

**Scope:** Display and JSON evidence only. `eval_via_bc` remains the single numeric truth.

## When to show steps

Show Schritt-fuer-Schritt decomposition when the normalized expression contains:

- Parentheses, or
- At least one binary operator (`+`, `-`, `*`, `/`, `^`)

Single-token expressions (`42`, `2^10`, `sqrt(144)`) may still show one Klammer step for parens/sqrt.

## Step order

1. Innermost parentheses (evaluated via `bc -l`)
2. `*` and `/` left to right
3. `+` and `-` left to right

## German labels

| Field | Text |
|-------|------|
| Step N | `Schritt N` |
| Paren step | `Schritt N (Klammer)` |
| Final line | `Endergebnis` |
| INTEGER | `Ganzzahl — exaktes Ergebnis ohne Nachkommastellen.` |
| FRACTIONAL | `Bruchzahl — Dezimalanteil vorhanden.` |
| HUGE | `Sehr gross — Betrag ab 10^12.` |
| TINY | `Sehr klein — Betrag unter 10^-6.` |

## JSON evidence (`--json`)

Additive fields on v1 shape:

```json
{
  "skill": "calculate",
  "version": 2,
  "steps": [
    { "label_de": "Schritt 1", "expr": "3*4", "partial": "12" }
  ],
  "interpretation": "Ganzzahl — ..."
}
```

## Out of scope (deferred)

- Natural-language expression parser
- Live API integrations (Wolfram, etc.)
- Synapse `skill.*` emission (handled by Pi synapse-notifier)
