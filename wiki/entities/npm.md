---
type: entity
title: NPM
created: 2026-06-08
updated: 2026-06-10
sources: 14
tags: []
status: draft
gzmo_synthetic: true
---














# NPM

Type: TOOL

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- Many packages include native bindings precompiled against glibc.

## From [[architectural-framework-and-development-of-pi-codi|architectural-framework-and-development-of-pi-codi]] (2026-06-08)
- Pi's decentralized package model integrates with the existing npm ecosystem.
- Used to install packages globally via `pi install npm`.
- The agent automatically runs `npm install` within a package directory to resolve third-party dependencies.

## From [[openclaw-deep-research-part6|openclaw-deep-research-part6]] (2026-06-08)
- Used to install the official OpenClaw package.

## From [[drive-research-developing-pi-coding-agent-ide-extensions|drive-research-developing-pi-coding-agent-ide-extensions]] (2026-06-08)
- The ecosystem used for distributing Pi Packages.
- Used to resolve third-party dependencies when installing packages.

## From [[drive-research-pi-coding-agent-local-deployment-customization|drive-research-pi-coding-agent-local-deployment-customization]] (2026-06-08)
- A global package manager that can be used for installation.
- External npm dependencies are pinned to exact versions.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Bun's performance advantage extends deeply into package management.
- When bun install is executed, it avoids the systemic inefficiency of Node's npm by reducing operating system syscalls from nearly one million down to approximately 165,000.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Has systemic inefficiency in package management compared to bun install.

## From [[high-performance-typescript-execution-and-architec-part1-micro05|high-performance-typescript-execution-and-architec-part1-micro05]] (2026-06-09)
- Extensions can leverage standard npm dependencies.
- The Pi package installation process executes 'npm install --omit=dev' by default.
- Dependencies must be declared in the package.json file.

## From [[high-performance-typescript-execution-and-architec-part1-micro07|high-performance-typescript-execution-and-architec-part1-micro07]] (2026-06-09)
- Used for installing dependencies.
- Can be used to distribute Pi Packages.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro08|obolus-vs-codium-extension-konzept-research-part1-micro08]] (2026-06-09)
- Used to install the Webview UI Toolkit with the command 'npm install --save @vscode/webview-ui-toolkit'.

## From [[openclaw-deep-research-part10-micro06|openclaw-deep-research-part10-micro06]] (2026-06-09)
- Used for installing dependencies, including 'npm install --omit=dev' in plugin directories.
- Global install command is 'sudo npm i -g openclaw@latest'.

## From [[prompt-agent-engineering-part2-micro04|prompt-agent-engineering-part2-micro04]] (2026-06-09)
- Associated with Node.js.
- Contributes to massive overhead.

## From [[openclaw-deep-research-part9-micro03|openclaw-deep-research-part9-micro03]] (2026-06-10)
- Node package manager used for installation

## From [[openclaw-deep-research-part9-micro04|openclaw-deep-research-part9-micro04]] (2026-06-10)
- A node package manager used by OpenClaw for installing skills
