---
type: entity
title: SAP CRM
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# SAP CRM

Type: SYSTEM

## From [[drive-research-hermes-system-untersuchung-und-erweiterung|drive-research-hermes-system-untersuchung-und-erweiterung]] (2026-06-08)
- Has architectural separations between Business Object Layer (BOL) and Generic Interaction Layer (GenIL).
- A rollback on the GenIL layer does not necessarily affect the BOL.
- Requires calling the REVERT method to clean up modifications on the BOL.
