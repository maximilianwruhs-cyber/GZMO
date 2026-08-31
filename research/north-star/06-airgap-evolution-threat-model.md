# 06 — Airgap and self-evolution threat model

**Research date:** 2026-08-31  
**Scope:** Portable boot media + one physical air-gapped node + hostile corpora + local models + self-generated change candidates.  
**Non-goals:** Network-dependent controls; implementation; product vendor selection; relaxing constitutional floors.

## Executive finding

An air-gapped Self-developing Living Database is not “offline therefore safe.” The dominant trust problem is **offline supply-chain + content-as-code**: portable media, model/update bundles, hostile corpora, retrieval indexes, and self-generated candidates can all carry executable intent without a live network path. Primary standards converge on a single design shape:

1. **Hardware/firmware roots** protect, detect, and recover platform integrity (NIST SP 800-193 Protection / Detection / Recovery).
2. **Measured boot + sealed storage** bind durable secrets and data unlock to a known boot state (TCG PC Client firmware profile via UAPI PCR registry; systemd TPM2 LUKS enrollment).
3. **Role-separated, threshold-signed offline update metadata** with explicit anti-rollback/freshness (TUF v1.0.36 as of 2026-08-05).
4. **Content and model artifacts treated as untrusted inputs** until digest + signature + policy gates pass (OWASP LLM03/04/08; NIST AI 100-2e2025 GenAI supply-chain and indirect prompt injection taxonomies; MITRE ATLAS AML.T0010 / T0051 / T0070 / T0018).
5. **Authority envelopes that cannot be widened by the agent** (OWASP LLM06 Excessive Agency; GZMO North Star autonomy boundary).

For GZMO specifically, existing doctrine already encodes several non-negotiable floors that this threat model must preserve rather than reinvent: one physical writer, airgap honesty, honeypot+verify+promote, operator ack for promote-by-loop, dual-writer FAIL, and non-compensable faithfulness/sovereignty/reliability/resource/audit/rollback floors (`docs/ADR-0003`–`0007`, `docs/AIRGAP_LIVING.md`, `MACHINE.md`, wayfinder map).

**Decision-relevant summary:** treat the appliance as three nested trust domains—(A) firmware/boot/OS integrity, (B) signed offline catalogs (OS, containers, models, policy envelopes), (C) untrusted runtime content and candidate evaluation—and never allow C to mint authority in A or B. Memory may evolve autonomously inside C→honeypot gates; code/schema/model binary/security policy/capability expansion require operator-held signatures outside the runtime key material.

## Decision-relevant facts

### 1. Assets (what must not be lost, forged, or silently degraded)

| Asset class | Examples | Failure mode if compromised |
|---|---|---|
| **Identity & keys** | Operator root/threshold keys; node identity; LUKS volume keys; recovery keys; capability-envelope signing keys | Attacker becomes the authority of record; recovery may be impossible without out-of-band root rotation (TUF root-threshold compromise warning) |
| **Platform firmware & boot chain** | UEFI/BIOS, option ROMs, bootloader, UKI/initrd, Secure Boot policy (PK/KEK/db/dbx), measured PCRs | Persistent implant, permanent brick, or secret unseal under attacker-controlled measurement (NIST SP 800-193; PCR 0–7 ownership) |
| **Immutable system image** | Read-only root / A/B slots / dm-verity root hash | Silent OS/runtime replacement; evaluator sandbox breakout substrate |
| **Durable living state** | Vault, honeypot, supersession chains, audit/event ledger, capability envelopes, model catalog metadata, machine-id | Poisoned memory as “truth”; rollback to vulnerable past; loss of auditability |
| **Model binaries & adapters** | Base GGUF/weights, LoRA/PEFT adapters, tokenizers, eval fixtures | Backdoored inference, pickle/malware load paths, safety-strip fine-tunes (OWASP LLM03/04; ATLAS AML.T0010.003 / T0018) |
| **Offline update/media bundles** | Boot USB, update USB, model packs, corpus packs, signed manifests | Classic package-manager class attacks offline (TUF attack set) |
| **Untrusted corpora & retrieval indexes** | Ingest folders, Qdrant/vector stores, wiki derived pages, takeaways | Indirect prompt injection / RAG poisoning / secret leakage into recall (OWASP LLM01/08; ATLAS AML.T0051.001 / T0070) |
| **Self-generated candidates** | Proposed code, schema, tunables, prompts, security-policy diffs, capability expansions | Candidate escape; authority escalation; reward hacking of evaluators |
| **Operator workflow & recovery media** | Recovery keys, spare boot media, offline root ceremony materials | Media loss = permanent lockout or forced insecure recovery |
| **Constitutional floors** | Faithfulness, sovereignty, one-writer, resource floors, audit, rollback | Softened into “best effort” by multi-objective optimization |

