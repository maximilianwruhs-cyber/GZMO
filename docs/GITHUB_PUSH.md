# GitHub push (local operator)

## Setup

```bash
cp .env.template .env.local   # or create manually
# Edit .env.local:
# GITHUB_TOKEN=ghp_…
```

`.env.local` is gitignored. **Never commit tokens.**

## Push

```bash
./scripts/push-github.sh feat/context-compress-headroom
# or default branch ref:
./scripts/push-github.sh HEAD
```

## Rotate token (if exposed)

1. https://github.com/settings/tokens
2. Revoke the compromised token
3. Create new classic or fine-grained PAT (Contents: read/write on `GZMO`)
4. Update **only** `.env.local`
5. Do not paste new tokens in chat

Current PR: https://github.com/maximilianwruhs-cyber/GZMO/pull/23
