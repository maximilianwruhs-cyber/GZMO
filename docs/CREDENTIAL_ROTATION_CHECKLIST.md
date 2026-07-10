# Credential Rotation Checklist

**Created:** 2026-07-08  
**Reason:** Plaintext secrets were present in `infrastructure-report.md` (now redacted). Rotate any credentials that may have been exposed in chat logs, agent transcripts, or git history.

## Rotate immediately

| Secret | Where used | Action |
|--------|------------|--------|
| **OpenRouter API key** | CT101 `/opt/gzmo/.env` `GZMO_OPENROUTER_KEY` | Revoke at [openrouter.ai/keys](https://openrouter.ai/keys), generate new key, update `.env`, restart `gzmo-daemon` |
| **HuggingFace token** | Was in infrastructure report | Revoke at [huggingface.co/settings/tokens](https://huggingface.co/settings/tokens) if still active |
| **Proxmox root password** | PVE host `.200` | Change via `pveum passwd root` or web UI |
| **VM200 maximilian password** | ollamagpu `.110` | `passwd maximilian` on VM200 |
| **Neo4j password** | CT101 Docker + `.env` | Change in Neo4j + update compose + `/opt/gzmo/.env` + `gzmo.toml` MCP env |

## Prefer key-based auth (reduce password exposure)

1. Generate `~/.ssh/id_sidecar_proxmox` on workstation
2. Run `github-clone/swap/scripts/authorize_vm.py` for VM200
3. Add pubkey to Proxmox root `authorized_keys` for passwordless `pct exec`
4. Disable password auth on Proxmox once keys work (optional hardening)

## Verify after rotation

```bash
# CT101 daemon still healthy
ssh root@192.168.31.200 "pct exec 101 -- /opt/gzmo/survey_GZMO/target/release/gzmo health"

# Neo4j MCP still connects
ssh root@192.168.31.200 "pct exec 101 -- journalctl -u gzmo-daemon --since '2 min ago' | grep -i neo4j"
```

## Do not commit

- `/opt/gzmo/.env` on CT101
- Password manager exports
- This checklist does not contain live secrets