Local evidence of current product assets and gates: pipeline `extract → verify → promote → vault → honeypot` (`MACHINE.md`); living quality pillars including Immune / Airgap honesty (`docs/KEEP_QUALITY.md`); dual-writer refuse paths in attach/enqueue scripts and ADRs.

### 2. Trust roots (what the system may ultimately believe)

Ordered from hardest to softest:

1. **Operator physical custody + offline root key ceremony**  
   Human operator is the only party allowed to expand capability, rotate roots, or accept first-install identity. TUF: client ships with trusted root keys; root private keys MUST stay offline; threshold root compromise is treated as near-total loss.
2. **Platform RoT (firmware protection/detection/recovery)**  
   NIST SP 800-193: Root of Trust for Update (RTU), Detection (RTD), Recovery (RTRec). Prefer authenticated firmware update, detection of corruption, and local recovery images—not network-assisted recovery for airgap runtime.
3. **UEFI Secure Boot policy + measured boot PCRs**  
   PCR 7 reflects Secure Boot state/cert databases; PCR 4 boot loader; PCR 11 UKI/kernel image + boot phase (systemd-stub / systemd-pcrphase); PCR 14 shim MOK when used (UAPI.7 PCR Registry; systemd-cryptenroll guidance prefers binding to PCR 7/11/14 over brittle PCR 0/2 code hashes).
4. **TPM-sealed disk encryption + recovery key**  
   systemd-cryptenroll: LUKS2 can enroll TPM2 (optionally PCR-bound or signed-policy-bound), FIDO2, PKCS#11, passphrase, and high-entropy recovery keys. Secrets unseal only in expected measurement state.
5. **Offline repository root (TUF-style roles)**  
   Root → Targets / Snapshot / Timestamp (+ optional Mirrors). Snapshot prevents mix-and-match; targets carry hashes/sizes; versioning + expiry fight rollback and freeze. For airgap, “timestamp freshness” becomes **operator-presented bundle generation time + monotonic local watermark**, not network polling.
6. **Artifact attestations (in-toto / SLSA provenance predicates)**  
   Authenticated metadata about *how* an artifact was produced (envelope → statement → predicate). Useful for OS/container/model build provenance verified offline against pinned builder identities—not as a substitute for operator targets signatures.
7. **Content signing keys distinct from root**  
   Cosign/Sigstore patterns: airgap must prefer **self-managed offline keys** (`cosign sign --key`) and offline verification bundles; public Fulcio/Rekor keyless flow assumes network identity issuers and transparency logs and is **not** a runtime dependency.
8. **Runtime policy engine / capability envelopes**  
   Softest root: local policy that interprets signatures and refuses unsigned authority expansion. Must not hold the private keys that could rewrite itself into a higher tier.

**Explicit non-root:** local LLM outputs, honeypot contents, eval scores, “benchmarks say safe,” USB presentation order, container image tags without digest, model cards without signatures, wiki pages (derived, never re-ingested per `WIKI.md`).

### 3. Actors

| Actor | Intent | Typical access |
|---|---|---|
| **Primary operator** | Legitimate owner; may err | Physical node, recovery keys, offline root keys (ideally split) |
| **Secondary operator / delegate** | Day-2 corpus load, bundle apply | Limited signing role or no signing; physical USB |
| **Supply-chain adversary** | Malicious media, models, adapters, “helpful” update packs | Controls files before airgap crossing |
| **Content adversary** | Poison documents, resumes, code comments, images | Writes only untrusted corpora |
| **Physical adversary** | Theft, evil-maid, port implants, cold-boot-adjacent | Intermittent physical access |
| **Compromised insider process** | Malware already on node | Local code exec below or beside sandbox |
| **Self-evolution process** | Optimize fitness; may reward-hack | Candidate generation + shadow eval APIs only |
| **Evaluator/judge models** | Score candidates; may be manipulated | Read candidate + fixtures; no prod write |

### 4. Trust zones

