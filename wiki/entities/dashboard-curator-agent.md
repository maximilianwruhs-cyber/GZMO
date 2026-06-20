---
type: entity
title: Dashboard Curator Agent
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Dashboard Curator Agent

Type: AGENT

## From [obolus-micro04](/entities/obolus-micro04.md) (2026-06-09)
- visual guardian of the ServiceBot system
- ensures the status report (dashboard.html) remains up-to-date, technically precise, and visually appealing
- validates data provided by update_live_dashboard.py
- suggests new metrics when new sub-agents or services are added
- improves the CSS/layout of the dashboard page
- monitors the cron job that updates the dashboard and reports errors
- works closely with the Ops & Monitoring Agent
- uses the update_live_dashboard.py script as its primary tool
- documents design changes in ServiceBot/DASHBOARD_CHANGELOG.md
