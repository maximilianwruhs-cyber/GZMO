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
- [[vscode-window-withprogress-api|vscode.window.withProgress API]] (TOOL)
- [[websocket-kanal|WebSocket-Kanal]] (SYSTEM)
- [[aos-open-benchmark-wizard|AOS: Open Benchmark Wizard]] (TOOL)
- [[src-panels-benchmarkwizardpanel-ts|src/panels/BenchmarkWizardPanel.ts]] (SYSTEM)
- [[intelligence-dashboard-sidebar|Intelligence Dashboard (Sidebar)]] (CONCEPT)
- [[activity-bar|Activity Bar]] (CONCEPT)
- [[aos-telemetry-engine|AOS Telemetry Engine]] (CONCEPT)
- [[blueprint-f-r-phase-4|Blueprint für Phase 4]] (CONCEPT)
- [[dom-manipulation|DOM manipulation]] (CONCEPT)
- [[fastapi|FastAPI]] (SYSTEM)
- [[architektur-blueprint|Architektur-Blueprint]] (CONCEPT)
- [[abortcontroller|AbortController]] (TOOL)
- [[energy-timeline|Energy Timeline]] (CONCEPT)
- [[aos-open-leaderboard|AOS: Open Leaderboard]] (CONCEPT)
- [[webview-aswebviewuri|webview.asWebviewUri]] (TOOL)
- [[vscodium|VSCodium]] (SYSTEM)
- [[aos-intelligence-dashboard|AOS Intelligence Dashboard]] (PROJECT)
- [[src-extension-ts|src/extension.ts]] (SYSTEM)
- [[sidebarprovider|SidebarProvider]] (SYSTEM)
- [[extension-host|Extension Host]] (SYSTEM)
- [[package-json|package.json]] (SYSTEM)
- [[obolus-vs-codium-extension-konzept-research-part1|obolus-vs-codium-extension-konzept-research-part1]] (PROJECT)
- [[http-strategie-zero-dependency-ansatz|HTTP-Strategie: Zero-Dependency-Ansatz]] (CONCEPT)
- [[benchmarkrequest|BenchmarkRequest]] (CONCEPT)
- [[native-ux-doppeltes-feedback-withprogress|Native UX: Doppeltes Feedback (withProgress)]] (CONCEPT)
- [[obolus-runner|Obolus-runner]] (TOOL)
- [[content-security-policy-csp|Content Security Policy (CSP)]] (CONCEPT)

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
