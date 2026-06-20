---
type: source
title: obolus-vs-codium-extension-konzept-research-part1-micro06
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# obolus-vs-codium-extension-konzept-research-part1-micro06

Ingested source summary (2026-06-09).

## Entities
- [vscode.window.withProgress API](/entities/vscode-window-withprogress-api.md) (TOOL)
- [WebSocket-Kanal](/entities/websocket-kanal.md) (SYSTEM)
- [AOS: Open Benchmark Wizard](/entities/aos-open-benchmark-wizard.md) (TOOL)
- [src/panels/BenchmarkWizardPanel.ts](/entities/src-panels-benchmarkwizardpanel-ts.md) (SYSTEM)
- [Intelligence Dashboard (Sidebar)](/entities/intelligence-dashboard-sidebar.md) (CONCEPT)
- [Activity Bar](/entities/activity-bar.md) (CONCEPT)
- [AOS Telemetry Engine](/entities/aos-telemetry-engine.md) (CONCEPT)
- [Blueprint für Phase 4](/entities/blueprint-f-r-phase-4.md) (CONCEPT)
- [DOM manipulation](/entities/dom-manipulation.md) (CONCEPT)
- [FastAPI](/entities/fastapi.md) (SYSTEM)
- [Architektur-Blueprint](/entities/architektur-blueprint.md) (CONCEPT)
- [AbortController](/entities/abortcontroller.md) (TOOL)
- [Energy Timeline](/entities/energy-timeline.md) (CONCEPT)
- [AOS: Open Leaderboard](/entities/aos-open-leaderboard.md) (CONCEPT)
- [webview.asWebviewUri](/entities/webview-aswebviewuri.md) (TOOL)
- [VSCodium](/entities/vscodium.md) (SYSTEM)
- [AOS Intelligence Dashboard](/entities/aos-intelligence-dashboard.md) (PROJECT)
- [src/extension.ts](/entities/src-extension-ts.md) (SYSTEM)
- [SidebarProvider](/entities/sidebarprovider.md) (SYSTEM)
- [Extension Host](/entities/extension-host.md) (SYSTEM)
- [package.json](/entities/package-json.md) (SYSTEM)
- [obolus-vs-codium-extension-konzept-research-part1](/entities/obolus-vs-codium-extension-konzept-research-part1.md) (PROJECT)
- [HTTP-Strategie: Zero-Dependency-Ansatz](/entities/http-strategie-zero-dependency-ansatz.md) (CONCEPT)
- [BenchmarkRequest](/entities/benchmarkrequest.md) (CONCEPT)
- [Native UX: Doppeltes Feedback (withProgress)](/entities/native-ux-doppeltes-feedback-withprogress.md) (CONCEPT)
- [Obolus-runner](/entities/obolus-runner.md) (TOOL)
- [Content Security Policy (CSP)](/entities/content-security-policy-csp.md) (CONCEPT)

## Relations
- AOS: Open Benchmark Wizard → RELATED_TO → FastAPI
- AOS: Open Benchmark Wizard → USES → AbortController
- AOS: Open Benchmark Wizard → RELATED_TO → Extension Host
- AOS: Open Benchmark Wizard → PART_OF → src/panels/BenchmarkWizardPanel.ts
- FastAPI → USES → Obolus-runner
- vscode.window.withProgress API → USES → VSCodium
- vscode.window.withProgress API → USES → AOS: Open Benchmark Wizard
- Architektur-Blueprint → RELATED_TO → AOS: Open Benchmark Wizard
- Extension Host → RELATED_TO → AOS: Open Benchmark Wizard
- Extension Host → RELATED_TO → SidebarProvider
- src/panels/BenchmarkWizardPanel.ts → RELATED_TO → Extension Host
- BenchmarkRequest → RELATED_TO → AOS: Open Benchmark Wizard
- AbortController → USES → AOS: Open Benchmark Wizard
- vscode.window.withProgress API → USES → SidebarProvider
- vscode.window.withProgress API → USES → package.json
- Intelligence Dashboard (Sidebar) → RELATED_TO → VSCodium
- package.json → PART_OF → AOS Telemetry Engine
- AOS Telemetry Engine → RELATED_TO → AOS Intelligence Dashboard
- AOS Intelligence Dashboard → PART_OF → Activity Bar
- AOS Intelligence Dashboard → RELATED_TO → Intelligence Dashboard (Sidebar)
- src/extension.ts → RELATED_TO → Extension Host
- src/extension.ts → USES → vscode.window.withProgress API
- src/extension.ts → RELATED_TO → SidebarProvider
- SidebarProvider → RELATED_TO → Extension Host
- SidebarProvider → USES → vscode.window.withProgress API
- SidebarProvider → USES → Content Security Policy (CSP)
- SidebarProvider → USES → webview.asWebviewUri
- Content Security Policy (CSP) → RELATED_TO → SidebarProvider
- webview.asWebviewUri → USES → SidebarProvider