```text
┌─ Z0 Operator ceremony (offline keys, recovery, first identity) ─────────┐
│  Never co-located long-term with production node private material       │
└─────────────────────────────────────────────────────────────────────────┘
        │ signed bundles / recovery media only
        ▼
┌─ Z1 Platform (firmware RoT, Secure Boot, TPM) ──────────────────────────┐
│  Measured boot → unlock Z2                                              │
└─────────────────────────────────────────────────────────────────────────┘
        ▼
┌─ Z2 Immutable appliance OS + signed runtime (dm-verity/A/B, containers) ┐
│  Policy verifier, one-writer daemon, local engines                      │
└─────────────────────────────────────────────────────────────────────────┘
        ▼
┌─ Z3 Durable encrypted state (vault/honeypot/audit/envelopes/catalog) ───┐
│  TPM-bound LUKS; integrity-checked ledgers                              │
└─────────────────────────────────────────────────────────────────────────┘
        │
   ┌────┴──────────────────────────┐
   ▼                               ▼
┌─ Z4 Untrusted ingest ─┐   ┌─ Z5 Candidate sandbox ─────────────────────┐
│ corpora, USB drops,   │   │ no prod credentials; no raw block devices; │
│ model *proposals*     │   │ no network namespace; resource caps;       │
│ never auto-exec       │   │ ephemeral FS; signed promote path only     │
└───────────────────────┘   └────────────────────────────────────────────┘
        │                               │
        └──────────► Z6 Recall/MCP attach (stdio/localhost only) ◄────────┘
                     least privilege; no unsigned schema/code promote
```

**Crossing rules**
- Z4 → Z3 only through verify/honeypot gates (memory path).
- Z5 → Z2/Z3 executable/schema/model/policy only with **operator signature** matching envelope.
- Z6 never reaches Z0 keys.
- No zone may open outbound network as a control dependency (constitutional airgap).

### 5. Attacker capabilities (assume true unless hardware profile proves otherwise)

**Physical / media**
- Full read/write to portable boot and update media before first use and between uses.
- Evil-maid: reboot, replace ESP, downgrade bootloader if Secure Boot/PCR binding weak.
- Steal node and media; attempt offline LUKS brute force if only passphrase and weak entropy.
- Implant MCU/firmware on pluggable devices (NIST SP 800-193 scope includes option ROMs, NIC/GPU/storage firmware—not only host BIOS).

**Offline supply chain**
- Craft bundles that replay old-but-valid signatures (rollback), freeze clients on last-seen metadata, mix targets from different times, overflow with endless data, or substitute wrong trusted file (TUF §1.5.2 attack list; still fully relevant without network MITM because the USB *is* the channel).
- Publish poisoned models/adapters with convincing cards and benchmark scores (OWASP LLM03; PoisonGPT-class parameter edits; malicious pickle; LoRA bolt-ons).
- Strip or forge licensing metadata.

**Content / retrieval**
- Direct and indirect prompt injection, including hidden text, multimodal stego, split payloads (OWASP LLM01; NIST AI 100-2e2025 GenAI indirect prompt injection).
- RAG poisoning and false entry injection (OWASP LLM08; ATLAS AML.T0070 / T0071).
- Embedding inversion / cross-context bleed if multi-tenant vectors ever share indexes (less relevant on single-operator node, still relevant across “lab vs living” data if mis-bound).

**Self-evolution**
- Generate candidates that disable gates, widen tools, or propose “temporary” network for eval.
- Poison eval fixtures or judge prompts so unsafe candidates score high (evaluator compromise).
- Escape sandbox via confused-deputy tool calls (OWASP LLM05/06; ATLAS agent tool invocation / escape-to-host).
- Attempt authority escalation: self-sign capability expansion using keys reachable from Z5/Z6.

**Operational**
- Exhaust local resources (OWASP LLM10 unbounded consumption) to force fail-open behavior.
- Induce dual-writer or second metabolism if mutex is advisory only (local scar: ADR-0003/0005 dual-writer FAIL doctrine).

### 6. Standard mechanisms that map cleanly (availability as of 2026-08-31)

