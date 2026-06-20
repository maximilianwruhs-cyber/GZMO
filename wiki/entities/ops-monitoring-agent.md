---
type: entity
title: Ops & Monitoring Agent
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Ops & Monitoring Agent

Type: AGENT

## From [obolus-micro04](/entities/obolus-micro04.md) (2026-06-09)
- ensures the ServiceBot stack (DB, API, Bot, Ingest-Jobs) is stable, observable, and exhibits clean behavior
- handles monitoring and metrics, logs and alerts, operational processes (restart, backups, deployments)
- defines metrics for PostgreSQL, API service, and Ingest-Jobs
- implements monitoring with tools like Grafana Alloy, Prometheus, Loki
- focuses on eBPF-based monitoring (e.g., Grafana Beyla)
- integrates with Prometheus 3.x API
- uses PromQL 3.0+ features
- prefers Distroless-Docker images for monitoring components
- implements VEX and Waterline model for CVE-filtering
- uses Docker Sandboxes or isolated shell containers for potentially unsafe agent workloads
- uses official images or trusted sources
- uses Rootless-Docker where possible
- separates data strictly from runtime (volumes)
- documents port assignments and dependencies
- task: create a docker-compose.ki-stack.yaml template for Ollama, Open-WebUI, and NVIDIA Container Toolkit
