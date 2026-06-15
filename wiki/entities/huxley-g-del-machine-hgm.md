---
type: entity
title: Huxley–Gödel Machine (HGM)
created: 2026-06-09
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# Huxley–Gödel Machine (HGM)

Type: SYSTEM

## From [[ai-research-part3-micro03|ai-research-part3-micro03]] (2026-06-09)
- demonstrates significant gains and strong absolute performance
- discovered an optimized agent that solves 61.4% tasks
- positions it among the top-10 agents over all checked submissions
- demonstrates a promising potential for competing with established human-designed baselines under identical model constraints
- self-evolution produces agents with stronger general coding ability
- outperforms SWE-agent + GPT-5-mini
- consistently produces higher quality agents than prior self-improving frameworks
- reduces wall-clock time
- generalizes across both dataset and model shifts
- achieving human-level coding agent design performance on SWE-bench Lite with GPT-5 despite being optimized on SWE-bench Verified with GPT-5-mini
- approximates CMP
- uses CMP to guide expansion through Thompson sampling with adaptive scheduling
- consistently produces higher quality agents than prior self-improving frameworks while also reducing wall-clock time

## From [[ai-research-part3-micro05|ai-research-part3-micro05]] (2026-06-09)
- Algorithm 1 presents its procedure.
- Initializes by expanding the initial agent 5 times with each of the processes in parallel.
- Asynchronization can introduce bias favoring agents with fewer evaluated results.
- Selection Policy is an adaptive choice between modification and evaluation.
- Expansion Policy selects the node based on the statistics of the clade stemming from a given node.
- Evaluation Policy selects the agent based on the statistics and evaluates it on a single task.
- Has a prover with full knowledge of the utility function.
- Proofs do not consume the budget.
- Each self-modification costs one unit of the budget.
- Can be simulated with a CMPπ oracle.
- Is defined by a prover that produces a proof whether accepting or selecting a given node (or rejecting).

## From [[ai-research-part8-micro03|ai-research-part8-micro03]] (2026-06-09)
- Addresses bottlenecks in autonomous self-improvement by analyzing expansion strategies.
- Identifies a flaw in favoring agents with the highest immediate scores.

## From [[ai-research-part8-micro07|ai-research-part8-micro07]] (2026-06-09)
- It replaces immediate benchmark scoring with Clade-Level Metaproductivity (CMP).
- It prioritizes agents whose code structures lead to superior descendants.
- It is associated with clade-based estimation.
- It is a leading AI framework for Evolutionary Agents.
- It is related to the Darwin Gödel Machine (DGM).
- It is associated with Clade-Level Metaproductivity (CMP).
- It is related to the Huxley-Gödel Machine.
