---
type: source
title: drive-research-pi-coding-agent-local-deployment-customization
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-pi-coding-agent-local-deployment-customization

Ingested source summary (2026-06-08).

## Entities
- [[wezterm|WezTerm]] (TOOL)
- [[silence-detection-algorithm|silence detection algorithm]] (CONCEPT)
- [[earendil-works-pi-ai|@earendil-works/pi-ai]] (TOOL)
- [[bash-ts|bash.ts]] (TOOL)
- [[xdg-base-directory-specification|XDG Base Directory Specification]] (CONCEPT)
- [[anthropic-js|anthropic.js]] (TOOL)
- [[ghostty|Ghostty]] (TOOL)
- [[user-query|User Query]] (CONCEPT)
- [[settings-manager-ts|settings-manager.ts]] (TOOL)
- [[agent-skills-standard|Agent Skills standard]] (CONCEPT)
- [[blargh|blargh]] (TOOL)
- [[amazon-bedrock|Amazon Bedrock]] (SYSTEM)
- [[earendil-works-pi-coding-agent|@earendil-works/pi-coding-agent]] (SYSTEM)
- [[csi-u-extended-keys|CSI-u Extended Keys]] (CONCEPT)
- [[google-js|google.js]] (TOOL)
- [[romansix-pi-tmux|@romansix/pi-tmux]] (TOOL)
- [[openai-js|openai-*.js]] (TOOL)
- [[deepseek|DeepSeek]] (ORGANIZATION)
- [[vllm|vLLM]] (TOOL)
- [[gondolin|Gondolin]] (SYSTEM)
- [[earendil-works-pi-agent-core|@earendil-works/pi-agent-core]] (TOOL)
- [[anthropic-messages|Anthropic Messages]] (CONCEPT)
- [[edit|edit]] (TOOL)
- [[docker|Docker]] (SYSTEM)
- [[aws-bedrock|AWS Bedrock]] (ORGANIZATION)
- [[mario-zechner|Mario Zechner]] (PERSON)
- [[openai-responses|OpenAI Responses]] (CONCEPT)
- [[claude-md|CLAUDE.md]] (CONCEPT)
- [[macos-apple-silicon|macOS Apple Silicon]] (SYSTEM)
- [[chatgpt-plus-pro|ChatGPT Plus/Pro]] (ORGANIZATION)
- [[github-copilot|GitHub Copilot]] (TOOL)
- [[azure-openai|Azure OpenAI]] (ORGANIZATION)
- [[rsa-oaep|RSA-OAEP]] (CONCEPT)
- [[google-gemini|Google Gemini]] (ORGANIZATION)
- [[offline-ant-pi-tmux|offline-ant/pi-tmux]] (TOOL)
- [[vs-code-integrated-terminal|VS Code Integrated Terminal]] (TOOL)
- [[claude-code|Claude Code]] (TOOL)
- [[pi-chat|pi-chat]] (TOOL)
- [[semaphores|semaphores]] (CONCEPT)
- [[terminus|Terminus]] (SYSTEM)
- [[openrouter|OpenRouter]] (ORGANIZATION)
- [[qemu|QEMU]] (TOOL)
- [[npm|npm]] (TOOL)
- [[db-query-executor|db-query-executor]] (CONCEPT)
- [[claude-pro-max|Claude Pro/Max]] (ORGANIZATION)
- [[earendil-works-pi-tui|@earendil-works/pi-tui]] (TOOL)
- [[skill-md|SKILL.md]] (CONCEPT)
- [[terminalbench|TerminalBench]] (CONCEPT)
- [[aes-256-gcm|AES-256-GCM]] (CONCEPT)
- [[intellij-idea-terminal|IntelliJ IDEA Terminal]] (TOOL)
- [[discord|Discord]] (ORGANIZATION)
- [[ollama|Ollama]] (TOOL)
- [[chat-request-secret|chat_request_secret]] (TOOL)
- [[agent-session-ts|agent-session.ts]] (TOOL)
- [[node-js|Node.js]] (SYSTEM)
- [[cwe-78|CWE-78]] (CONCEPT)
- [[tmux-agents-workspace|tmux-agents workspace]] (CONCEPT)
- [[agents-md|AGENTS.md]] (CONCEPT)
- [[lm-studio|LM Studio]] (TOOL)
- [[openai-completions|OpenAI Completions]] (CONCEPT)
- [[windows-terminal|Windows Terminal]] (TOOL)
- [[pi-dev-secret|pi.dev/secret]] (CONCEPT)
- [[google-generative-ai|Google Generative AI]] (CONCEPT)
- [[slack|Slack]] (ORGANIZATION)
- [[alpine-linux|Alpine Linux]] (SYSTEM)
- [[write|write]] (TOOL)
- [[read|read]] (TOOL)

