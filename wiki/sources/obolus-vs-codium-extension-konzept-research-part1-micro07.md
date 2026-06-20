---
type: source
title: obolus-vs-codium-extension-konzept-research-part1-micro07
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# obolus-vs-codium-extension-konzept-research-part1-micro07

Ingested source summary (2026-06-09).

## Entities
- [GZMO](/entities/gzmo.md) (CONCEPT)
- [State Management via getState/setState](/entities/state-management-via-getstate-setstate.md) (CONCEPT)
- [vscode-webview-ui-toolkit](/entities/vscode-webview-ui-toolkit.md) (TOOL)
- [vscode.ExtensionContext](/entities/vscode-extensioncontext.md) (SYSTEM)
- [Sidebar](/entities/sidebar.md) (SYSTEM)
- [FastAPI](/entities/fastapi.md) (SYSTEM)
- [Benchmark Wizard](/entities/benchmark-wizard.md) (SYSTEM)
- [package.json](/entities/package-json.md) (TOOL)
- [vscode.WebviewPanel](/entities/vscode-webviewpanel.md) (SYSTEM)
- [src/webview/wizard.ts](/entities/src-webview-wizard-ts.md) (SYSTEM)
- [BenchmarkWizardPanel](/entities/benchmarkwizardpanel.md) (SYSTEM)
- [vscode.Uri](/entities/vscode-uri.md) (SYSTEM)
- [Obolus](/entities/obolus.md) (CONCEPT)
- [src/extension.ts](/entities/src-extension-ts.md) (SYSTEM)

## Relations
- Benchmark Wizard → USES → State Management via getState/setState
- Benchmark Wizard → RELATED_TO → Sidebar
- BenchmarkWizardPanel → USES → vscode.ExtensionContext
- BenchmarkWizardPanel → USES → vscode.WebviewPanel
- BenchmarkWizardPanel → USES → FastAPI
- src/webview/wizard.ts → USES → State Management via getState/setState
- src/webview/wizard.ts → USES → vscode.Uri
- Obolus → RELATED_TO → GZMO
