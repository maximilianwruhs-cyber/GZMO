## Summary

<!-- 1–3 bullets: why this change, not just what -->

## Living takeaway (side-effect — not a second chat)

If this PR came from a real work session that already burned Cursor credits, leave **one** durable fact for the living vault when you close — do **not** open a memory-gym agent.

```bash
# Prefer living host / CT101 enqueue (no --now while CT101 owns overnight)
gzmo session close --takeaway '…one durable fact…'
# or herdr close-ritual:
# herdr plugin pane open --plugin gzmo.metabolism --entrypoint close-ritual
```

- [ ] One living takeaway enqueued (or N/A: docs-only / no session)
- [ ] No second Cursor chat whose only job was “feed the vault”

## Test plan

- [ ] `bash scripts/brain-feed-check.sh` (or mission-specific gate)
- [ ] CI green
