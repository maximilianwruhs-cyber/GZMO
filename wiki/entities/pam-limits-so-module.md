---
type: entity
title: pam_limits.so module
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# pam_limits.so module

Type: TOOL

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- The pam_limits.so module evaluates and applies resource boundaries during the user authentication phase.
- Modifying configuration files like limits.conf will have no effect on current terminal sessions or active background processes until a fresh login occurs, as these limits are applied by this module.
- Standard PAM settings defined in /etc/security/limits.conf do not apply when a service is managed by systemd.
- Configuration directives must be defined in the PAM limits configuration subsystem to apply resource limits persistently.
- Resource boundaries are evaluated and applied by the pam_limits.so module during the user authentication phase.
