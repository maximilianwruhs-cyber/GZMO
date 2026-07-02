# Abschlussbericht: thema_008 (Google Scholar Labs) — Integration in GZMO Sovereign Node

**Datum:** 2026-06-26  
**Status:** Phase 0–7 abgeschlossen (2026-06-26)  
**Repo:** `_foundation-audit/survey_GZMO`

---

## 1. Ausgangslage

### Research-Quelle (thema_008)

| Datei | Pfad |
|-------|------|
| Part 1 (EN Blueprint) | `/home/maximilian-wruhs/Schreibtisch/research/thema_008/part1.md` |
| Part 2 (DE Deep Dive) | `/home/maximilian-wruhs/Schreibtisch/research/thema_008/part2.md` |

Inhalt: 5-Layer Agentic Workflow für Google Scholar Labs — Playwright-Infrastruktur, Navigator Agent, Parsing/Extraktion, Multi-Turn Follow-up, Verifikation via OpenAlex/Crossref/Semantic Scholar/Unpaywall.

### Einordnung in die Research-Serie

| Thema | Inhalt | KB-Status (2026-06-26) |
|-------|--------|------------------------|
| thema_001 | Open Knowledge Format | ingest + OKF C-Full Handoff |
| thema_002 | Agentic Resource Discovery | ingest + ARD-Probe |
| thema_003 | Agent-Reach | ingest + Forum-Session |
| thema_004 | arXiv Preprint Ecosystem | ingest + Skill + Compliance + Discovery Phase 3 |
| **thema_008** | **Google Scholar Labs Agentic Workflow** | **ingest + Skill + Compliance + Discovery Phase 3 + Orchestrator** |

### Bewertungsrahmen (Direktive)

- [SOUL.md](../SOUL.md) — Sovereignty First, lokaler Core, Memory Discipline
- [ARCH-DIR-001-GZMO.md](ARCH-DIR-001-GZMO.md) — Sovereign Constitution, Obolus-Bilanz
- [THEMA_004_INTEGRATION_ABSCHLUSS.md](THEMA_004_INTEGRATION_ABSCHLUSS.md) — Preprint-Komplement (OAI-PMH, arXiv)
- **Operator-Vorgabe (2026-06-26):** Google Scholar Labs als experimentelle Plattform — Tier-2 Exception nur mit Operator-Policy-Dokumentation (Rate-Limits, Auth-Runbook)

---

## 2. Durchgeführte Änderungen

### 2.1 Compliance — Tier-2 Network Exception (Code)

**Herkunft:** Erweiterung von thema_004 Compliance um Scholar Labs und Verifikations-APIs.

| Datei | Änderung |
|-------|----------|
| [gzmo-core/src/config.rs](../gzmo-core/src/config.rs) | `default_network_exceptions()` erweitert um `"scholar"` |
| [gzmo-core/src/compliance.rs](../gzmo-core/src/compliance.rs) | `SCHOLAR_MARKERS` mit Google Scholar Labs, OpenAlex, Crossref, Semantic Scholar, Unpaywall |
| [gzmo.toml](../gzmo.toml) | Implizit via Default (keine explizite network_exceptions nötig) |
| Tests | 13/13 grün (`cargo test -p gzmo-core compliance::`) — 7 neue Tests für Scholar/OpenAlex/Crossref/S2/Unpaywall |
| Build | `cargo build --release -p gzmo-cli` erfolgreich |

**Verhalten nach Änderung:**

| Aktion | Sovereign-Modus (`allow_cloud_tools=false`) |
|--------|-----------------------------------------------|
| `scholar.google.com` via `skill_scholar.sh` | Erlaubt |
| `api.openalex.org` (Verifikation) | Erlaubt |
| `api.crossref.org` (DOI) | Erlaubt |
| `api.semanticscholar.org` | Erlaubt |
| `api.unpaywall.org` (OA PDF) | Erlaubt |
| `curl https://example.com` (generisch) | Blockiert |
| Vault / Honeypot / Prime | Unverändert lokal (Tier 1) |

### 2.2 KB-Ingest (thema_008)

**Herkunft:** Konsolidierung aus thema_008-Research (ohne Base64-Bilder aus Part 2).

| Artefakt | Pfad |
|----------|------|
| Curated Source | `~/Schreibtisch/knowledge/curated/thema_008-scholar-labs-topology.md` |
| Ingest-Befehl | `gzmo ingest …/thema_008-scholar-labs-topology.md` |
| Wiki-Source | [wiki/sources/thema-008-scholar-labs-topology.md](../wiki/sources/thema-008-scholar-labs-topology.md) — **16 Entities**, 1 Relation |
| Qdrant | honeypot (post-sync) |

Neue/abgeleitete Wiki-Entities: Google Scholar Labs, Navigator Agent, lit-review-orchestrator, paper-search-mcp, OpenAlex, Crossref, Unpaywall, Semantic Scholar (Verification Layer).

