# Code Stitcher — Implementation Plan

**2026-08-15 · Priority: Highest First**

## Übersicht

Fünf Use Cases für Code Stitcher im GZMO/Operator-Kontext.
Die Implementierung folgt einer logischen Reihenfolge — jeder Schritt
baut auf dem vorherigen auf.

---

## Phase 1: MCP Bridge + Recipe Approval Gate

**Ziel:** Code Stitcher als MCP-Tool für den Agenten verfügbar machen,
mit Approval Gate das Max kontrolliert.

### Schritte

#### 1.1 Code Stitcher bauen und testen

```bash
cd /home/gzmo/Projects/code-stitcher
cargo test --all-features
cargo build --release
```

Prüfen: alle Tests grün, Clippy sauber.

#### 1.2 MCP Server aus `src/engine/mcp.rs` als Standalone-Binary

Aktuell ist `mcp.rs` ein Engine-Modul. Es braucht einen Einstiegspunkt
der als stdio MCP Server läuft — analog zu ADOS, Obolus, HSP.

**Minimales MCP Tool-Set:**

| Tool | Input | Output |
|------|-------|--------|
| `cs_ingest` | `source: String` → parsed Ingredient | Ingredient JSON (BLAKE3 id) |
| `cs_stitch` | `recipe: JSON` → validate → emit | Generated Rust source |
| `cs_verify` | `recipe: JSON` → verify integrity | Pass/Fail + reasons |
| `cs_list_ingredients` | - | Alle Ingredients im Store |
| `cs_list_recipes` | - | Verfügbare Recipes (approved/draft) |

#### 1.3 Approval Gate Workflow

- Recipes in `recipes/approved/` sind durchstichbar (mit `approved: true`)
- Recipes in `recipes/drafts/` sind nicht durchstichbar
- Max setzt `approved: true` → ich darf stitchen
- Trusted Keys aus `CODE_STITCHER_TRUSTED_PUBKEYS` oder `recipes/trusted_keys/*.hex`

#### 1.4 Integration ins GZMO Repo

- MCP Server als Binary-Baustein dokumentieren
- `gzmo.toml` um Code Stitcher MCP Fragment erweitern
- Startup-Script in `scripts/` (analog zu `pi-gzmo-mcp-serve.sh.override`)

**Status:** ⬜ Nicht begonnen

---

## Phase 2: Autonomic Task Pipeline (Stigmergy → Stitcher → ADOS)

**Ziel:** Code-Generierung durchläuft Stitcher-Qualitäts-Gate bevor
ADOS signiert und HSP abspielt.

### Schritte

#### 2.1 Stigmergy Task → Stitcher Recipe

Ein Task auf dem Stigmergy Board enthält statt rohem Prompt ein
Recipe-JSON. Der Worker ruft `cs_stitch` auf statt `route_model`.

#### 2.2 ADOS Signing für Recipes

- Recipe wird vor dem Stitch signiert
- ADOS `sign_envelope` signiert den Receipt (wie heute)
- Zusätzlich: Rezept-Hash im Receipt als Provenance-Link

#### 2.3 AOS Gateway Routing

- `route_model()` bestimmt Budget
- Budget ≥ 5.0 J → Stitcher wird verwendet (heavy tasks)
- Budget < 5.0 J → direkter LLM-Call (lightweight tasks)
- Escalation Ladder: Stitcher-Fail → Fallback auf LLM-Code-Gen

**Status:** ⬜ Nicht begonnen

---

## Phase 3: Skill Workshop Integration

**Ziel:** Skills als content-addressed Ingredients verwalten.

### Schritte

#### 3.1 Ingredients aus Skills generieren

Jeder Skill (Shell-Script, Rust-Snippet, Prozedur) wird beim
`skill_workshop create` als Ingredient in den Code Stitcher Store
aufgenommen — BLAKE3 gehasht, Interface extrahiert.

#### 3.2 Recipes aus Skill-Kombinationen

Ein Recipe beschreibt: "Nimm Skill A + Skill B → kombiniere sie".
`eml-core` könnte Metrik-Funktionen für Skill-Auswahl liefern.

#### 3.3 Deterministic Skill Emission

`cs_emit-source` produziert byte-identischen Code aus Skills →
reproduzierbare Skill-Ketten.

**Status:** ⬜ Nicht begonnen

---

## Phase 4: GZMO Pipeline Hooks

**Ziel:** Stitcher-generierte Formeln in der Memory Pipeline nutzen.

### Schritte

#### 4.1 EML + Stitcher für Honeypot Confidence

- `eml-core` liefert deterministische Confidence-Scores
- Code Stitcher stellt sie als signed Functions bereit
- Honeypot Gate importiert signed Functions statt LLM-Heuristik

#### 4.2 EML + Stitcher für Spark/Dream Decay

- Decay-Kurven als EML-Ausdrücke definiert
- Code Stitcher kompiliert sie zu native Functions
- Spark/Dream Engine ruft diese auf

**Status:** ⬜ Nicht begonnen

---

## Phase 5: Sicherheitskritische Code-Emission (Live)

**Ziel:** Ich produziere auditierten Code der vor Emission durch
das Approval Gate läuft.

### Schritte

#### 5.1 Auth-Audit Recipe als Demo

Das existierende `auth_audit_recipe.json` ist das Musterbeispiel:
- HMAC Password Verify
- HS256 JWT Generation
- Token Validation

Dieses Recipe durchläuft den gesamten Stitcher → ADOS → HSP Zyklus.

#### 5.2 Erweiterung auf allgemeine Code-Artifakte

Jeder write/edit-Befehl von mir könnte optional durch den Stitcher
laufen — für sicherheitskritische Dateien (Auth, Config mit Secrets).

#### 5.3 Trusted Key Rotation

- Max' Key(s) in `recipes/trusted_keys/`
- Rotation via Datei-Austausch + CI-Prüfung
- ADOS + Stitcher teilen sich die Key-Infrastruktur

**Status:** ⬜ Nicht begonnen

---

## Abhängigkeiten

```
Phase 1 (MCP Bridge) ← Phase 2 (Pipeline) ← Phase 4 (GZMO Hooks)
                    ← Phase 3 (Skills)
                    ← Phase 5 (Code Emission)
```

Phase 1 ist **Voraussetzung für alles andere**. Ohne MCP-Anbindung
kann ich als Agent nicht mit Code Stitcher interagieren.

---

## Nächste konkrete Schritte (ab sofort)

1. `cargo test` in code-stitcher laufen lassen → Status prüfen
2. `cargo build --release` → Binary existiert
3. `mcp.rs` lesen → verstehen was schon da ist
4. GZMO-Repo Dokumentation schreiben: `docs/CODE_STITCHER_MCP.md`
5. Ersten Test-Stitch mit `auth_audit_recipe.json` vorbereiten

Diese Schitte starte ich selbstständig, ohne vorheriges Approval.
