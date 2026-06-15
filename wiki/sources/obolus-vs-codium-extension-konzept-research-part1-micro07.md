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
- [[gzmo|GZMO]] (CONCEPT)
- [[state-management-via-getstate-setstate|State Management via getState/setState]] (CONCEPT)
- [[vscode-webview-ui-toolkit|vscode-webview-ui-toolkit]] (TOOL)
- [[vscode-extensioncontext|vscode.ExtensionContext]] (SYSTEM)
- [[sidebar|Sidebar]] (SYSTEM)
- [[fastapi|FastAPI]] (SYSTEM)
- [[benchmark-wizard|Benchmark Wizard]] (SYSTEM)
- [[package-json|package.json]] (TOOL)
- [[vscode-webviewpanel|vscode.WebviewPanel]] (SYSTEM)
- [[src-webview-wizard-ts|src/webview/wizard.ts]] (SYSTEM)
- [[benchmarkwizardpanel|BenchmarkWizardPanel]] (SYSTEM)
- [[vscode-uri|vscode.Uri]] (SYSTEM)
- [[obolus|Obolus]] (CONCEPT)
- [[src-extension-ts|src/extension.ts]] (SYSTEM)

## Relations
- Benchmark Wizard → USES → State Management via getState/setState
- Benchmark Wizard → RELATED_TO → Sidebar
- BenchmarkWizardPanel → USES → vscode.ExtensionContext
- BenchmarkWizardPanel → USES → vscode.WebviewPanel
- BenchmarkWizardPanel → USES → FastAPI
- src/webview/wizard.ts → USES → State Management via getState/setState
- src/webview/wizard.ts → USES → vscode.Uri
- Obolus → RELATED_TO → GZMO
