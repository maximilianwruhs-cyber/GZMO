---
type: entity
title: Captive Portal Detection
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Captive Portal Detection

Type: CONCEPT

## From [[architectural-strategy-for-stealthy-portable-cli-a|architectural-strategy-for-stealthy-portable-cli-a]] (2026-06-08)
- Endpoints used by modern operating systems to determine if behind a captive network portal.
- Exploited by the agent to verify active internet connectivity.
- Endpoints like http://captive.apple.com/hotspot-detect.html and http://www.msftconnecttest.com/connecttest.txt are globally whitelisted.

## From [[drive-research-architecting-zero-configuration-portable-agents-s-micro03|drive-research-architecting-zero-configuration-portable-agents-s-micro03]] (2026-06-09)
- A strategy to verify internet connectivity without resembling malicious telemetry.
- Exploits endpoints that modern operating systems ping to determine if the host is behind a captive network portal.
