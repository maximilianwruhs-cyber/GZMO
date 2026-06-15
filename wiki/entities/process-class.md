---
type: entity
title: Process Class
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Process Class

Type: CONCEPT

## From [[drive-research-architecting-a-linux-task-manager-design-principl|drive-research-architecting-a-linux-task-manager-design-principl]] (2026-06-08)
- A modular component in C/C++ task manager architectures.
- Holds process data.
- Overloads the less-than operator (operator<) based on CPU utilization for rapid sorting (std::sort).

## From [[gzmo-soul-merged-new-part1|gzmo-soul-merged-new-part1]] (2026-06-09)
- Class instantiated for every active PID in a C++ task manager architecture
