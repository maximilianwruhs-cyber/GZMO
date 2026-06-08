# Cloud Mode Diagnosis Report

**Datum:** 2026-06-07 16:16 (Pi) · **Korrigiert:** 2026-06-07 16:35 (Cursor)  
**Status:** Hintergrund-Cognition **cloud-first aktiv** (live verifiziert). Chat bleibt lokal-first.

> **KORREKTURHINWEIS (2026-06-07 16:35):** Die ursprüngliche Diagnose unten
> verwechselt zwei getrennte Mechanismen und ist überholt. `active_mode`
> steuert **nur den interaktiven Chat**; das Hintergrund-Routing wird seit
> heute durch `cloud_first_background = true` gesteuert und ist **live als
> cloud-first bewiesen** (siehe Abschnitt 0). Der `mode=local`-Logeintrag ist
> erwartet und kein Beweis gegen Cloud-Nutzung. Die Abschnitte 1–7 bleiben als
> historischer Kontext erhalten.

---

## 0. KORREKTUR — Live-Beweis (2026-06-07 16:35)

### Zwei getrennte Mechanismen

| Schalter | Steuert | Wert |
|----------|---------|------|
| `active_mode` (gzmo.toml Z.178) | **Nur** interaktiven Chat (`active_engine()`) | `local` |
| `cloud_first_background` (gzmo.toml `[routing]`) | **Alle** Hintergrund-Tasks (dream/spark/ingest/distill/daemon) | `true` |

Der `mode=local`-Logeintrag des Daemons stammt aus `active_mode` (config.rs:
`mode = %config.engine.active_mode`) und sagt **nichts** über das
Hintergrund-Routing aus.

### Architektur (seit 2026-06-07)

`GatewayRouter` umhüllt jeden Hintergrund-`TaskKind` als
`FallbackGateway(cloud → legacy)`: OpenRouter Nemotron zuerst, bei
Nichterreichbarkeit automatisch das bisherige Profil (Prime `:8000` bzw. VM200
Librarian). Fallback-`local`/`prime` ist fest an `[engine.local]` gepinnt,
unabhängig von `active_mode` — kein Rückfall in die Cloud. `Chat` ist
ausgenommen (Chat-Subagenten bleiben lokal).

### Live-Probe (nebenwirkungsfrei, echter Routing-Pfad)

Test: `gzmo-core/tests/live_cloud_probe.rs` (standardmäßig `#[ignore]`).

```
cargo test -p gzmo-core --test live_cloud_probe -- --ignored --nocapture
```

**Positiv** (`live_background_uses_cloud_first`):
```
[probe] cloud model=nvidia/nemotron-3-super-120b-a12b:free url=https://openrouter.ai/api/v1 key=sk-...122f
[probe] DIRECT cloud reply: PONG               # OpenRouter-Leaf live
[probe] BACKGROUND SparkHypothesis reply: PONG # echter Daemon-Pfad cloud-first
test result: ok
```

**Fallback** (`live_background_falls_back_to_prime_on_bad_cloud_key`):
```
[probe] FALLBACK reply (served by Prime): <think>   # Qwen3.6-Prime-Signatur, nicht Nemotron
test result: ok
```
Kaputter Cloud-Key → 401 → automatischer Failover auf Prime. Ohne Fallback
wäre der Test gescheitert.

### Empfehlung (revidiert)

`active_mode = "cloud"` ist für das Qualitätsziel (Hintergrund-Cognition über
OpenRouter) **nicht nötig** und würde den interaktiven Chat umstellen —
entgegen der Entscheidung "Chat bleibt lokal-first". Aktuelle Konfiguration ist
korrekt; keine Änderung erforderlich.

---

## 1. Konfigurationszustand

### `active_mode` in `gzmo.toml`
```toml
active_mode = "local"  # Zeile 178
```
**Fest:** Der Config-Wert ist auf `local` gesetzt.

### Cloud-Config vorhanden
```toml
[engine.cloud]
provider      = "openrouter"
url           = "https://openrouter.ai/api/v1"
model         = "nvidia/nemotron-3-super-120b-a12b:free"
api_key       = ""  # Leer — wird aus .env geladen
temperature   = 0.4
top_p         = 0.95
max_tokens    = 8192

fallback_provider  = "gemini"
fallback_url       = "https://generativelanguage.googleapis.com/v1beta/openai"
fallback_model     = "gemini-2.5-flash"
fallback_api_key   = ""
```