| Need | Primary mechanism | Status note |
|---|---|---|
| Firmware resiliency | NIST SP 800-193 P/D/R + RTU/RTD/RTRec | Final guideline (2018-05); procurement-relevant, not a product |
| Measured boot PCR semantics | TCG PC Client PFP (authoritative for 0–7); UAPI.7 Linux PCR Registry | Linux OS bindings documented; Windows out of scope for UAPI |
| Disk unlock bound to boot | systemd-cryptenroll TPM2 ± PCR or signed PCR policy; FIDO2; recovery key | Current in systemd 261 docs |
| OS image integrity | dm-verity (kernel device-mapper verity target): read-only data + hash tree; root hash as trust anchor | Upstream kernel feature; root hash must be in signed boot chain |
| Offline update security | TUF roles, thresholds, snapshot consistency, rollback/freeze defenses | Spec v1.0.36 (modified 2026-08-05); designed to work without TLS |
| Build provenance | in-toto attestation + SLSA provenance predicate | Verify offline; does not replace targets signing |
| Artifact signatures | Cosign key-based sign/verify; optional attest predicates | Keyless/Fulcio/Rekor need net—**exclude from runtime airgap path** |
| LLM/content threats | OWASP Top 10 for LLM Apps 2025; NIST AI 100-2e2025; MITRE ATLAS | Taxonomy + mitigations; not push-button controls |
| Agency bounding | OWASP LLM06 least privilege, no open-ended shell, human approval for high impact | Aligns with North Star signed approval tiers |

### 7. Threat → control mapping (mandatory controls by surface)

#### Boot & portable media
- **Threats:** evil-maid bootloader swap; malicious autorun; recovery-image replacement; firmware brick.
- **Controls:** UEFI Secure Boot with controlled PK/KEK/db; measured boot; UKI measured into PCR 11; dm-verity root; A/B slots with signed metadata; boot media is **read-prefer** and verity-checked; disable autorun; separate **install/recovery** media identity from **day-2 data** media; NIST recovery image for firmware where hardware supports it.
- **Invariant:** untrusted USB content never executes before signature + verity verification in Z2.

#### Storage
- **Threats:** stolen disk; hot-plug data theft; silent bit-flip or targeted corruption of vault/audit; LUKS header downgrade.
- **Controls:** LUKS2 full-disk or dedicated data volumes; TPM2 seal to PCR set {7,11,(14)} or vendor-signed PCR policy; always enroll **recovery key** held offline; integrity for audit ledger (append-only, hash-chained, ideally separate MAC key sealed to TPM); periodic detection jobs comparing Merkle/head hashes to last operator-witnessed watermark.
- **Invariant:** production state is ciphertext at rest whenever node is off or measurements mismatch.

#### Model catalog
- **Threats:** poisoned base models, malicious adapters, unsigned format conversion, license laundering, stale vulnerable models.
- **Controls:** catalog entries are TUF targets (hash, size, version, license SPDX, eval suite id, architecture, quantization); refuse pickle-executing load paths for untrusted artifacts; adapters signed and bound to base digest; red-team/eval gates before enablement; licensing metadata is **policy-enforced**, not advisory.
- **Invariant:** no model loads without digest match to signed catalog; license deny fails closed.

#### Evolution loop
- **Threats:** candidate escape; gate deletion; evaluator poisoning; autonomous promotion of code/schema/model/security/capability; dual-writer.
- **Controls:**  
  - **Memory path (autonomous allowed):** ingest → extract → verify → promote → vault → honeypot qualify; supersession; immune/closed-class reporting (`KEEP_QUALITY`, `MACHINE.md`).  
  - **Tunable path (bounded):** only keys listed in operator-signed capability envelope; numeric bounds; automatic rollback on floor breach.  
  - **Hard path (operator-signed only):** code, schema, model binaries, security policy, envelope expansion—produce candidate in Z5; shadow eval; never apply without detached signature verifying against Z0/Z1 operator targets role.  
  - Sandbox: no production credentials, no `/dev` raw disks, no mutable host mounts, cgroup/memory/time caps, seccomp/landlock/capability drop, separate user + mount + IPC namespaces; open-ended shell tools forbidden (LLM06).  
  - Evaluators are **not** trusted oracles: fixed fixture digests, multi-objective **non-compensable floors first**, human-visible diff, fail closed on instrument failure.
- **Invariant:** the process that proposes cannot be the process that expands its own authority.

#### Operator actions
- **Threats:** lost keys; social-engineered “emergency” unsigned apply; accidental dual-writer; cloud opt-in creep.
- **Controls:** split root threshold (m-of-n offline keys); recovery runbooks that re-establish identity from Z0; every promote-by-loop requires ack + dual_writer_risk=false (ADR-0005/CONTINUOUS_UPGRADE); airgap honesty checklist (`AIRGAP_LIVING.md`); cloud LLM remains explicit opt-in and never on core path; media loss procedures (below).
- **Invariant:** convenience never bypasses signature verification (“break-glass” still signs with recovery role and logs immutably).

