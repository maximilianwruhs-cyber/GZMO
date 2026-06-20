---
type: entity
title: "Agent-Reach Patterns"
created: "2026-06-20"
updated: "2026-06-20"
status: draft
sources: 1
tags:
  - research
  - agent-reach
  - compliance
---

# Agent-Reach Patterns

This document details the compliance and architectural design patterns implemented to integrate Agent-Reach concepts within GZMO under the strict Sovereign Node Directive (SND).

## Security Compliance & Sovereign Node Directive (SND)

Under the Sovereign Node Directive, GZMO nodes must operate in an offline, air-gapped, and telemetriefreie (telemetry-free) environment. Therefore:
- Active web-scraping, remote session hijacking, and residential proxy networks are **explicitly blocked** by default.
- No remote exfiltration of credentials or cookies is permitted.

To research and implement features of Agent-Reach safely, GZMO utilizes **Sandbox Fallback Routing** and the **Operator Confirm Gate**.

---

## 1. Sandbox Fallback Routing

When an agent or script requests an active Agent-Reach capability (such as scraping an external resource), the request is intercepted and routed to a local hermetic fallback environment.

### Design Pattern
1. **Capability Interception:** The router detects tools tagged with `requires-internet` or `agent-reach`.
2. **Local Mock Registry:** Instead of making an outbound HTTP/TLS connection, the router queries a local JSON database or mock directory matching the requested signature.
3. **CLI Fallback Execution:** For interactive tasks, a local headless sandbox CLI (running inside a restricted cgroup/namespace under `~/.agent-reach/sandbox/`) mimics the target interface.

```
[Agent Action]
      │
      ▼
[GZMO Outbound Router]
      │
      ├─► [Remote URL Requested?] ──► BLOCKED (SND Compliance)
      │
      └─► [Fallback Route] ──► [Restricted Local Sandbox] ──► [Local JSON Mock]
```

---

## 2. Operator Confirm Gate (OCG)

The Operator Confirm Gate is a synchronous, blocking approval mechanism designed to prevent accidental execution of unverified sandbox escaping or network execution.

### OCG Schema and Flow
Every command executing a fallback script containing sandbox commands must pass through an interactive validation check.

```rust
pub struct OperatorConfirmGate {
    pub action_id: String,
    pub command: String,
    pub risk_tier: String, // "High" or "Critical"
}

impl OperatorConfirmGate {
    pub fn verify_operator_consent(&self) -> bool {
        println!("WARNING: Execution of High-Risk Agent-Reach Command Requested!");
        println!("Command: {}", self.command);
        println!("Do you approve this execution? [y/N]");
        // Block and wait for operator interactive input
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).is_ok() && input.trim().to_lowercase() == "y"
    }
}
```

### Allowlist/Blocklist Verification
Any URL or command payload is verified against a strict hash-based allowlist:
- **Blocklist:** All external IP ranges, commercial proxy endpoints, and remote credential vaults.
- **Allowlist:** Local loops (`127.0.0.1`, `localhost`), local MCP server commands, and pre-approved mock testing suites.
