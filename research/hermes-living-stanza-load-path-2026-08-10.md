# Hermes living stanza load path (2026-08-10)

**Ticket:** [#152 Hermes living stanza load path](https://github.com/maximilianwruhs-cyber/GZMO/issues/152) (map [#151](https://github.com/maximilianwruhs-cyber/GZMO/issues/151))  
**Host probed:** workstation user `gzmo` (`$HOME=/home/gzmo`)  
**Hermes source pin:** [NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent) `@03fa32c92dd445eb64c7f67434dd91b32c40701d` (shallow clone for this note)  
**Status:** Answer for later tickets — do not re-probe unless Hermes release notes change config paths.

---

## Answer gist

Hermes Agent loads `mcp_servers` **only** from **`$HERMES_HOME/config.yaml`** (default **`~/.hermes/config.yaml`**). It does **not** read **`~/.hermes.toml`**. On this host today: `~/.hermes.toml` exists (dead scar), `~/.hermes/` is absent, and the `hermes` binary is not on `PATH` — so no live Hermes MCP load is happening.

---

## 1. Which product is “Hermes”?

| Claim | Evidence |
|-------|----------|
| “Hermes” in GZMO living-attach docs means **NousResearch Hermes Agent** (CLI `hermes`), not a GZMO-invented “Hermes Protocol” / OpenClaw MCP layer | Official docs: [MCP feature guide](https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp); repo `NousResearch/hermes-agent`; herdr agent id `hermes` / alias `hermes-agent` in `~/.local/state/herdr/agent-detection/remote/hermes.toml` |
| Folklore that “`.hermes.toml` is the standard MCP config for GZMO/OpenClaw” is **wrong** | Local scar docs under `~/tmp/*telegram*` / `~/tmp/hermes-explanation.md` / `~/tmp/clarification-of-setup.md` claim this; Hermes source has **zero** references to `hermes.toml` / `.hermes.toml` (rg over clone `@03fa32c`) |

---

## 2. Canonical MCP load path (Hermes primary sources)

### 2.1 File + key

| Claim | Evidence |
|-------|----------|
| MCP servers live under key `mcp_servers` in `config.yaml` | Docs: “Hermes reads MCP config from `~/.hermes/config.yaml` under `mcp_servers`” — [website/docs/user-guide/features/mcp.md](https://github.com/NousResearch/hermes-agent/blob/main/website/docs/user-guide/features/mcp.md) (line ~342 in clone); same in published site |
| Config path is `get_hermes_home() / "config.yaml"` | `hermes_constants.get_config_path()` → `return get_hermes_home() / "config.yaml"` ([`hermes_constants.py`](https://github.com/NousResearch/hermes-agent/blob/03fa32c92dd445eb64c7f67434dd91b32c40701d/hermes_constants.py) ~1293–1299); re-export in [`hermes_cli/config.py`](https://github.com/NousResearch/hermes-agent/blob/03fa32c92dd445eb64c7f67434dd91b32c40701d/hermes_cli/config.py) ~694–696 |
| Runtime MCP registration reads that config via `load_config()` then `config.get("mcp_servers")` | [`tools/mcp_tool.py`](https://github.com/NousResearch/hermes-agent/blob/03fa32c92dd445eb64c7f67434dd91b32c40701d/tools/mcp_tool.py) `_load_mcp_config()` ~4985–5025 |
| CLI/docs directory tree shows `~/.hermes/config.yaml` as the settings file | [website/docs/user-guide/configuration.md](https://github.com/NousResearch/hermes-agent/blob/main/website/docs/user-guide/configuration.md) “Directory Structure” |

**Living attach implication:** paste the GZMO stanza under `mcp_servers.gzmo-living` **inside** `~/.hermes/config.yaml` (YAML), not into a root-home `~/.hermes.toml`.

### 2.2 How `HERMES_HOME` is resolved

| Precedence (highest first) | Evidence |
|----------------------------|----------|
| 1. Context-local override (`set_hermes_home_override`) | `get_hermes_home()` docstring + body — [`hermes_constants.py`](https://github.com/NousResearch/hermes-agent/blob/03fa32c92dd445eb64c7f67434dd91b32c40701d/hermes_constants.py) ~114–139 |
| 2. Env `HERMES_HOME` | `_hermes_home_from_env()` same file ~62–74 |
| 3. Platform default: Linux/macOS → `Path.home() / ".hermes"` | `_get_platform_default_hermes_home()` ~53–59 |

Installer default data home is also `$HOME/.hermes` (`HERMES_HOME="${HERMES_HOME:-$HOME/.hermes}"` in [`scripts/install.sh`](https://github.com/NousResearch/hermes-agent/blob/main/scripts/install.sh)). Code install defaults to `$HERMES_HOME/hermes-agent` for non-root.

Profiles: a profile is a **separate Hermes home directory** with its own `config.yaml` (docs: [profiles](https://hermes-agent.nousresearch.com/docs/user-guide/profiles)). MCP for that profile is still `{that_home}/config.yaml`, not a second file format.

### 2.3 What is *not* a Hermes MCP config file

| Path | Role for Hermes MCP? | Evidence |
|------|----------------------|----------|
| `~/.hermes.toml` | **Never loaded** | No matches in Hermes source for `hermes.toml` / `.hermes.toml` |
| `~/.codex/config.toml` `[mcp_servers.*]` | Import **source** only (`hermes import-agent codex` → writes Hermes `config.yaml`) | [`hermes_cli/agent_import.py`](https://github.com/NousResearch/hermes-agent/blob/03fa32c92dd445eb64c7f67434dd91b32c40701d/hermes_cli/agent_import.py) header mapping lines 27–29 |
| OpenClaw / Cursor JSON homes | Separate clients; OpenClaw **migrate-from-Hermes** reads `~/.hermes/config.yaml` | OpenClaw docs `migrating-hermes.md`: detects `~/.hermes`, imports MCP from `mcp_servers` or `mcp.servers`; `discoverHermesSource` sets `configPath` to `…/config.yaml` |

---

## 3. Precedence when more than one “config” exists

There is **no** Hermes precedence between `~/.hermes.toml` and `~/.hermes/config.yaml` because the former is not a load path.

Within Hermes itself, relevant precedence for MCP:

| Rule | Evidence |
|------|----------|
| Settings stack (CLI > `config.yaml` > `.env` > defaults) applies to Hermes settings generally; MCP server **definitions** are the `mcp_servers` map in `config.yaml` | [configuration.md “Configuration Precedence”](https://github.com/NousResearch/hermes-agent/blob/main/website/docs/user-guide/configuration.md) |
| `HERMES_SAFE_MODE` → `_load_mcp_config()` returns `{}` (no MCP) | `tools/mcp_tool.py` ~5000–5001 |
| Plugin “portable” MCP servers are merged **after** native `mcp_servers`; if a name already exists in native config, portable is **skipped** (native wins) | `tools/mcp_tool.py` ~5018–5027 (`conflicts with native config; skipping`) |
| `--ignore-user-config` / equivalent: ignore `~/.hermes/config.yaml` and use built-in defaults (`.env` credentials still load) | CLI help in `hermes_cli/_parser.py` |

**Not observed / not applicable:** dual-file merge of TOML + YAML; YAML does not fall back to `~/.hermes.toml`.

---

## 4. On-disk evidence (this host, 2026-08-10)

| Probe | Result |
|-------|--------|
| `command -v hermes` / `~/.local/bin/hermes` | **Missing** (not installed on PATH now) |
| `~/.hermes/` | **Absent** (no `config.yaml`, no install tree under `~/.hermes/hermes-agent`) |
| `~/.hermes.toml` | **Present** (307 bytes, birth 2026-08-09, modified 2026-08-10); TOML `[mcp_servers.gzmo-living]` → `scripts/pi-gzmo-mcp-serve.sh`, `GZMO_LIVING=1` |
| `HERMES_*` env | **None** set in probe shell |
| `bash_history` | Prior `hermes` invocations and `curl … NousResearch/hermes-agent …/install.sh \| bash` — install was attempted historically; current tree does not retain `~/.hermes` |
| herdr | Still ships remote detection rules for agent id `hermes` / `hermes-agent` |
| OpenClaw migrate expectation | Would look for `~/.hermes/config.yaml`; that path does not exist here |

**Conclusion for this host:** any living stanza written only to `~/.hermes.toml` is **inert** for Hermes Agent. To attach living once Hermes is (re)installed: create/edit **`~/.hermes/config.yaml`** with YAML `mcp_servers.gzmo-living: …` (or `hermes mcp add` / paste from emitter).

---

## 5. GZMO docs/scripts vs Hermes truth

| Artifact | What it claims | Fit |
|----------|----------------|-----|
| [`docs/EXTERNAL_LIVING_ATTACH.md`](../docs/EXTERNAL_LIVING_ATTACH.md) | Emit Hermes fragment; paste under `mcp_servers.gzmo-living` | Correct **key/shape**; does not spell `~/.hermes/config.yaml` in the happy-path bullets (implied by “Hermes”) |
| [`docs/examples/hermes-gzmo-living.yaml`](../docs/examples/hermes-gzmo-living.yaml) | YAML `mcp_servers:` stanza; “does not rewrite `~/.hermes`” | Correct dialect for `config.yaml` |
| [`scripts/emit-living-mcp-fragment.sh`](../scripts/emit-living-mcp-fragment.sh) | Emits Hermes YAML; “does not touch `~/.hermes`” | Correct safety posture; output is YAML for `config.yaml`, not TOML |
| [`research/living-external-attach-plug-and-play-2026-07-22.md`](./living-external-attach-plug-and-play-2026-07-22.md) | Hermes home `~/.hermes/config.yaml`; “do not thrash `~/.hermes` from scripts” | **Aligned** with Hermes source |
| Host scar `~/.hermes.toml` + `~/tmp/*` Telegram notes | Treat TOML at `$HOME/.hermes.toml` as the MCP home | **Misaligned** folklore — ignore for spec/installers |

**Spec lock recommendation (for #151):** Hermes living home = **`~/.hermes/config.yaml`** (`mcp_servers.gzmo-living`). Refuse/document that **`~/.hermes.toml` is not a Hermes load path** (cleanup optional follow-on).

---

## 6. Operator checklist (no re-research needed)

```bash
# Prove Hermes home + MCP file (after install)
echo "HERMES_HOME=${HERMES_HOME:-$HOME/.hermes}"
test -f "${HERMES_HOME:-$HOME/.hermes}/config.yaml" && echo OK_config_yaml
command -v hermes

# Emit living stanza (repo) and paste into THAT yaml under mcp_servers:
bash scripts/emit-living-mcp-fragment.sh --format hermes
bash scripts/living-attach-check.sh

# Dead scar (optional awareness only):
test -f "$HOME/.hermes.toml" && echo 'WARN: ~/.hermes.toml is NOT read by Hermes Agent'
```

---

## 7. Citation index (primary)

1. Hermes MCP docs — `~/.hermes/config.yaml` + `mcp_servers`: [mcp.md](https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp), source [`website/docs/user-guide/features/mcp.md`](https://github.com/NousResearch/hermes-agent/blob/main/website/docs/user-guide/features/mcp.md)  
2. Hermes configuration home / precedence: [`website/docs/user-guide/configuration.md`](https://github.com/NousResearch/hermes-agent/blob/main/website/docs/user-guide/configuration.md)  
3. `get_hermes_home` / `get_config_path`: [`hermes_constants.py`](https://github.com/NousResearch/hermes-agent/blob/03fa32c92dd445eb64c7f67434dd91b32c40701d/hermes_constants.py)  
4. `_load_mcp_config`: [`tools/mcp_tool.py`](https://github.com/NousResearch/hermes-agent/blob/03fa32c92dd445eb64c7f67434dd91b32c40701d/tools/mcp_tool.py)  
5. Installer `HERMES_HOME` default: [`scripts/install.sh`](https://github.com/NousResearch/hermes-agent/blob/main/scripts/install.sh)  
6. OpenClaw Hermes migrate (expects `~/.hermes` + `config.yaml`): bundled `openclaw/docs/install/migrating-hermes.md` on this host’s OpenClaw install  
7. Host probes: presence of `/home/gzmo/.hermes.toml`, absence of `/home/gzmo/.hermes/`, absence of `hermes` on `PATH` (2026-08-10)  
8. GZMO prior research aligning on `config.yaml`: [`research/living-external-attach-plug-and-play-2026-07-22.md`](./living-external-attach-plug-and-play-2026-07-22.md)