### 8. Scenario catalog (required coverage)

| Scenario | Primary standards refs | Mandatory posture |
|---|---|---|
| **Prompt / data poisoning** | OWASP LLM01/04; NISTAML prompt & data poisoning IDs | Untrusted content labeled; never executes tools directly; verify gates before honeypot |
| **Retrieval poisoning** | OWASP LLM08; ATLAS AML.T0070/T0071 | Provenance tags on chunks; canary/honeypot probes; refuse wiki re-ingest (`WIKI.md`) |
| **Malicious model/update bundle** | OWASP LLM03; TUF; in-toto/SLSA; ATLAS AML.T0010 | Offline TUF verify; digest pin; no tag-only trust |
| **Candidate escape** | OWASP LLM05/06; ATLAS escape/tool techniques | Hard sandbox; complete mediation in policy engine, not in prompt |
| **Authority escalation** | TUF role separation; North Star autonomy boundary | Envelope expansion = root/targets ceremony |
| **Secrets leakage** | OWASP LLM02/07; ATLAS credential harvesting | No secrets in prompts/logs; MCP least privilege; sealed key material |
| **Rollback poisoning** | TUF rollback/fast-forward/freeze | Monotonic watermark + signed snapshot versions on node |
| **Stale/replayed bundles** | TUF freeze + expiry | Bundle `expires` + local last-seen; operator must present newer snapshot |
| **Physical access / media loss** | NIST SP 800-193 recovery; systemd recovery key | Steal ≠ plaintext; lost media ≠ lost root if threshold survivors exist |
| **Evaluator compromise** | NIST AI 100-2 (integrity of judgments); LLM09 misinformation | Floors independent of LLM judge; golden fixtures; multi-judge optional but not sole gate |
| **Licensing metadata attack** | OWASP LLM03 licensing risks | Machine-enforced allowlist; deny unknown/NC if product requires |

### 9. Residual risks (accepted, not “solved by architecture”)

1. **Prompt injection is not fully preventable** (OWASP LLM01 explicitly). Mitigation is impact bounding, not perfect filtering.
2. **Firmware implants below or beside measured components** (malicious BMC/NIC/GPU firmware) remain hard; SP 800-193 helps procurement but does not eliminate advanced physical supply-chain risk.
3. **TPM and Secure Boot do not stop an attacker who also obtains operator threshold keys or recovery keys.**
4. **Threshold root compromise ≈ assume malware ownership** (TUF). Safe recovery is nearly impossible without rebuild from known-good media and new identity.
5. **Model behavioral backdoors / sleeper agents** may pass limited eval suites (Anthropic sleeper-agent literature cited by OWASP LLM04). Residual: continuous canaries + narrow tool surface.
6. **Operator error** (applying wrong bundle, disabling Secure Boot “to debug,” storing recovery key with the node) dominates many real failures.
7. **Availability under endless hostile corpora** can still DoS local resources; fail **closed** (pause ingest), never fail open to skip verify.
8. **No remote attestation consumer** in pure airgap: measurements protect local unseal and operator-inspected logs, not continuous third-party monitoring.
9. **Sigstore public good transparency** is unavailable at runtime by design; offline key compromise detection is operator audit duty.
10. **Hardware without TPM/Secure Boot** cannot meet the full binding story; capability ladder must **declare** reduced assurance rather than silently claim parity (wayfinder hardware ladder constraint).

### 10. Recovery assumptions (must be true for the model to hold)

1. At least one **known-good recovery boot medium** exists, verity-signed, stored separately from the node.
2. **Recovery keys** for LUKS exist offline; loss of TPM alone is survivable; loss of recovery + TPM + all passphrases is data-loss.
3. **Root/threshold operator keys** are not solely stored on the living node or its day-2 USB.
4. Compromised candidate recovery = discard Z5, restore last signed A/B slot and last known-good state snapshot; **never** “hotfix” by letting the candidate patch production.
5. Suspected root compromise = stop metabolism, boot recovery, reinstall platform from known-good, re-init identity, restore data only after offline scan + watermark verification.
6. Media loss (boot USB stolen): rotate device identity if private material might have been on media; treat media as attacker-owned; do not reuse unencrypted private keys from it.
7. Dual-writer or audit break = **FAIL** and halt promotes (existing GZMO doctrine), not warn-and-continue.

