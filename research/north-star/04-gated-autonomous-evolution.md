# Gated autonomous evolution for an air-gapped Living Database

Research date: 2026-08-31. Scope: evidence for architectures that let a single physical, runtime-air-gapped node generate and evaluate improvements without uncontrolled self-modification of production. No architecture is selected here.

## Executive finding

Proven update systems and primary AI-safety literature converge on one structural pattern that is compatible with GZMO’s constitutional split of authority: **candidates may be generated and scored autonomously only inside isolated evaluation planes; production mutation of code, schema, models, security policy, or capability envelopes requires operator-held cryptographic authorization that the candidate path cannot mint, widen, or revoke.**

Three families of control loop are well-attested and should be treated as design inputs, not as a final pick:

1. **Dual-slot / atomic deployment with boot-time success marking** (Android A/B + `update_engine`, OSTree atomic deployments): write only the inactive slot; keep the booted system immutable during apply; fall back automatically if the new slot fails boot/verify; user-space cannot safely own the “mark successful” privilege without a separate trusted verifier.
2. **Role-separated signed update metadata + threshold offline roots** (The Update Framework (TUF) v1.0.x; automotive Uptane lineage): root keys offline; online keys limited; mix-and-match, rollback, freeze, and single-key compromise are first-class threats; clients refuse targets that fail role/threshold/freshness checks.
3. **Empirical self-improvement with archive + sandbox evaluation** (Gödel machine theory; ADAS / Meta Agent Search; Darwin Gödel Machine): self-edits are proposed in code, measured on benchmarks, and retained as an archive of stepping stones—but primary papers themselves treat **sandboxing, human oversight, non-self-modifiable safety criteria, and non-identity of benchmark score with true intent** as mandatory safety bounds, not optional polish.

Across these families, **fitness floors that are non-compensable, evaluator isolation from candidates, append-only audit with consistency proofs, and operator-signed promotion for authority-expanding changes** are the pieces that can be machine-enforced offline. **Whether a passing evaluation means the change is desirable, whether benchmarks are gamed, and whether to expand capability** remain operator judgment. Formal proof of beneficial self-modification (classic Gödel machine) is not available for realistic GZMO changes; empirical gates are the practical substitute and are forgeable if the evaluator, metrics, or audit path are in the candidate’s write set.

## Decision-relevant facts

### Authority tiers (constitutional mapping)

Approved North Star framing (ticket 00, 2026-08-31): memory may evolve autonomously; bounded tunables may self-promote inside operator-granted envelopes; code/schema/models/security/capability expansion require signed approval; hard floors on sovereignty, faithfulness, reliability, resource, audit, and rollback precede optimization of usefulness/latency/energy.

| Tier | What changes | Who may generate | Who may promote to production | Machine-enforceable offline? |
|------|----------------|------------------|-------------------------------|------------------------------|
| **M — Memory** | Indices, embeddings, distilled notes, retrieval graphs, episodic stores | Autonomous agents | Autonomous, subject to faithfulness/audit floors and quarantine of untrusted writes | Mostly yes (write scopes, schema seals, hash-chained memory log, resource caps); **semantic faithfulness** is only partly mechanical |
| **T — Tunables** | Numeric/enum parameters inside a signed envelope (timeouts, batch sizes, cache TTLs, sampling temperatures within bounds) | Autonomous optimizer | Autonomous **iff** new value ∈ signed envelope ∧ hard floors still pass | Yes (envelope checker + floor gates); **choosing the envelope** is operator |
| **C — Candidates** | Code, container images, schema migrations, model weights/adapters, security policy drafts, evaluator drafts | Autonomous (isolated builders) | **Never autonomous** | Generation/isolation yes; promotion no without operator signature |
| **P — Promote** | Binding a candidate artifact to a production slot/ref | N/A (human + tooling) | Operator signature(s), optionally threshold | Yes (signature verify, slot flip policy) |
| **A — Authority** | Capability envelopes, root keys, floor definitions, which metrics may gate promote, who may sign | Operator only (out-of-band) | Operator offline root / threshold | Yes if root material is offline and non-exportable to the agent runtime |