### 2.3 Playwright Driver (Python-Package)

**Herkunft:** thema_008 Part 1/2 Architektur-Blueprint.

| Artefakt | Pfad |
|----------|------|
| Auth Setup | `scripts/scholar_labs/auth_setup.py` |
| Query Engine | `scripts/scholar_labs/query.py` |
| HTML Parser | `scripts/scholar_labs/parse.py` |
| Follow-up Handler | `scripts/scholar_labs/followup.py` |
| Verifikation | `scripts/scholar_labs/verify.py` |
| Orchestrator | `scripts/scholar_labs/orchestrate.py` |
| Requirements | `scripts/scholar_labs/requirements.txt` |
| README | `scripts/scholar_labs/README.md` |

Cache-Struktur: `data/scholar-cache/{queries.jsonl, raw/, sessions/}` (gitignored).

### 2.4 Live Scholar Skill

**Herkunft:** thema_008 Part 1/2 + thema_004 Skill-Pattern.

| Artefakt | Pfad |
|----------|------|
| Skill-Script | [skills/skill_scholar.sh](../skills/skill_scholar.sh) |
| Slash-Command | `/scholar` in [skills/skills.toml](../skills/skills.toml) |
| Cache | `data/scholar-cache/` (gitignored) |
| Auth State | `playwright/.auth/google_state.json` (gitignored) |

Subcommands: `status`, `auth-setup`, `query`, `followup`, `verify`, `ingest-query`, `harvest`, `ingest-harvest`, `navigator-prompt`.

**Verifiziert:** `status` → zeigt Python-Deps, Auth-Status, Cache-Stats.

### 2.5 Harvest → Ingest Pipeline

**Herkunft:** Pattern aus thema_004 `build-arxiv-harvest-curated.py`.

| Artefakt | Pfad |
|----------|------|
| Build Script | `scripts/build-scholar-harvest-curated.py` |
| Output | `~/Schreibtisch/knowledge/curated/thema_008-scholar-harvest-*.md` |
| Skill Integration | `skill_scholar.sh ingest-harvest` |

Format: OKF-Style Markdown mit Verification-Status pro Paper.

### 2.6 Multi-Turn Orchestrator (Bibliothekars-Agent Runtime)

**Herkunft:** thema_008 Part 1 §4 (Multi-Turn Loop) + GZMO Wiki Bibliothekars-Agent.

| Artefakt | Pfad |
|----------|------|
| Orchestrator | `scripts/scholar_labs/orchestrate.py` |
| Pipeline | Navigator → Query → Gap Eval → Follow-up → Verify → Deduplicate → Batch |
| Navigator Prompt | `skill_scholar.sh navigator-prompt` |

Dies ist die **erste konkrete Bibliothekars-Agent Runtime** — orchestrierter Skill-Chain für agentische Literatur-Synthese.

### 2.7 Wiki-Dokumentation (manuell)

| Entity | Zweck |
|--------|-------|
| google-scholar-labs | thema_008-Kern-Entität |
| navigator-agent | Query Formulation Layer |
| lit-review-orchestrator | Referenz-Implementierung (GitHub) |
| paper-search-mcp | OA-First-Fallback-Chain Referenz |
| openalex | Verifikations-API |
| crossref | DOI-Verifikation |
| semantic-scholar | SPECTER-Embeddings + Zitationen |
| unpaywall | OA PDF Resolution |
| bibliothekars-agent | Verbindung zu thema_008 (USES) |
| librarian-agent | Verbindung zu OpenAlex (USES) |

Eintrag in [wiki/log.md](../wiki/log.md): `ingest | thema_008-scholar-labs-topology`.

### 2.8 Discovery Phase 3 (thema_008)

| Artefakt | Pfad |
|----------|------|
| Discovery-Prompt | `~/gzmo_skills/prompts/research/discovery-scholar-fit.md` |
| KB-Probe | `~/gzmo_skills/scripts/discovery-probes/probe-scholar-kb.sh` (B13) |
| pillars.json | `scholar\|scholar-labs\|openalex\|unpaywall\|navigator-agent\|bibliothekars-agent` in Pillar A/B |

---

## 3. Risk Matrix & Operator Policy

| Risk | Mitigation |
|------|------------|
| ToS / Account Lock | Rate sleep 3s zwischen Queries; kein automatisierter Login; Auth-Runbook Dokumentation |
| Bot-Detektion | Persistent Browser State (storage_state); kein Headless-Login-Loop; ggf. auth-setup wiederholen |
| Selector Drift (UI-Änderung) | Raw HTML Caching; versionierte Selector-Fallbacks in parse.py; `status` Health-Check |
| Auth Expiry | Detection via Login-Redirect; klare Error-Message mit "run auth-setup" |
| Halluzinierte Metadaten | Mandatory Verify-Pass vor Ingest; Levenshtein 0.85 Threshold; Unpaywall OA-Link |
| Obolus Blocks Shell Skill | Direkter Aufruf `./skills/skill_scholar.sh` dokumentiert; optional Rust-Registry später |

