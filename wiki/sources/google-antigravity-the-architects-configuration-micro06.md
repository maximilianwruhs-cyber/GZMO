---
type: source
title: google-antigravity-the-architects-configuration-micro06
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# google-antigravity-the-architects-configuration-micro06

Ingested source summary (2026-06-09).

## Entities
- [jscodeshift](/entities/jscodeshift.md) (TOOL)
- [Canary-Releases](/entities/canary-releases.md) (CONCEPT)
- [Technische Gruppierung](/entities/technische-gruppierung.md) (CONCEPT)
- [Single Responsibility Principle](/entities/single-responsibility-principle.md) (CONCEPT)
- [Bounded Contexts](/entities/bounded-contexts.md) (CONCEPT)
- [CI/CD-Pipeline](/entities/ci-cd-pipeline.md) (SYSTEM)
- [Feature-Sliced Design](/entities/feature-sliced-design.md) (CONCEPT)
- [Tightly Coupled & Circular Dependencies](/entities/tightly-coupled-circular-dependencies.md) (CONCEPT)
- [Pfad-Aliase](/entities/pfad-aliase.md) (CONCEPT)
- [Microservices](/entities/microservices.md) (CONCEPT)
- [Technische Schulden](/entities/technische-schulden.md) (CONCEPT)
- [Codemods](/entities/codemods.md) (TOOL)
- [Software-Architekt](/entities/software-architekt.md) (PERSON)
- ["Junk Drawer"-Ordner](/entities/junk-drawer-ordner.md) (CONCEPT)
- [Dependency Inversion Principle](/entities/dependency-inversion-principle.md) (CONCEPT)
- [Interfaces](/entities/interfaces.md) (CONCEPT)
- [Characterization/Approval Tests](/entities/characterization-approval-tests.md) (CONCEPT)
- [Facade-Pattern](/entities/facade-pattern.md) (CONCEPT)
- [Abstract Syntax Tree (AST)](/entities/abstract-syntax-tree-ast.md) (CONCEPT)
- [Big Ball of Mud](/entities/big-ball-of-mud.md) (CONCEPT)
- [Strangler-Fig-Pattern](/entities/strangler-fig-pattern.md) (CONCEPT)
- [Zu tiefe Verschachtelung](/entities/zu-tiefe-verschachtelung.md) (CONCEPT)
- [Monolithen](/entities/monolithen.md) (CONCEPT)
- [Branch by Abstraction](/entities/branch-by-abstraction.md) (CONCEPT)
- [Git](/entities/git.md) (TOOL)
- [ort-merge](/entities/ort-merge.md) (CONCEPT)
- [Feature-Isolation](/entities/feature-isolation.md) (CONCEPT)
- [Performance-Metriken](/entities/performance-metriken.md) (CONCEPT)
- [Domain-Driven Design (DDD)](/entities/domain-driven-design-ddd.md) (CONCEPT)
- [Monorepo-Struktur](/entities/monorepo-struktur.md) (CONCEPT)
- [Legacy-Codebases](/entities/legacy-codebases.md) (CONCEPT)

## Relations
- Software-Architekt → USES → Legacy-Codebases
- Software-Architekt → USES → Technische Schulden
- Software-Architekt → USES → Monolithen
- Software-Architekt → USES → Domain-Driven Design (DDD)
- Software-Architekt → USES → Feature-Sliced Design
- Software-Architekt → USES → Monorepo-Struktur
- Software-Architekt → USES → Big Ball of Mud
- Software-Architekt → USES → Tightly Coupled & Circular Dependencies
- Software-Architekt → USES → Zu tiefe Verschachtelung
- Software-Architekt → USES → Technische Gruppierung
- Software-Architekt → USES → "Junk Drawer"-Ordner
- Software-Architekt → USES → Dependency Inversion Principle
- Software-Architekt → USES → Interfaces
- Software-Architekt → USES → Facade-Pattern
- Software-Architekt → USES → Branch by Abstraction
- Software-Architekt → USES → Strangler-Fig-Pattern
- Software-Architekt → USES → Codemods
- Software-Architekt → USES → Git
- Software-Architekt → USES → CI/CD-Pipeline
- Software-Architekt → USES → Single Responsibility Principle
- Software-Architekt → USES → Characterization/Approval Tests
- Software-Architekt → USES → Performance-Metriken
- Software-Architekt → USES → Microservices
- Legacy-Codebases → RELATED_TO → Big Ball of Mud
- Legacy-Codebases → RELATED_TO → Tightly Coupled & Circular Dependencies
- Legacy-Codebases → RELATED_TO → Zu tiefe Verschachtelung
- Legacy-Codebases → RELATED_TO → Technische Gruppierung
- Legacy-Codebases → RELATED_TO → "Junk Drawer"-Ordner
- Domain-Driven Design (DDD) → PART_OF → Bounded Contexts
- Monorepo-Struktur → PART_OF → Bounded Contexts
- Monorepo-Struktur → PART_OF → Feature-Isolation
- Monorepo-Struktur → PART_OF → Pfad-Aliase
- Feature-Isolation → RELATED_TO → Feature-Sliced Design
- Dependency Inversion Principle → RELATED_TO → Interfaces
- Dependency Inversion Principle → RELATED_TO → Facade-Pattern
- Branch by Abstraction → RELATED_TO → Canary-Releases
- Strangler-Fig-Pattern → RELATED_TO → Canary-Releases
- Codemods → USES → Abstract Syntax Tree (AST)
- Codemods → RELATED_TO → jscodeshift
- Git → USES → ort-merge
- Git → USES → CI/CD-Pipeline
- CI/CD-Pipeline → USES → Git
- Single Responsibility Principle → RELATED_TO → "Junk Drawer"-Ordner
- Characterization/Approval Tests → USES → Legacy-Codebases
