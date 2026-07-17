# Core insight provenance stub

**Canonical document:** [`docs/CORE_INSIGHT.md`](../docs/CORE_INSIGHT.md)

Vault/honeypot rows seeded from that document use `source_file = manual/core_insight_20260717.md`
so recall provenance stays stable even if the markdown path is edited later.

Do not duplicate fact cards here — edit `docs/CORE_INSIGHT.md` and re-run:

```bash
python3 scripts/seed-core-stack.py --doc docs/CORE_INSIGHT.md --db data-next/vault.db
```
