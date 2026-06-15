---
type: entity
title: limits.conf
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# limits.conf

Type: TOOL

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- Standard PAM settings defined in /etc/security/limits.conf do not apply when a service is managed by systemd.
- Configuration directives must be defined in the PAM limits configuration subsystem to apply resource limits persistently.
- Editing /etc/security/limits.conf is standard, but deploying a dedicated configuration file inside /etc/security/limits.d/ is preferred.
- Configuration directives in limits.conf are evaluated and applied by the pam_limits.so module during the user authentication phase.
