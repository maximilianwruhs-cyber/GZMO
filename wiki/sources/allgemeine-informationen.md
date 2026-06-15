---
type: source
title: allgemeine-informationen
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# allgemeine-informationen

Ingested source summary (2026-06-08).

## Entities
- [[bash|Bash]] (TOOL)
- [[redis|Redis]] (SYSTEM)
- [[devdocs|DevDocs]] (TOOL)
- [[ghcr-io-hkuds-nanobot-ai|ghcr.io/hkuds/nanobot-ai]] (ORGANIZATION)
- [[grafana|Grafana]] (TOOL)
- [[dev-containers|Dev Containers]] (TOOL)
- [[snyk-code|Snyk Code]] (TOOL)
- [[ai-assisted-automation|AI-assisted automation]] (CONCEPT)
- [[sast|SAST]] (CONCEPT)
- [[nanobot-cli-commands|nanobot.cli.commands]] (SYSTEM)
- [[roo-cline|Roo-Cline]] (TOOL)
- [[github-actions|GitHub Actions]] (TOOL)
- [[wecom-aibot-sdk-python|wecom-aibot-sdk-python]] (TOOL)
- [[linear|Linear]] (TOOL)
- [[litellm|LITELLM]] (SYSTEM)
- [[gpt-4o|gpt-4o]] (CONCEPT)
- [[openai|OpenAI]] (ORGANIZATION)
- [[cursor|Cursor]] (TOOL)
- [[typer-cli|Typer CLI]] (TOOL)
- [[terraform|Terraform]] (TOOL)
- [[sentry|Sentry]] (TOOL)
- [[wechat|WECHAT]] (SYSTEM)
- [[sqlite|SQLite]] (SYSTEM)
- [[qrcode|qrcode]] (TOOL)
- [[docker-compose|Docker Compose]] (TOOL)
- [[claude-code|Claude Code]] (TOOL)
- [[gitops|GitOps]] (CONCEPT)
- [[vs-code|VS Code]] (TOOL)
- [[spotify-backstage|Spotify Backstage]] (TOOL)
- [[slack|Slack]] (TOOL)
- [[n8n|n8n]] (TOOL)
- [[gitlab-ci-cd|GitLab CI/CD]] (TOOL)
- [[windsurf|Windsurf]] (TOOL)
- [[pycryptodome|pycryptodome]] (TOOL)
- [[nanobot-gateway|nanobot-gateway]] (SYSTEM)
- [[platform-engineering|platform engineering]] (CONCEPT)
- [[github-copilot|GitHub Copilot]] (TOOL)
- [[asyncio|asyncio]] (CONCEPT)
- [[langsmith|langsmith]] (TOOL)
- [[port|Port]] (TOOL)
- [[postman|Postman]] (TOOL)
- [[makefiles|Makefiles]] (TOOL)
- [[pip|pip]] (TOOL)

## Relations
- nanobot-gateway → USES → ghcr.io/hkuds/nanobot-ai
- LITELLM → USES → gpt-4o
- nanobot.cli.commands → USES → Redis
- nanobot.cli.commands → USES → SQLite
- nanobot.cli.commands → USES → OpenAI
- qrcode → RELATED_TO → WECHAT
- pycryptodome → RELATED_TO → WECHAT
- GitHub Copilot → RELATED_TO → AI-assisted automation
- Claude Code → RELATED_TO → AI-assisted automation
- Roo-Cline → RELATED_TO → AI-assisted automation
- GitLab CI/CD → USES → GitOps
- Dev Containers → USES → Docker Compose
- Linear → USES → GitHub Actions
- Linear → USES → Slack