### Auth Runbook

```bash
# Einmalig: Google Login durchführen
./skills/skill_scholar.sh auth-setup
# → Browser öffnet sich → Login → Return to terminal → ENTER

# Auth Status prüfen
./skills/skill_scholar.sh status

# Bei "Session expired": Auth neu aufsetzen
./skills/skill_scholar.sh auth-setup
```

**Wichtig:** Niemals automatisierte Login-Versuche codieren (triggert 2FA/Bot-Detektion).

---

## 4. Smoke Test Sequence

```bash
# 1. Skill Status
./skills/skill_scholar.sh status

# 2. Auth Setup (einmalig, manuell)
./skills/skill_scholar.sh auth-setup

# 3. Single Query (ohne Ingest)
./skills/skill_scholar.sh query \
  --question "How do transformer architectures affect citation graph construction?" \
  --output /tmp/test_query.json

# 4. Verify Results
./skills/skill_scholar.sh verify \
  --input /tmp/test_query.json \
  --output /tmp/test_verified.json

# 5. Query → Verify → Curated → Ingest (Single Step)
./skills/skill_scholar.sh ingest-query \
  --question "How do microplastics affect gut microbiota in fish?"

# 6. Harvest von Question-List
./skills/skill_scholar.sh harvest \
  --questions-file ~/questions.txt \
  --output-dir /tmp/scholar_harvest

# 7. Harvest → Curated → Ingest
./skills/skill_scholar.sh ingest-harvest \
  --input-dir /tmp/scholar_harvest \
  --batch-size 50

# 8. Multi-Turn Orchestrator (Bibliothekars-Agent)
python scripts/scholar_labs/orchestrate.py \
  --topic "AI in radiology" \
  --max-turns 3 \
  --output-dir /tmp/orchestrated

# 9. Compliance Tests
cargo test -p gzmo-core compliance::

# 10. KB Recall (Discovery Probe)
~/gzmo_skills/scripts/discovery-probes/probe-scholar-kb.sh
```

---

## 5. Provenance-Matrix

| Entscheidung / Artefakt | Primärquelle |
|-------------------------|--------------|
| thema_008-Inhalt (5-Layer Arch) | Schreibtisch/research/thema_008/part{1,2}.md |
| Playwright Driver | thema_008 Part 1 + PyPI playwright/bs4/httpx/rapidfuzz |
| Compliance Extension (SCHOLAR_MARKERS) | thema_004 Pattern + Operator Tier-2 Policy |
| skill_scholar.sh | thema_004 skill_arxiv.sh Template |
| build-scholar-harvest-curated.py | thema_004 build-arxiv-harvest-curated.py Pattern |
| orchestrate.py | thema_008 Part 1 §4 + Bibliothekars-Agent KB-Entity |
| 16 Ingest-Entities | Prime extract+verify aus curated Doc |
| Tier-1 Strenge | SOUL.md, ARCH-DIR-001-GZMO.md |

---

## 6. Quick Reference

```bash
# Skill direkt
./skills/skill_scholar.sh status
./skills/skill_scholar.sh query --question "..." --output file.json
./skills/skill_scholar.sh verify --input file.json
./skills/skill_scholar.sh ingest-query --question "..."

# Über Chaos-Pantheon
# (Anmerkung: Rust Registry nicht implementiert — Shell-Direktaufruf empfohlen)

# KB nachladen
gzmo ingest ~/Schreibtisch/knowledge/curated/thema_008-scholar-labs-topology.md

# Python Dependencies
pip install -r scripts/scholar_labs/requirements.txt
playwright install chromium

# Compliance prüfen
cargo test -p gzmo-core compliance::
```

---

## 7. Fazit

**thema_008 ist vollständig integriert** unter der kanonischen Tier-2-Policy: lokaler Core (Tier 1) bleibt streng; Google Scholar Labs + Verifikations-APIs (Tier 2) haben permanenten Netzwerk-Zugriff; abgerufene Metadaten werden via Playwright Driver → Parser → Verifikation → Curated Markdown → Ingest in vault/honeypot überführt.

**Erledigt:** Compliance-Code (13 Tests), KB-Ingest (16 Entities), Wiki-Doku, Playwright Driver (6 Module), Live-Skill (9 Subcommands), Harvest-Pipeline, Multi-Turn Orchestrator, Discovery Phase 3.

**Complement zu thema_004:** arXiv (OAI-PMH, strukturierte Preprints) + Scholar Labs (semantische konversationelle Suche über publizierte Literatur) = vollständige Literaturabdeckung.

**Risk Governance:** Operator-Policy dokumentiert (Auth-Runbook, Rate-Limits, ToS-Awareness).

---

*Erstellt im Rahmen der thema_008-Integrationsarbeit · GZMO Sovereign Node · 2026-06-26*
