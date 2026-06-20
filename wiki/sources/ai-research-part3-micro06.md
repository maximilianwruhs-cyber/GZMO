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
- [reset_to_commit](/entities/reset-to-commit.md) (TOOL)
- [python_module_not_found](/entities/python-module-not-found.md) (CONCEPT)
- [pytest](/entities/pytest.md) (TOOL)
- [AgenticSystem](/entities/agenticsystem.md) (SYSTEM)
- [apply_patch](/entities/apply-patch.md) (TOOL)
- [test_failure](/entities/test-failure.md) (CONCEPT)
- [diff_versus_commit](/entities/diff-versus-commit.md) (TOOL)
- [open](/entities/open.md) (TOOL)
- [coding_agent.py](/entities/coding-agent-py.md) (SYSTEM)
- [os.path.join](/entities/os-path-join.md) (TOOL)
- [diagnose_errors](/entities/diagnose-errors.md) (TOOL)
- [python_syntax_error](/entities/python-syntax-error.md) (CONCEPT)
- [attempt_error_resolution](/entities/attempt-error-resolution.md) (CONCEPT)
- [utils.git_utils](/entities/utils-git-utils.md) (SYSTEM)
- [tools.edit](/entities/tools-edit.md) (SYSTEM)
- [ai-research-part3](/entities/ai-research-part3.md) (PROJECT)
- [cloud KG extraction](/entities/cloud-kg-extraction.md) (CONCEPT)
- [tools.bash](/entities/tools-bash.md) (SYSTEM)
- [apply_automated_fix](/entities/apply-automated-fix.md) (TOOL)

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
