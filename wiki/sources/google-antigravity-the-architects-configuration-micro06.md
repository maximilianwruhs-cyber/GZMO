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
- [[jscodeshift|jscodeshift]] (TOOL)
- [[canary-releases|Canary-Releases]] (CONCEPT)
- [[technische-gruppierung|Technische Gruppierung]] (CONCEPT)
- [[single-responsibility-principle|Single Responsibility Principle]] (CONCEPT)
- [[bounded-contexts|Bounded Contexts]] (CONCEPT)
- [[ci-cd-pipeline|CI/CD-Pipeline]] (SYSTEM)
- [[feature-sliced-design|Feature-Sliced Design]] (CONCEPT)
- [[tightly-coupled-circular-dependencies|Tightly Coupled & Circular Dependencies]] (CONCEPT)
- [[pfad-aliase|Pfad-Aliase]] (CONCEPT)
- [[microservices|Microservices]] (CONCEPT)
- [[technische-schulden|Technische Schulden]] (CONCEPT)
- [[codemods|Codemods]] (TOOL)
- [[software-architekt|Software-Architekt]] (PERSON)
- [[junk-drawer-ordner|"Junk Drawer"-Ordner]] (CONCEPT)
- [[dependency-inversion-principle|Dependency Inversion Principle]] (CONCEPT)
- [[interfaces|Interfaces]] (CONCEPT)
- [[characterization-approval-tests|Characterization/Approval Tests]] (CONCEPT)
- [[facade-pattern|Facade-Pattern]] (CONCEPT)
- [[abstract-syntax-tree-ast|Abstract Syntax Tree (AST)]] (CONCEPT)
- [[big-ball-of-mud|Big Ball of Mud]] (CONCEPT)
- [[strangler-fig-pattern|Strangler-Fig-Pattern]] (CONCEPT)
- [[zu-tiefe-verschachtelung|Zu tiefe Verschachtelung]] (CONCEPT)
- [[monolithen|Monolithen]] (CONCEPT)
- [[branch-by-abstraction|Branch by Abstraction]] (CONCEPT)
- [[git|Git]] (TOOL)
- [[ort-merge|ort-merge]] (CONCEPT)
- [[feature-isolation|Feature-Isolation]] (CONCEPT)
- [[performance-metriken|Performance-Metriken]] (CONCEPT)
- [[domain-driven-design-ddd|Domain-Driven Design (DDD)]] (CONCEPT)
- [[monorepo-struktur|Monorepo-Struktur]] (CONCEPT)
- [[legacy-codebases|Legacy-Codebases]] (CONCEPT)

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