## Options and trade-offs

| Option | Pros | Cons | Airgap fit |
|---|---|---|---|
| **A. Full measured + verity + TUF + TPM-sealed data (recommended direction)** | Matches primary standards; clear zones; anti-rollback | Higher bring-up cost; hardware TPM dependency on reference node | Strong |
| **B. Secure Boot + signed images only (no TPM seal)** | Simpler disks | Stolen disk = plaintext if OS unlocked or weak passphrase-only | Medium |
| **C. Application-level signatures only (no platform RoT)** | Portable across odd boards | Evil-maid and firmware persistence win | Weak — fails physical threat |
| **D. Networked Sigstore keyless + Rekor** | Easy CI UX | Runtime/net dependency; identity issuers outside operator control | **Rejected for runtime** |
| **E. Trust model cards + hashes without roles/thresholds** | Low ceremony | Single key compromise, mix-and-match, freeze unprotected | Weak — TUF explicitly warns |
| **F. LLM-as-sole-gate for promotes** | Autonomous feel | Evaluator compromise = full integrity loss | **Rejected** |

Trade-off judgment for North Star design (not a final operator product choice): **Option A** is the only posture that simultaneously covers portable media, physical theft, offline replay, and self-evolution authority boundaries without inventing network dependence.

## Constraints for GZMO

Non-negotiable invariants (compose wayfinder + ADRs + this model):

1. **One physical node** runtime; local containers allowed; no cloud/second-machine control plane.
2. **Airgap honesty:** core extract/verify/dream/distill/recall must not require public net (`ADR-0004`/`0005`/`0007`, `AIRGAP_LIVING.md`).
3. **One overnight writer** per vault; dual-writer is FAIL (`ADR-0003`/`0005`, attach scripts).
4. **Autonomy boundary:** memory autonomous; bounded tunables inside signed envelopes; code/schema/models/security/capability expansion operator-signed (wayfinder map; issue 00 answer).
5. **Non-compensable floors:** faithfulness, sovereignty, reliability, resource, audit, rollback — scores may not buy past a floor failure.
6. **Honeypot path remains the memory integrity spine** (`MACHINE.md`); Qdrant is mirror/search, not an unsigned write backdoor (local CORE claims / attach docs).
7. **Wiki and synthetic pages never re-enter honeypot** (`WIKI.md`) — blocks one retrieval-poisoning amplifier.
8. **Offline verification only:** TUF/cosign/in-toto verify against **operator-pinned roots** present on device; no phone-home.
9. **Capability ladder honesty:** missing TPM/Secure Boot/verity ⇒ declared degraded trust profile, not silent equivalent.
10. **Break-glass still audited and signed**; unsigned apply is out of constitution.
11. **Candidate compromise recovery** restores last signed system+state; does not negotiate with the candidate.
12. **Licensing and model provenance** are hard denies in catalog policy.

## Unknowns

- Exact portable-media role (boot-only vs installer vs runtime) — owned by ticket 02; this model requires whatever is chosen to remain in Z1/Z2 verification before Z3 unlock.
- Reference hardware TPM quality (fTPM vs dTPM), firmware resiliency certifications, and discrete BMC exposure — ticket 01.
- Concrete envelope schema and promote API — ticket 04; must preserve authority split herein.
- Whether dm-verity + A/B lands as OSTree, RAUC, swupdate, systemd-sysupdate, or custom — mechanism choice open; properties are not.
- Operator threshold *m* and key custody UX (pedantic security vs solo operator practicality).
- How far local red-team suites can go on-device without special hardware for model backdoor detection.
- Quantized/GGUF-specific malware and parser CVEs over time — catalog must stay patchable offline.
- Legal interpretation of specific model licenses for sovereign commercial use (product/legal, not purely technical).

## Primary sources

