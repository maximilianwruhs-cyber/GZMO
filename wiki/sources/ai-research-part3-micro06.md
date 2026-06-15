---
type: source
title: ai-research-part3-micro06
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# ai-research-part3-micro06

Ingested source summary (2026-06-09).

## Entities
- [[reset-to-commit|reset_to_commit]] (TOOL)
- [[python-module-not-found|python_module_not_found]] (CONCEPT)
- [[pytest|pytest]] (TOOL)
- [[agenticsystem|AgenticSystem]] (SYSTEM)
- [[apply-patch|apply_patch]] (TOOL)
- [[test-failure|test_failure]] (CONCEPT)
- [[diff-versus-commit|diff_versus_commit]] (TOOL)
- [[open|open]] (TOOL)
- [[coding-agent-py|coding_agent.py]] (SYSTEM)
- [[os-path-join|os.path.join]] (TOOL)
- [[diagnose-errors|diagnose_errors]] (TOOL)
- [[python-syntax-error|python_syntax_error]] (CONCEPT)
- [[attempt-error-resolution|attempt_error_resolution]] (CONCEPT)
- [[utils-git-utils|utils.git_utils]] (SYSTEM)
- [[tools-edit|tools.edit]] (SYSTEM)
- [[ai-research-part3|ai-research-part3]] (PROJECT)
- [[cloud-kg-extraction|cloud KG extraction]] (CONCEPT)
- [[tools-bash|tools.bash]] (SYSTEM)
- [[apply-automated-fix|apply_automated_fix]] (TOOL)

## Relations
- ai-research-part3 → RELATED_TO → cloud KG extraction
- diagnose_errors → PART_OF → tools.bash
- apply_automated_fix → PART_OF → tools.edit
- attempt_error_resolution → USES → diagnose_errors
- attempt_error_resolution → USES → apply_automated_fix
- attempt_error_resolution → RELATED_TO → python_module_not_found
- attempt_error_resolution → RELATED_TO → python_syntax_error
- attempt_error_resolution → RELATED_TO → test_failure
- AgenticSystem → USES → attempt_error_resolution
- AgenticSystem → PART_OF → coding_agent.py
- coding_agent.py → USES → utils.git_utils
- coding_agent.py → USES → tools.bash
- coding_agent.py → USES → tools.edit
- diff_versus_commit → PART_OF → utils.git_utils
- reset_to_commit → PART_OF → utils.git_utils
- apply_patch → PART_OF → utils.git_utils
