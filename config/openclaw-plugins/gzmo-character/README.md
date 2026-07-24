# openclaw-plugin-gzmo-character

Registers the `character` agent tool used by Telegram `/character`.

Runs `scripts/openclaw-choose-character.sh` with `OPENCLAW_CHARACTER_FORCE=1`
so pack installs never wipe GZMO `AGENTS.md`.

## Install (workstation)

```bash
cd config/openclaw-plugins/gzmo-character && npm install
openclaw plugins install --link --force "$(pwd)"
openclaw plugins enable gzmo-character
systemctl --user restart openclaw-gateway.service
```

Skill frontmatter uses `command-dispatch: tool` → `command-tool: character`
so `/character list` bypasses the local model.