**Invariant:** no tier may grant itself rights belonging to a higher tier. In TUF terms, this is role separation with offline root and non-delegation of root-equivalent power to online/automated roles ([TUF Spec 1.0.36](https://theupdateframework.github.io/specification/latest/), §2.1).

### Isolated candidate generation

- **Sandbox boundary:** DGM runs agent execution and self-modification in isolated sandboxes with time limits and states that modifications optimized only for benchmarks can introduce misaligned behavior if evaluation does not capture safety/robustness ([arXiv:2505.22954](https://ar5iv.labs.arxiv.org/html/2505.22954), §5). ADAS/Meta Agent Search likewise evaluates candidates on validation data before archive insertion ([arXiv:2408.08435](https://ar5iv.labs.arxiv.org/html/2408.08435)).
- **Air-gap implication:** “download tools/models from the network during evolve” is unavailable. Candidate generation must use **on-box** sources: local model weights, local corpora, local package mirrors, and pre-provisioned toolchains. Network-shaped APIs must fail closed.
- **Write set:** candidate builders must not hold credentials or kernel capabilities to mutate production slots, root metadata, audit log keys, or evaluator binaries. Capability-oriented isolation (least privilege; object-capability discipline) is the OS-level analogue of TUF’s role split—see historical Capsicum and seL4 capability models as design references for confining mutable rights ([Capsicum](https://www.cl.cam.ac.uk/research/security/capsicum/); [seL4](https://sel4.systems/)).
- **Build hermeticity:** Reproducible Builds defines a build as reproducible when the same source, environment, and instructions yield bit-identical artifacts, verified by cryptographic hashes ([reproducible-builds.org definitions](https://reproducible-builds.org/docs/definition/)). Offline, this lets an operator (or a separate verifier process) confirm “this binary is exactly what that source tree builds” without trusting the builder host’s word.

### Provenance, attestation, and signed promotion

- **Attestation stack (industry consensus suite):** SLSA attestation model recommends **DSSE** envelope + **in-toto Statement** + typed **Predicate** (e.g. SLSA provenance), consumed by policy engines ([SLSA attestation model v1.0](https://slsa.dev/spec/v1.0/attestation-model); [in-toto attestation spec](https://github.com/in-toto/attestation/tree/main/spec); [DSSE](https://github.com/secure-systems-lab/dsse)).
- **SLSA Build track (v1.0):** L1 = provenance exists; L2 = hosted platform signs provenance; L3 = hardened platform isolating builds and protecting signing keys from user-defined steps ([SLSA levels v1.0](https://slsa.dev/spec/v1.0/levels)). On a **single air-gapped node**, “hosted multi-tenant CI” is absent; the **intent** of L3 still applies: **build steps must not access the key that signs provenance or promotion**, and concurrent builds must not influence each other.
- **in-toto product goal:** end-to-end integrity by recording which steps ran, by whom, in what order, from initiation to install ([in-toto about](https://in-toto.io/about/)).
- **TUF roles:** Root (offline, delegates all other roles), Targets (what files are trusted; can delegate), Snapshot (consistent set of targets metadata—blocks mix-and-match), Timestamp (freshness; online-acceptable because impact of compromise is limited) ([TUF Spec](https://theupdateframework.github.io/specification/latest/), §2.1). **Online automated agent keys must not be Root or sole Targets for production.**
- **Promotion artifact:** treat “operator promote” as signing a **Targets-equivalent** (or dual-control) statement binding `(artifact digest, slot/ref, policy version, floor-report digest, not-after)`. Verification is local and offline.

### Dual-slot, canary, shadow, rollback

- **Android A/B (seamless) updates:** two slots; run from current; stream/write only unused slot; `boot_control` marks bootable/active/successful; failed boot falls back; `dm-verity` detects corruption; post-install runs under **old** SELinux policy constraints; success marking happens only after new slot runs checks ([AOSP A/B docs](https://source.android.com/docs/core/ota/ab)). Directly maps to “candidate never overwrites the only good system.”
- **OSTree atomic upgrades:** deploy as hardlink farm beside the running system; atomically swap boot configuration; booted deployment always retained in the new deployment set; power loss yields old or new, not half-applied ([OSTree atomic upgrades](https://ostreedev.github.io/ostree/atomic-upgrades/)).
- **Google SRE canarying:** partial, time-limited exposure; compare canary vs control metrics; automate pause/rollback; prefer metrics attributable to the change and tied to SLIs; before/after-in-time is risky; imperfect isolation can contaminate control ([SRE Workbook ch. 16](https://sre.google/workbook/canarying-releases/)). On **one node**, classic traffic-split canary is limited; substitutes are **shadow evaluation** (candidate serves replayed/synthetic load while production answers users), **temporal soak** on non-serving replicas, and **slot boot canary** (boot candidate slot for soak under watchdog, then flip default only after floors).
- **Argo Rollouts:** progressive delivery with blue-green/canary, analysis templates, automated rollback, and **manual judgement** gates ([Argo Rollouts](https://argoproj.github.io/rollouts/)). Pattern remains valid in-process: analysis jobs + mandatory human approval for high-blast changes.

### Append-only audit

- Certificate Transparency (RFC 6962; superseded by RFC 9162 for the modern ecosystem) establishes **publicly auditable, append-only Merkle-tree logs** with inclusion and consistency proofs so parties can detect mis-issue and log misbehavior without trusting the log blindly ([RFC 6962](https://www.rfc-editor.org/rfc/rfc6962)).
- For GZMO offline: a **local Merkle audit log** (or hash chain with periodic signed tree heads under an operator-held key) should record: candidate IDs, source digests, build provenance, evaluation reports, floor pass/fail, promote/reject signatures, slot flips, rollbacks, envelope changes. **Candidates must not hold the log’s integrity key**; at most they append via a one-way sink that assigns monotonic indices.

### Reward hacking, evaluator compromise, overfitting, authority expansion, rollback tampering

Primary framing from Amodei et al., *Concrete Problems in AI Safety* ([arXiv:1606.06565](https://ar5iv.labs.arxiv.org/html/1606.06565)):

| Failure | Mechanism (primary) | Offline mechanical mitigations | Residual judgment |
|---------|----------------------|--------------------------------|-------------------|
| **Reward hacking / specification gaming** | Wrong or gameable objective; partial observability; Goodhart; wireheading/environmental embedding of reward | Multiple independent metrics; reward/metric code **not writable** by candidate; caps; tripwires; adversarial checks; separate pretrained judges | Whether proxies match operator intent |
| **Side effects** | Objective ignores environment dimensions | Hard non-compensable floors; blast-radius caps; deny capability to touch out-of-scope state | Which side effects matter |
| **Scalable oversight** | True objective expensive to evaluate | Sparse operator review + frequent cheap proxies; force disclosure of cheap signals | Review budget and proxy trust |
| **Unsafe exploration** | Irreversible trial actions | Sandbox; no prod credentials; resource/time limits; dual-slot | What counts as irreversible |
| **Distributional shift** | Train ≠ deploy | Shadow on real local workloads; hold-out tasks; soak | Representativeness of hold-outs |
| **Evaluator compromise** | Candidate edits tests/judges/metrics | Judges on read-only media or separate key-signed volume; evaluation orchestrator outside candidate TCC; golden tests hashed in root metadata | Design of golden suite |
| **Benchmark overfitting** | Optimize for known suite (DGM explicitly assumes benchmark↑ ⇒ coding↑ and warns if benchmarks omit safety) | Private hold-out suites; rotate tasks; multi-suite min-pass; prohibit suite mutation without Authority tier | When to refresh suites |
| **Self-expanding authority** | Instrumental goal to disable off-switch / widen rights | Off-switch / promote path not controllable by agent (Hadfield-Menell et al. *Off-Switch Game* [arXiv:1611.08219](https://ar5iv.labs.arxiv.org/html/1611.08219)); capability bits only in operator-signed envelopes; deny `cap_sys_admin`-class rights to evolve services | Root key ceremony |
| **Rollback tampering** | Destroy last good slot or success flags | Bootloader/OS policy: current slot immutable during update; N successful boots required; rollback targets signed known-good; audit of slot state | Manual recovery UI |
| **Unsafe recursive change** | Self-mod modifies self-mod policy | Split **object code** (may evolve under gates) from **meta policy** (floors, roles, keys)—DGM notes open-ended parent selection is currently fixed/not self-modifiable and flags unmodifiable constitutional core as future work ([arXiv:2505.22954](https://ar5iv.labs.arxiv.org/html/2505.22954), §5) | What belongs in the frozen core |

Wireheading-class failures require treating **metric computation and promote authorization as environmental resources the agent must not control**—Amodei’s “careful engineering / sandbox the reward” and “trip wires,” plus classic computer-security separation of duty.

### What machine enforcement can and cannot do offline

**Can enforce (with correct TCB placement):**

- Signature presence/threshold/expiry on promote and envelope change.
- Artifact digest match to signed target list.
- Build provenance shape and builder identity (local policy).
- Bit-reproducible rebuild equality when toolchain is pinned.
- Slot write only to inactive partition/deployment.
- Automatic boot fallback on verify failure.
- Resource ceilings (CPU, RAM, disk, device access) for candidate jobs.
- Append-only audit with monotonic indices and signed tree heads.
- Denial of network egress at runtime air-gap boundary.
- Rejection of tunable values outside signed min/max/enum sets.
- Non-compensable conjunction of floor predicates (all must pass; no score averaging away a floor fail).

**Cannot fully enforce (operator / human judgment):**

- Adequacy of floor definitions vs real harm.
- Whether shadow metrics capture user-visible faithfulness.
- Semantic quality of memory mutations beyond mechanical invariants.
- Acceptance of residual risk after a clean scorecard.
- Interpretation of novel failure modes absent from suites.
- Political/operational decision to expand capabilities or rotate roots after suspected compromise.
- True “provably beneficial” self-modification (Gödel machine proof obligation is impractical; DGM states formal proofs of beneficial rewrites are infeasible without restrictive assumptions).

## Options and trade-offs

### Pattern A — Dual-slot production + signed artifacts (Android/OSTree lineage)

**Loop:** build candidate → verify signature & provenance → write inactive slot/deployment → reboot/switch → health verify → mark successful **or** automatic revert.

| Pros | Cons |
|------|------|
| Strong rollback story under power loss and failed boot | Coarse-grained (whole system/service tree); weak for partial memory/schema evolves |
| Production immutability during apply is battle-tested | Needs careful success-mark privilege separation |
| Fits one physical node without traffic split | Slow iteration; poor fit for high-frequency tunable search |

**Best fit tier:** P for code/runtime images; support for C packaging.

### Pattern B — Role-separated metadata control plane (TUF-like)

**Loop:** offline Root signs role keys → automated processes may refresh Timestamp/Snapshot only → Targets for production require operator threshold → client rejects rollback/mix-and-match/stale metadata.

| Pros | Cons |
|------|------|
| Explicit threat model for compromised online keys | Does not by itself evaluate *behavioral* fitness |
| Clear map to Authority vs Promote vs automated roles | Operational key ceremony burden on single operator node |
| Works fully offline once keys and mirrors are local | Easy to misconfigure delegation into “agent is Targets” |

**Best fit tier:** A and P; wraps any artifact type (models, schemas, containers).

### Pattern C — Empirical self-improve archive (ADAS / DGM lineage)

**Loop:** sample parent → mutate own code/config → sandbox evaluate on suites → archive if valid → repeat; optionally open-ended archive to escape local optima.

| Pros | Cons |
|------|------|
| Demonstrated automated discovery of better agent designs on coding benches (DGM: SWE-bench 20%→50% under paper setup; ADAS large gains vs hand agents on several benches—**vendor/paper-reported, specific dates/models in papers**) | Benchmark≠production intent; safety section warns of misalignment amplification |
| Open archive avoids single brittle hill-climb | Compute-heavy; needs strong sandbox; recursive self-mod of meta-policy is dangerous |
| Matches “generate many candidates” for Living Database research | Must **not** own production promote |

**Best fit tier:** C generation + scoring only; **never** autonomous P/A.

### Pattern D — Progressive delivery / shadow+canary analysis (SRE / Argo lineage)

**Loop:** deploy candidate to shadow or small exposure → compare SLI-linked metrics to control → auto-abort or manual judgement → full promote.

| Pros | Cons |
|------|------|
| Ties decisions to reliability science (error budgets, SLIs) | One-node traffic canaries are weak; isolation contamination real |
| Explicit manual judgement step for high risk | Metric selection is hard and gameable (Goodhart) |
| Composable with A/B slots | Needs high-quality local workload replay to substitute for multi-instance canary |

**Best fit tier:** evaluation phase before P; T may use a miniaturized version inside envelopes.

### Composite implied by evidence (not a selection)

A defensible GZMO **research baseline** (still not a final architecture choice) is: **C generates → hermetic build + SLSA-shaped provenance → Pattern D shadow/soak against non-compensable floors → Pattern B operator-signed Targets → Pattern A slot flip with bootloader/OS rollback → CT-like audit of every transition.** Memory (M) and tunables (T) short-circuit only through floor gates and envelopes, never through Targets forgery.

## Constraints for GZMO

Derived mandatory **invariants** (machine-checkable where noted). These narrow design space; they do not choose among Patterns A–D implementations.

1. **Single physical node; runtime air-gap.** No evolve path may require outbound network. Pre-provisioned content only. (Sovereignty floor.)
2. **Separation of duty in process identity.** Distinct OS identities (or stronger capability domains) for: builder, evaluator, production server, audit sink, promote verifier. Candidate code runs only as builder/eval subject.
3. **Non-writable TCB.** Floor predicates, promote verifier, slot manager, audit integrity keys, and root role keys are not in any agent write set. Prefer measured boot / read-only volumes where available.
4. **Non-compensable floors.** Faithfulness, sovereignty, reliability, resource, audit, rollback gates are **conjunctions**. A usefulness gain cannot offset a floor fail. (Aligns with ticket 00 fitness contract.)
5. **Envelope-bounded autonomy.** Tunable self-promote allowed only inside operator-signed envelopes; envelope edits are Authority tier.
6. **Operator-signed production binding.** Code/schema/model/security/capability artifacts need an operator signature (threshold optional but recommended for high-risk) over digest+policy before slot bind.
7. **Inactive-target mutation.** Updates write only non-serving slots/deployments/databases clones; serving state flips is atomic and reversible within declared RPO/RTO.
8. **Evaluator integrity.** Golden tests and judges are hashed into Root/Targets metadata; candidates cannot add “always pass” tests without Authority signature.
9. **Append-only evolution audit.** Every generate/eval/promote/rollback/envelope event is logged with prev-hash or Merkle inclusion; consistency proofs available to operator tooling.
10. **Rollback path outlives candidate.** Last-known-good digests pinned in signed metadata; agent cannot delete LKG or mark a bad slot successful without verifier policy.
11. **No self-issued authority.** Agent-held keys, if any, are limited to candidate labeling and memory writes—not Root, not production Targets, not audit roots.
12. **Corrigibility of evolve daemon.** Stop/disable evolve is always available to operator and is not blocked by agent policy; prefer hardware or init-level controls (*Off-Switch Game* motivation).
13. **Local containers OK; container escape is in threat model.** Container isolation is necessary but not sufficient; treat breakout as requiring still-valid signature checks on host slot manager.
14. **Reproducibility where feasible.** Release builds aim at bit-reproducible artifacts; when not feasible, require stronger dual-control review and provenance (document exceptions).
15. **Memory autonomy ≠ silent schema break.** Autonomous memory may not change production schema/API contracts; those are Candidates.
16. **Research ≠ license to implement.** This brief forbids autonomous production mutation in any prototype until later ADRs choose mechanisms.

## Unknowns

- Exact **one-node** substitute for multi-replica canary statistical power on GZMO workloads (needs workload capture design; ticket 11).
- Whether **model weight** updates can be made meaningfully reproducible/attestable at acceptable cost on edge hardware.
- Minimum **threshold cryptography / split operator keys** ceremony that a single human operator will actually perform air-gapped without unsafe bypass.
- How to measure **faithfulness** floors with low Goodhart risk on living memory (related local doctrine exists under keep-quality gates; not universal).
- Formal mapping of GZMO organs to slot granularity (whole appliance vs per-service ostree-like deployments vs DB clone promote).
- Interaction with ticket **06** air-gap threat model (supply chain of pre-provisioned media, evil maid, compromised builder bootstrap).
- Whether seL4/CHERI-class hardware capability enforcement is in scope for the reference node or only Linux containers + classic DAC/MAC.
- Long-term **recursive** improvement of the *evaluator* itself without evaluator-capture (open research; DGM/ADAS treat this as caution zone).
- Legal/operational definition of “operator” if multiple humans share one node over time (key handover).

## Primary sources

Standards, specs, and first-party manuals

- [The Update Framework Specification v1.0.36 (5 Aug 2026)](https://theupdateframework.github.io/specification/latest/) — roles, threats (rollback, freeze, mix-and-match, key compromise), offline root.
- [SLSA specification v1.0 — levels](https://slsa.dev/spec/v1.0/levels) and [attestation model](https://slsa.dev/spec/v1.0/attestation-model); [provenance concept (v1.2 track page)](https://slsa.dev/spec/v1.2/provenance).
- [in-toto attestation framework spec](https://github.com/in-toto/attestation/tree/main/spec); [in-toto project about](https://in-toto.io/about/).
- [DSSE: Dead Simple Signing Envelope](https://github.com/secure-systems-lab/dsse).
- [Reproducible Builds — definitions](https://reproducible-builds.org/docs/definition/).
- [RFC 6962 Certificate Transparency](https://www.rfc-editor.org/rfc/rfc6962) (experimental; see also RFC 9162 as successor for modern CT).
- [Android AOSP — A/B (seamless) system updates](https://source.android.com/docs/core/ota/ab).
- [OSTree — Atomic upgrades](https://ostreedev.github.io/ostree/atomic-upgrades/).
- [Google SRE Workbook — Canarying Releases](https://sre.google/workbook/canarying-releases/).
- [Argo Rollouts](https://argoproj.github.io/rollouts/).

Papers (primary)

- Amodei, Olah, Steinhardt, Christiano, Schulman, Mané — *Concrete Problems in AI Safety* (2016), [arXiv:1606.06565](https://ar5iv.labs.arxiv.org/html/1606.06565).
- Hadfield-Menell, Dragan, Abbeel, Russell — *The Off-Switch Game* (2016), [arXiv:1611.08219](https://ar5iv.labs.arxiv.org/html/1611.08219).
- Hu, Lu, Clune — *Automated Design of Agentic Systems* (2024), [arXiv:2408.08435](https://ar5iv.labs.arxiv.org/html/2408.08435).
- Zhang, Hu, Lu, Lange, Clune — *Darwin Gödel Machine* (2025), [arXiv:2505.22954](https://ar5iv.labs.arxiv.org/html/2505.22954) (includes safety discussion: sandboxing, oversight, benchmark limits).
- Schmidhuber — Gödel machine framing as cited in DGM (theoretical self-mod with proof obligation; practical infeasibility of proofs motivates empirical gates).

Local constitutional anchors

- `GZMO/.scratch/self-developing-living-database/issues/00-north-star-framing.md` — approved autonomy split and fitness floors (2026-08-31).
- `GZMO/.scratch/self-developing-living-database/issues/04-gated-autonomous-evolution.md` — this ticket’s question.
- `GZMO/.scratch/self-developing-living-database/issues/11-choose-evolution-loop.md` — downstream choice blocked on this research.
- Related measurement doctrine (not evolution control plane, but floor-gate precedent): `GZMO/research/essential-living-diagnosis/05-quality-gates.md`.

### Note on claims

Benchmark percentages from ADAS/DGM are **paper-reported under their experimental setups and dates**, not independent GZMO reproductions. SLSA “hosted build L2/L3” benefits must be **reinterpreted** for a single air-gapped node as isolation and key hygiene properties, not as adoption of public CI. No product purchase or implementation is recommended here.