### `.env` — API Keys
| Key | Status |
|-----|--------|
| `GZMO_OPENROUTER_KEY` | ✅ Vorhanden (`sk-or-v1-...`) |
| `GZMO_SERPAPI_KEY` | ❌ Leer |
| `GZMO_GEMINI_KEY` | ❌ Leer |

---

## 2. Daemon-Log-Analyse

Letzte Daemon-Starts (alle auf `mode=local`):

| Zeit | Log-Eintrag |
|------|-------------|
| 2026-06-07 11:59 | `mode=local engine_url=http://localhost:8000/v1` |
| 2026-06-05 15:46 | `mode=local engine_url=http://localhost:8000/v1` |
| 2026-06-05 15:40 | `mode=local engine_url=http://localhost:8000/v1` |
| 2026-06-05 15:33 | `mode=local engine_url=http://localhost:8000/v1` |

**Kein einziger Eintrag** mit `mode=cloud` in den Logs.

---

## 3. Synapse Event-Log

In `data/Synapse/events.jsonl` gibt es einen Quest-Eintrag vom 2026-06-07 14:15, der diese Diagnose als Antwort enthält — derselbe, den ich gerade liefere. **Keine Cloud-Aktivität** in den Events.

---

## 4. verify-production.sh Ergebnis

```
[PASS] Prime :8000
[PASS] Daemon running
[PASS] gzmo health (llm)
[PASS] gzmo health (embeddings)
[PASS] gzmo health (neo4j)
[PASS] gzmo health (qdrant)
[FAIL] Redis scratch redis://192.168.31.202:6379
```

**Alle Probes gehen über `localhost:8000` (Prime lokal).** Keine Cloud-Requests.

---

## 5. Analyse: Warum war der Cloud-Mode nicht persistent?

Mögliche Szenarien:

### A. `/mode cloud` im TUI/REPL (wahrscheinlich)
- Der Befehl `/mode cloud` wechselt nur die **session-laufzeit**
- Wird **nicht** in `gzmo.toml` geschrieben
- Bei Daemon-Neustart oder Config-Reload → zurück zu `local`
- **Beweis:** Alle Daemon-Starts zeigen `mode=local`

### B. Daemon-Neustart zwischen Session und jetzt
- Daemon lädt `gzmo.toml` bei jedem Start neu
- Selbst wenn `/mode cloud` session-aktiv war → verloren nach Restart

### C. Session-Only TUI-Modus
- Wenn der Wechsel im GZMO TUI/REPL geschah (nicht Daemon)
- Session-Ende → Modus verloren

---

## 6. Empfohlene Aktionen

### Option 1: Cloud als Default setzen (persistent)
```toml
# In gzmo.toml Zeile 178 ändern:
active_mode = "cloud"
```
→ Cloud wird bei jedem Daemon-Start aktiv.

### Option 2: Cloud als Fallback konfigurieren
GZMO unterstützt `cloud_first_background = true` (Zeile 413) — prüfe ob das gewünschte Verhalten ist.

### Option 3: Session-Only (aktuell)
Weiterhin `/mode cloud` im TUI/REPL verwenden — nicht persistent, aber flexibel.

---

## 7. Fazit

**Cloud Mode war nicht aktiv.** Die Konfiguration ist vollständig vorhanden und der OpenRouter-Key ist gesetzt. Der Wechsel war wahrscheinlich session-aktiv im TUI/REPL und wurde nach Session-Ende oder Daemon-Neustart verworfen.

**Empfehlung:** Wenn Cloud permanent gewünscht ist, `active_mode = "cloud"` in `gzmo.toml` schreiben.

> **ÜBERHOLT — siehe Abschnitt 0.** Hintergrund-Cognition ist bereits cloud-first
> (`cloud_first_background = true`, live bewiesen). `active_mode = "cloud"` würde
> nur den interaktiven Chat umstellen und ist nicht erforderlich.

---

*Erstellt: 2026-06-07 16:16 by Pi Agent · Korrigiert: 2026-06-07 16:35 by Cursor (Live-Beweis)*