### Standards, specs, and first-party docs
- [The Update Framework Specification v1.0.36](https://theupdateframework.github.io/specification/latest/) (last modified 2026-08-05) — roles, thresholds, rollback/freeze/mix-and-match defenses; offline-friendly (no TLS required).
- [TUF Security properties](https://theupdateframework.io/docs/security/) — attack list and freshness/integrity principles.
- [NIST SP 800-193 Platform Firmware Resiliency Guidelines](https://doi.org/10.6028/NIST.SP.800-193) ([PDF](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-193.pdf)) — Protection, Detection, Recovery; RTU/RTD/RTRec.
- [NIST AI 100-2e2025 Adversarial Machine Learning](https://doi.org/10.6028/NIST.AI.100-2e2025) ([PDF](https://nvlpubs.nist.gov/nistpubs/ai/NIST.AI.100-2e2025.pdf)) — PredAI/GenAI taxonomies; supply chain, prompt injection, indirect injection, agents (Mar 2025).
- [UAPI.7 Linux TPM PCR Registry](https://uapi-group.org/specifications/specs/linux_tpm_pcr_registry/) — PCR 0–15 Linux/systemd assignments; points to TCG PC Client PFP as authoritative for firmware PCRs 0–7.
- [TCG PC Client Specific Platform Firmware Profile Specification](https://trustedcomputinggroup.org/resource/pc-client-specific-platform-firmware-profile-specification/) — authoritative PCR 0–7 (fetch may require TCG portal; cited via UAPI).
- [systemd-cryptenroll](https://www.freedesktop.org/software/systemd/man/latest/systemd-cryptenroll.html) — LUKS2 TPM2/FIDO2/PKCS#11/recovery-key enrollment; PCR binding guidance.
- [dm-verity kernel documentation](https://docs.kernel.org/admin-guide/device-mapper/verity.html) — integrity-checked read-only block devices anchored by root hash.
- [in-toto Attestation Framework Spec](https://github.com/in-toto/attestation/blob/main/spec/README.md) — envelope/statement/predicate authentication model.
- [SLSA Provenance](https://slsa.dev/spec/v1.0/provenance) (see also [v1.2](https://slsa.dev/spec/v1.2/provenance)) — build provenance predicate for offline verification.
- [Sigstore Cosign signing overview](https://docs.sigstore.dev/cosign/signing/overview/) — keyless vs roots; TUF distribution of Sigstore roots.
- [Cosign signing containers / local keys](https://docs.sigstore.dev/cosign/signing/signing_with_containers/) — `cosign sign --key` self-managed keys; bundles without registry upload.

### LLM / AI threat taxonomies
- [OWASP Top 10 for LLM Applications 2025](https://genai.owasp.org/llm-top-10/) — index of LLM01–LLM10.
- [LLM01 Prompt Injection](https://genai.owasp.org/llmrisk/llm01-prompt-injection/)
- [LLM03 Supply Chain](https://genai.owasp.org/llmrisk/llm032025-supply-chain/)
- [LLM04 Data and Model Poisoning](https://genai.owasp.org/llmrisk/llm042025-data-and-model-poisoning/)
- [LLM06 Excessive Agency](https://genai.owasp.org/llmrisk/llm062025-excessive-agency/)
- [LLM08 Vector and Embedding Weaknesses](https://genai.owasp.org/llmrisk/llm082025-vector-and-embedding-weaknesses/)
- [MITRE ATLAS](https://atlas.mitre.org/) — matrix including AML.T0010 supply chain, AML.T0051 prompt injection, AML.T0070 RAG poisoning, AML.T0018 model manipulate, agentic techniques.

### Local project evidence (not external authority; constraints)
- `GZMO/.scratch/self-developing-living-database/map.md` — topology, autonomy boundary, non-compensable floors.
- `GZMO/.scratch/self-developing-living-database/issues/00-north-star-framing.md` — approved framing 2026-08-31.
- `GZMO/docs/ADR-0003-one-instance-metabolism.md`, `ADR-0004-airgap-living-usp.md`, `ADR-0005-flywheel-over-frozen-topology.md`, `ADR-0007-one-product-living.md` — one writer, airgap honesty, promote ack.
- `GZMO/docs/AIRGAP_LIVING.md` — single-box topology, airgap checklist, immune in overnight path.
- `GZMO/docs/KEEP_QUALITY.md` — living quality pillars including Immune and Airgap honesty.
- `GZMO/MACHINE.md` — honeypot + verify + promote identity.
- `GZMO/WIKI.md` — wiki never re-ingested into honeypot.
- `GZMO/docs/CONTINUOUS_UPGRADE.md` — beat-gate + operator ack + dual_writer_risk=false.

---

*This brief narrows design constraints. Final control selection, hardware qualification, and key-ceremony UX remain operator-owned decisions after tickets 01–05 and the integrated North Star design pass.*