## Relations
- Mario Zechner → AUTHORED_BY → @earendil-works/pi-coding-agent
- @earendil-works/pi-coding-agent → PART_OF → @earendil-works/pi-ai
- @earendil-works/pi-coding-agent → RELATED_TO → Claude Code
- @earendil-works/pi-agent-core → PART_OF → @earendil-works/pi-ai
- @earendil-works/pi-tui → PART_OF → @earendil-works/pi-ai
- @earendil-works/pi-coding-agent → USES → read
- @earendil-works/pi-coding-agent → USES → write
- @earendil-works/pi-coding-agent → USES → edit
- @earendil-works/pi-coding-agent → USES → bash.ts
- @earendil-works/pi-coding-agent → USES → Node.js
- @earendil-works/pi-coding-agent → USES → npm
- Mario Zechner → AUTHORED_BY → blargh
- @earendil-works/pi-ai → USES → OpenAI Completions
- @earendil-works/pi-ai → USES → OpenAI Responses
- @earendil-works/pi-ai → USES → Anthropic Messages
- @earendil-works/pi-ai → USES → Google Generative AI
- @earendil-works/pi-coding-agent → USES → Ollama
- @earendil-works/pi-coding-agent → USES → LM Studio
- @earendil-works/pi-coding-agent → USES → vLLM
- @earendil-works/pi-coding-agent → USES → Docker
- @earendil-works/pi-coding-agent → USES → Alpine Linux
- @earendil-works/pi-coding-agent → USES → macOS Apple Silicon
- Amazon Bedrock → RELATED_TO → anthropic.js
- Amazon Bedrock → RELATED_TO → openai-*.js
- Amazon Bedrock → RELATED_TO → google.js
- @earendil-works/pi-coding-agent → USES → XDG Base Directory Specification
- @earendil-works/pi-coding-agent → USES → AGENTS.md
- @earendil-works/pi-coding-agent → USES → CLAUDE.md
- CWE-78 → RELATED_TO → settings-manager.ts
- settings-manager.ts → RELATED_TO → agent-session.ts
- settings-manager.ts → RELATED_TO → bash.ts
- @earendil-works/pi-coding-agent → USES → VS Code Integrated Terminal
- VS Code Integrated Terminal → RELATED_TO → Ghostty
- WezTerm → USES → @earendil-works/pi-coding-agent
- Windows Terminal → USES → @earendil-works/pi-coding-agent
- IntelliJ IDEA Terminal → USES → @earendil-works/pi-coding-agent
- @earendil-works/pi-coding-agent → USES → @romansix/pi-tmux
- @earendil-works/pi-coding-agent → USES → Agent Skills standard
- Agent Skills standard → USES → SKILL.md
- @earendil-works/pi-coding-agent → USES → db-query-executor
- @earendil-works/pi-coding-agent → USES → tmux-agents workspace
- tmux-agents workspace → USES → offline-ant/pi-tmux
- @romansix/pi-tmux → USES → @earendil-works/pi-coding-agent
- @romansix/pi-tmux → USES → semaphores
- @romansix/pi-tmux → USES → silence detection algorithm
- @earendil-works/pi-coding-agent → USES → Claude Pro/Max
- @earendil-works/pi-coding-agent → USES → ChatGPT Plus/Pro
- @earendil-works/pi-coding-agent → USES → GitHub Copilot
- anthropic.js → USES → Anthropic Messages
- openai-*.js → USES → OpenAI Completions
- DeepSeek → USES → OpenAI Completions
- Google Gemini → USES → Google Generative AI
- Azure OpenAI → USES → OpenAI Responses
- OpenRouter → USES → OpenAI Completions
- pi-chat → PART_OF → Gondolin
- Gondolin → USES → Slack
- Gondolin → USES → Discord
- Gondolin → USES → QEMU
- QEMU → USES → Alpine Linux
- pi-chat → USES → chat_request_secret
- chat_request_secret → RELATED_TO → pi.dev/secret
- pi-chat → USES → RSA-OAEP
- pi-chat → USES → AES-256-GCM
- @earendil-works/pi-coding-agent → RELATED_TO → TerminalBench
- Terminus → RELATED_TO → TerminalBench
- @earendil-works/pi-coding-agent → USES → CSI-u Extended Keys
