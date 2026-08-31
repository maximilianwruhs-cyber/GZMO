# 03 — Hardware-aware local model and runtime selection

**Research date:** 2026-08-31  
**Ticket:** `.scratch/self-developing-living-database/issues/03-hardware-aware-model-runtime.md`  
**Scope:** Offline boot discovery → capability manifest → signed model catalog → role-aware selection → fail-closed / honest-degrade. No downloads, no benchmarks on this workstation.

## Executive finding

A stable GZMO interface is a three-stage pipeline, not a model shopping list:

1. **Boot discovery** produces an immutable, signed **Hardware Inventory Record (HIR)** from firmware/driver facts only (SMBIOS/DMI, CPUID/ISA flags, PCI/DRM, NVML/`nvidia-smi`, ROCm/`rocminfo` **only when the GPU is on AMD’s published support matrix**, Vulkan ICD enumeration, memory/thermal/power caps).  
2. **Capability compiler** maps HIR → a **Capability Manifest (CM)** that names *enabled* backends, *explicit unavailable* backends, resource floors (RAM/VRAM/context/joules), and the **minimum capability profile** the node can honestly claim (`minimum` | `reference` | `forge` vocabulary is operator-owned; this brief only defines the measurement interface).  
3. **Model selector** matches a **signed offline Model Catalog** against CM + **task role**, then **cold-start qualifies** the chosen artifact. Runtime never invents CUDA/ROCm/Vulkan capability from marketing strings; missing backends stay `unavailable` and trigger declared degrade modes (FTS-only recall, paused distill, CPU-only extract) rather than silent cloud fallback.

**Primary runtime spine for the appliance:** pin **llama.cpp / GGUF** as the default local inference family (MIT-licensed engine; multi-backend CPU, CUDA, HIP/ROCm, Vulkan; OpenAI-compatible server with dedicated `--embedding` and `--rerank` modes; `--offline` for airgap). Treat TensorRT / Jetson-specific stacks, ONNX Runtime EPs, and SYCL/OpenVINO as **optional accelerator lanes** only when CM proves them and the catalog entry declares them. Do **not** treat multi-host embed/rerank (current VM200 pattern) as North Star topology—one physical node only.

**Authority boundary (constitution):** model binaries, runtime packages, and capability expansion remain **operator-signed**. Memory evolution may be autonomous; selecting a new model family or enabling a new backend class is not.

## Decision-relevant facts

### 1. Offline hardware inventory inputs (reliable, air-gapped)

| Domain | Primary source / interface | What to capture | Trust notes |
|--------|---------------------------|-----------------|-------------|
| System identity / CPU / RAM layout | [DMTF SMBIOS DSP0134](https://www.dmtf.org/standards/smbios) (v3.9.0, 19 Aug 2025); Linux `/sys/firmware/dmi/tables`, `dmidecode`; Windows WMI `Win32_ComputerSystem` / `Win32_Processor` / `Win32_PhysicalMemory` | vendor, product, UUID (hashed for privacy), CPU model, core/thread counts, installed RAM | Firmware tables are OS-present and OS-absent readable; do not probe hardware by guessing |
| CPU ISA features | CPUID / `/proc/cpuinfo` flags; Windows `IsProcessorFeaturePresent` | AVX2, AVX-512, AMX, NEON/SVE, etc. | Needed for CPU backend binary selection and BLAS path |
| Memory headroom | Linux `/proc/meminfo` (`MemTotal`, `MemAvailable`); Windows `GlobalMemoryStatusEx` | total + **available** at selection time | Available, not total, bounds cold-start |
| NVIDIA GPU | [NVML](https://docs.nvidia.com/deploy/nvml-api/nvml-api-reference.html) / `nvidia-smi --query-gpu=...`; [CUDA compute capability table](https://developer.nvidia.com/cuda/gpus) | name, UUID, driver version, total/free VRAM MiB, compute capability, power limit, persistence mode | Driver present ≠ CUDA toolkit present ≠ compute binary loadable |
| AMD GPU (ROCm path) | [ROCm 10.0.0 compatibility matrix](https://rocm.docs.amd.com/en/latest/compatibility/compatibility-matrix.html) (docs dated 2026-08-14); `rocminfo`, `rocm-smi` | gfx target (e.g. gfx1100), VRAM, driver/firmware versions | **ROCm only when GPU + OS + driver appear on the official matrix.** Unsupported consumer cards must not be claimed as ROCm-capable; use Vulkan or CPU instead |
| Vulkan | [Vulkan Loader architecture](https://github.com/KhronosGroup/Vulkan-Loader/blob/main/docs/LoaderInterfaceArchitecture.md); `vulkaninfo` | ICD list, device name, `apiVersion`, memory heaps, subgroup size, FP16 features | Loader discovers ICDs; presence of ICD ≠ GGML Vulkan ops complete for all models |
| PCI / DRM | Linux sysfs `/sys/bus/pci/devices`, `/sys/class/drm`; Windows SetupAPI/PCI | vendor:device IDs, driver binding | Cross-check vendor class before enabling backend |
| Thermal / power | Linux thermal zones; NVIDIA NVML power; AMD smi; Intel RAPL via [powercap sysfs](https://www.kernel.org/doc/Documentation/power/powercap/powercap.txt) (`/sys/class/powercap/intel-rapl:*/energy_uj`) | TDP hints, package energy counters | RAPL is microjoule counters; often root-restricted—see GZMO `docs/OBOLUS_ENERGY.md` |
| Storage for weights | `statvfs` / Windows volume APIs | free bytes on model volume | Catalog must refuse if artifact + scratch cannot fit |
| Edge SoC identity | Jetson module IDs from [Jetson Linux r36.4 Quick Start](https://docs.nvidia.com/jetson/archives/r36.4/DeveloperGuide/IN/QuickStart.html) (Orin NX/Nano/AGX Orin families) | module SKU, L4T version, unified memory | Jetson uses CUDA CC 8.7 (Orin); treat as NVIDIA-edge class |

**HIR schema (stable fields):**

```json
{
  "hir_version": "1.0",
  "collected_at": "ISO-8601",
  "node_id_hash": "sha256(...)",
  "cpu": {"model": "", "cores": 0, "threads": 0, "isa": ["avx2"], "ram_total_mib": 0, "ram_available_mib": 0},
  "gpus": [{"vendor": "nvidia|amd|intel|other", "name": "", "vram_total_mib": 0, "vram_free_mib": 0,
            "compute": {"cuda_cc": null, "rocm_gfx": null, "vulkan": false},
            "driver": {"name": "", "version": ""}}],
  "backends_probe": {
    "cpu": {"status": "ok|fail", "binary": "llama-server-cpu", "detail": ""},
    "cuda": {"status": "ok|unavailable|fail", "binary": "llama-server-cuda", "detail": ""},
    "hip": {"status": "unavailable", "reason": "gpu_not_on_rocm_matrix"},
    "vulkan": {"status": "ok|unavailable|fail", "detail": ""}
  },
  "energy": {"rapl_readable": false, "nvml_power_readable": false},
  "storage": [{"path": "/models", "free_mib": 0}],
  "probe_method": ["smbios", "nvml", "vulkaninfo", "binary_exec_help"]
}
```

**Probe discipline:** prefer read-only queries; for CUDA/HIP/Vulkan **binary** readiness, execute the shipped `llama-server-* --help` / `--list-devices` (pattern already in `GZMO/boot.sh`). Never claim a backend solely because a GPU name string matches a blog post.

### 2. Runtime backends for supported hardware classes

Sources: [llama.cpp README — Supported backends](https://github.com/ggml-org/llama.cpp/blob/master/README.md), [build.md](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md), [server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md).

| Class | Official path | When CM may enable | When CM must refuse |
|-------|---------------|--------------------|---------------------|
| **CPU** | llama.cpp CPU (+ optional BLAS/OpenBLAS/oneMKL); ISA: AVX/AVX2/AVX512/AMX, ARM NEON, RISC-V | Always baseline | Never “unavailable” if a CPU binary exists; may be `degraded` under RAM floors |
| **CUDA / NVIDIA discrete + Jetson Orin** | `GGML_CUDA=ON`; compute capability from NVIDIA table (Orin = 8.7; Ada = 8.9; Blackwell consumer = 12.0, etc.) | NVML sees GPU **and** CUDA-built binary loads **and** free VRAM ≥ catalog `min_vram_mib` | No driver, binary fails to load, CC below binary’s compiled arches (unless non-native multi-arch build) |
| **NVIDIA edge (Jetson)** | Jetson Linux L4T on Orin family; CUDA on-device; community/local LLM often via llama.cpp CUDA or other Jetson AI stacks | Module is Orin-class per NVIDIA docs; L4T present | Do not assume TensorRT-LLM is first-class on Jetson—NVIDIA forum guidance has historically distinguished desktop TensorRT-LLM from Jetson support; treat TensorRT as **optional** only with explicit catalog + measured load |
| **Vulkan** | `GGML_VULKAN=ON`; LunarG SDK / system `libvulkan` + `glslc`; verify with `vulkaninfo` | ICD enumerates a device; Vulkan binary loads; device memory heaps sufficient | No ICD, software-only ICD without operator allow, or load failure |
| **ROCm / HIP** | `GGML_HIP=ON` with ROCm install; `GPU_TARGETS=gfx…`; matrix-gated | GPU’s LLVM target + OS + amdgpu driver versions appear on [ROCm compatibility matrix](https://rocm.docs.amd.com/en/latest/compatibility/compatibility-matrix.html) | GPU absent from matrix; Windows without supported stack; `HSA_OVERRIDE_GFX_VERSION` hacks **must not** be used in production CM (unsupported override) |
| **Optional lanes** | ONNX Runtime EPs (CUDA, TensorRT, OpenVINO, DirectML, QNN, CoreML, …) per [ORT EP docs](https://onnxruntime.ai/docs/execution-providers/); Intel SYCL in llama.cpp; OpenVINO (in progress in llama.cpp) | Catalog artifact format is ONNX/OpenVINO **and** EP probe succeeds | Default LLM/embed path should not require ORT |

**Server capabilities that matter for GZMO roles** (llama-server):

- OpenAI-compatible chat + embeddings routes.  
- `--embedding` / `--embeddings` restricts server to embedding use.  
- `--rerank` / `--reranking` enables rerank endpoint ([PR #9510](https://github.com/ggml-org/llama.cpp/pull/9510) lineage; documented in server README).  
- `--offline` forces cache-only, no network.  
- `--list-devices`, `-ngl` / `--fit` for VRAM fit.  
- Default bind `127.0.0.1` (sovereignty-aligned).  
- Metrics optional via `--metrics`.

**Local evidence:** current living topology already separates Prime (`:8000`) from embeddings (`:8081`) and documents honest degrade when either is down (`docs/AIRGAP_LIVING.md`). `boot.sh` already probes CUDA vs CPU binaries and free VRAM before picking a GGUF—correct direction, but thresholds are hardcoded model names (not a signed catalog) and omit Vulkan/ROCm/role separation.

### 3. Signed offline model-catalog schema

Every shippable weight set is a **catalog entry** plus **content-addressed artifacts** on media. Operator signature covers the catalog document (and optionally each artifact). Unsigned or hash-mismatch artifacts are unloadable (fail-closed).

```json
{
  "catalog_version": "1.0",
  "catalog_id": "gzmo-models-2026-08",
  "signed_by": "operator-key-id",
  "signature": "base64-ed25519...",
  "entries": [
    {
      "artifact_id": "qwen2.5-7b-instruct-q4_k_m",
      "role": ["extract_verify", "conversational"],
      "family": "qwen2.5",
      "parameters_b": 7.61,
      "format": "gguf",
      "quantization": "Q4_K_M",
      "files": [
        {
          "path": "models/qwen2.5-7b-instruct-q4_k_m.gguf",
          "sha256": "...",
          "size_bytes": 0,
          "sigstore_or_cms": "optional-detached-sig"
        }
      ],
      "license": {
        "spdx": "Apache-2.0",
        "redistributable_on_media": true,
        "attribution_required": true,
        "notice_files": ["NOTICE/Qwen2.5.txt"],
        "use_policy_uri": null,
        "commercial_mau_gate": null
      },
      "architecture": {
        "arch": "qwen2",
        "context_train": 32768,
        "context_claim_max": 131072,
        "rope_scaling": "yarn_optional",
        "embedding_dim": null,
        "gguf_general": ["general.architecture", "general.file_type"]
      },
      "runtime_compat": {
        "engines": ["llama.cpp>=bXXXX"],
        "backends": ["cpu", "cuda>=8.0", "vulkan", "hip:gfx1100"],
        "min_ram_mib": 10240,
        "min_vram_mib_full_offload": 6144,
        "kv_cache_bytes_per_token_estimate": 0,
        "weights_bytes": 0
      },
      "resource_estimates": {
        "context_default": 8192,
        "context_max_qualified": 32768,
        "throughput_tps_hint": null,
        "joules_per_1k_tokens_hint": null,
        "estimate_provenance": "vendor_card|lab_measure|unmeasured"
      },
      "task_qualification": {
        "extract_verify": {"status": "required_gate", "baseline_id": "gzmo-extract-v1", "min_score": 0.9},
        "conversational": {"status": "smoke"},
        "code_candidate": {"status": "not_qualified"},
        "embed": {"status": "n/a"},
        "rerank": {"status": "n/a"}
      },
      "quality_baseline": {
        "suite_id": "keep-quality-extract-verify",
        "metrics": {"verify_pass_rate": 0.9, "faithfulness_floor": 0.9},
        "measured_on_profile": "reference",
        "measured_at": null
      },
      "min_capability_profile": "reference",
      "degrade_peer": "qwen2.5-3b-instruct-q4_k_m",
      "notes": "Apache-2.0 redistribution OK with NOTICE"
    }
  ]
}
```

**License / redistribution facts (shipping on airgap media):**

| Family example | License signal (primary) | Media shipping implication |
|----------------|--------------------------|----------------------------|
| Qwen2.5 Instruct | [Apache-2.0 on model card](https://huggingface.co/Qwen/Qwen2.5-7B-Instruct) | Redistributable with Apache terms (license copy, NOTICE, modification notices) |
| BGE-M3 embeddings | [MIT](https://huggingface.co/BAAI/bge-m3) | Redistributable; retain copyright |
| bge-reranker-v2-m3 | [Apache-2.0](https://huggingface.co/BAAI/bge-reranker-v2-m3) | Redistributable under Apache |
| Llama 3.1 | [Llama 3.1 Community License](https://raw.githubusercontent.com/meta-llama/llama-models/main/models/llama3_1/LICENSE) | Redistribution allowed **with** Agreement copy, “Built with Llama”, Notice file, Acceptable Use Policy; additional commercial terms if >700M MAU |
| Gemma (≤3.x lineage) | [Gemma Terms of Use](https://ai.google.dev/gemma/terms) (modified 2026-04-01) | Distribution requires passing use restrictions + Agreement + Notice file; prohibited-use policy binds downstream |
| Gemma 4 | Documented under [Apache 2.0 page](https://ai.google.dev/gemma/apache_2) (as of 2026-04-01 site) | Prefer SPDX Apache-2.0 path when card confirms; still ship NOTICE |
| TENNs-LLM (ADR-0008 spike) | cc-by-nc-4.0 (local ADR note) | **Non-commercial**—block for production sovereign product media unless legal clears |

**Catalog rule:** `redistributable_on_media=false` or missing license block → artifact may be **operator-imported** under their own compliance workflow but must not ship on default GZMO media. NC and custom community licenses need explicit operator accept flags in CM.

**Memory / context estimates (engineering, not vendor marketing):**

- Weights ≈ file size on disk for GGUF (single-file deployment per [GGUF spec](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md)).  
- KV cache scales with layers × KV heads × head_dim × context × element size × batch; catalog should store `kv_cache_bytes_per_token_estimate` measured or derived from GGUF metadata, not a vibe.  
- llama-server `--fit` / `--fit-target` can adjust layers/context to device memory at runtime—but **qualification context** is the one that passed gates, not the maximum the card advertises (e.g. Qwen2.5 “128k” with YaRN is optional config, not free).  
- Always reserve headroom: OS + sidecars + embed/rerank concurrent with Prime.

### 4. Task roles: separation and shared-model rules

| Role | Function in Living Database | Preferred model class | May share weights with | Must independently qualify |
|------|----------------------------|----------------------|------------------------|----------------------------|
| **embed** | Dense (and optional sparse) vectors for honeypot/corpus; dim must match vault index | Encoder / embedding GGUF or ONNX (e.g. BGE-M3, dim 1024, seq up to 8192) | — | Yes: dim, normalization, language coverage, recall@k on fixed probe set |
| **rerank** | Cross-encoder scores over retrieve candidates | Reranker checkpoint (e.g. bge-reranker-v2-m3); llama-server `--rerank` | Not the generative Prime | Yes: nDCG/MRR on fixed pairs; latency budget |
| **extract_verify** | Distill / librarian extract / honeypot verify / promote gates | Instruction LLM with strong JSON/schema fidelity | May share with conversational **only if** extract_verify gate still passes on that quant | Yes: faithfulness + verify_pass_rate floors (non-compensable) |
| **conversational** | Operator chat, MCP agent assist | Instruction LLM | May share extract_verify weights | Smoke + tool-use templates; lower bar than extract |
| **code_candidate** | Shadow code / schema patch proposals (never auto-promote) | Code-specialized or strong general LLM | May share conversational weights | Yes: compile/test harness on fixed tasks; separate from faithfulness floor |

**Hard separations:**

1. **embed ≠ generative Prime.** Different objective, often different architecture; mixing destroys vector space stability. Dimension is a **contract** with the vault (GZMO today: 1024-d path in living docs/ADR sketches). Changing embed model requires re-index + signed capability change.  
2. **rerank ≠ embed.** Cross-encoder vs bi-encoder; BGE docs explicitly recommend hybrid retrieve then rerank.  
3. **extract_verify is the faithfulness floor owner.** A model allowed for chat must not silently become the overnight extract engine without gate evidence.  
4. **code_candidate** outputs are untrusted candidates under constitution (operator-signed promotion). Qualifying for chat does not qualify for code.  
5. Optional: small **draft** model for speculative decoding stays an accelerator for an already-qualified target, not a separate cognitive authority.

**Process isolation:** prefer separate llama-server (or ORT) processes per role with localhost ports, matching current Prime/embed split—reduces blast radius and allows independent lifecycle. Co-hosting embed+rerank on one small GPU process is acceptable if CM VRAM math includes both and roles stay separately addressed.

### 5. Selection approaches compared

| Approach | Mechanism | Pros | Cons | Fit for GZMO |
|----------|-----------|------|------|--------------|
| **A. Static profile** | HIR → profile tier → predeclared model IDs (today’s `boot.sh` VRAM ladders) | Simple, deterministic, offline | Ignores actual free VRAM after sidecars; couples to specific filenames; no role gates; silent wrong model if file missing | Acceptable as **first filter** only |
| **B. Measured cold-start qualification** | Candidate from catalog → load → warmup → run role suite → persist `QualificationRecord` | Evidence-backed; catches OOM, wrong arch, quality cliffs | Longer first boot; needs suite fixtures on media | **Required** before enabling overnight writer / claiming profile |
| **C. Continuous energy/quality routing** | Online pick among already-qualified models using RAPL/NVML joules × quality | Optimizes usefulness after floors | Needs calibrated meters; risk of thrashing; must not bypass faithfulness | **Optional Phase-2** inside operator envelope; measure-first (Obolus doctrine: joules observability before gates) |

**Recommended composition:**

```text
HIR  →  CM (backends + budgets + explicit unavailables)
     →  Catalog filter (license ∩ backend ∩ RAM/VRAM ∩ role ∩ min_profile)
     →  Rank (prefer higher quality_baseline within budget)
     →  Cold-start qualify winner per role
     →  Pin SelectionRecord (signed/local attest)
     →  Runtime serve
     →  (optional) continuous router only among pins that already passed B
```

**Fail-closed rules:**

- No qualified model for `extract_verify` → **do not** enable overnight distill/promote; health = FAIL/WARN; never open cloud.  
- Hash/signature fail → refuse load.  
- Backend required by entry but probe `unavailable` → skip entry (do not force CPU path unless entry lists `cpu`).  
- Embed down → FTS-only recall, hold vector sync; do not claim hybrid GREEN (`docs/AIRGAP_LIVING.md`).  
- Qualification score below floor → try `degrade_peer`; if none, role stays unavailable.  
- Resource headroom below floor mid-run → pause heavy jobs; do not silently quant-swap to an unqualified file.

**Honest-degrade vocabulary (status surface):**

| CM flag | Meaning | User-visible claim |
|---------|---------|-------------------|
| `full` | All required roles qualified at profile | May claim profile name |
| `degraded.no_embed` | Generative OK, vectors off | “FTS-only recall” |
| `degraded.no_rerank` | Retrieve without cross-encoder | “Retrieve without rerank” |
| `degraded.cpu_only` | GPU backends unavailable | “CPU inference; reduced throughput/context” |
| `degraded.context_capped` | Running below card max, at qualified ctx | State actual ctx |
| `unavailable.extract` | No faithful extract engine | Overnight writer blocked |
| `incomplete.install` | Missing sidecars/models | Never “living GREEN” |

### 6. Stable interface: boot → manifest → selection

```text
┌──────────────┐   HIR    ┌──────────────────┐   CM     ┌────────────────────┐
│ BootDiscovery│ ───────► │ CapabilityCompiler│ ───────► │ ModelSelector      │
│ (read-only)  │          │ (pure function +  │          │ (catalog ∩ CM ∩    │
└──────────────┘          │  binary probes)   │          │  role policies)    │
                          └──────────────────┘          └─────────┬──────────┘
                                                                  │ SelectionPlan
                                                                  ▼
                                                        ┌────────────────────┐
                                                        │ Qualifier (cold)   │
                                                        │ → QualificationRec │
                                                        └─────────┬──────────┘
                                                                  │ pins
                                                                  ▼
                                                        ┌────────────────────┐
                                                        │ RuntimeSupervisor  │
                                                        │ llama-server roles │
                                                        │ localhost only     │
                                                        └────────────────────┘
```

**Interface contracts (versioned):**

1. `BootDiscovery.collect() -> HIR`  
2. `CapabilityCompiler.compile(HIR, appliance_policy) -> CM`  
3. `ModelSelector.select(CM, Catalog, role_set) -> SelectionPlan | InsufficientCapability`  
4. `Qualifier.run(plan, suites) -> QualificationRecord`  
5. `RuntimeSupervisor.apply(record) -> ServingEndpoints` (`127.0.0.1` ports per role)  
6. `Health.expose()` includes CM unavailables + qualification digests + degrade flags (never aspirational).

**CM minimum fields:** `profile_claim`, `backends{}`, `ram_budget_mib`, `vram_budget_mib`, `max_context_tokens`, `roles_enabled[]`, `roles_unavailable[]`, `energy_meters`, `signature_of_policy_envelope`.

**SelectionPlan fields:** per-role `artifact_id`, engine argv (`-m`, `-ngl`, `-c`, `--embedding`/`--rerank`), port, degrade_peer chain.

This is the contract issue 09 will grill; this research freezes the **evidence-backed shape**, not the final numeric ladder (issue 07/01).

## Options and tradeoffs

| Option | Summary | Tradeoff |
|--------|---------|----------|
| **O1. llama.cpp-only spine** | One engine family, multi-backend binaries on media | Best airgap story; GGUF ecosystem; embed/rerank supported. Weaker if a role needs non-GGUF-only arch (custom `trust_remote_code`) |
| **O2. llama.cpp + ORT side lane** | ORT for embed/rerank ONNX, llama for LLM | Better mobile/NPU EPs; more moving parts and EP matrix testing |
| **O3. Vendor edge stacks (TensorRT, Jetson-only)** | Max perf on specific NVIDIA SKUs | Fragmented media; Jetson ≠ desktop TRT-LLM assumptions; higher supply-chain complexity |
| **O4. Static VRAM ladder only** | Ship `boot.sh`-style thresholds | Fast; insufficient honesty for faithfulness floors and licenses |
| **O5. Continuous joule router from day one** | Always optimize energy | Premature without RAPL/NVML calibration; conflicts with “measure never invent” |

**Recommendation for North Star design:** **O1 as default**, **O2 optional** when catalog entries are ONNX and CM enables EP, **O5 only after** qualification pins exist and energy meters are trusted. Reject O4 as sole selector. Treat O3 as reference-node optimization, not portable requirement.

## Constraints for GZMO

1. **One physical node; localhost inference only** for core metabolism (ADR-0004/0007). No VM200-style second machine in the North Star topology.  
2. **Airgap:** catalog and suites on media; llama-server `--offline`; no HF pull at runtime.  
3. **Operator-signed** model/runtime/capability expansion; autonomous memory must not swap Prime weights.  
4. **Non-compensable floors:** faithfulness / verify gates beat throughput and joules.  
5. **ROCm only on official matrix**; else Vulkan or CPU. No `HSA_OVERRIDE_GFX_VERSION` in production.  
6. **License class on every artifact**; default media prefers OSI-friendly (Apache-2.0/MIT) and documented community licenses with NOTICE; NC blocked.  
7. **Role-separated processes and qualification**; embed dim is a breaking contract.  
8. **Honest degrade** strings; never claim living GREEN when extract or required roles fail.  
9. **Energy:** RAPL + NVML integration are observability first (`docs/OBOLUS_ENERGY.md`); continuous routing stays behind calibration.  
10. **Existing scars are evidence:** `boot.sh` binary probe + VRAM ladder; dual Prime/embed ports; degrade table in `AIRGAP_LIVING.md`; keep patterns, replace hardcoded model shopping with signed catalog + qualifier.

## Unknowns

- Exact **release-reference node** SKU and numeric RAM/VRAM floors (depends on tickets 01/07).  
- Whether **Gemma 4** Apache-2.0 applies uniformly to all Gemma 4 weight variants operators might want (confirm each model card at pin time).  
- **TensorRT-LLM on Jetson** support posture for the specific L4T version chosen at freeze—re-verify against NVIDIA docs for that pin; do not assume desktop TRT-LLM.  
- Production **embed** choice if not BGE-M3 (dim, license, GGUF quality).  
- Whether **single GPU** must time-slice Prime vs embed vs rerank or require VRAM enough for concurrent loads.  
- **Windows appliance** path: NVML path differences; RAPL absent; DirectML/ORT may matter more than on Linux reference.  
- Cold-start suite wall-clock budget acceptable to operators on minimum profile.  
- Multi-modal / vision roles—out of scope unless constitution expands.

## Primary sources

### Standards and inventory
- DMTF SMBIOS: https://www.dmtf.org/standards/smbios (DSP0134 3.9.0, 2025-08-19)  
- Linux powercap/RAPL: https://www.kernel.org/doc/Documentation/power/powercap/powercap.txt  
- NVIDIA NVML: https://docs.nvidia.com/deploy/nvml-api/nvml-api-reference.html  
- NVIDIA CUDA GPU CC table: https://developer.nvidia.com/cuda/gpus  
- AMD ROCm compatibility matrix: https://rocm.docs.amd.com/en/latest/compatibility/compatibility-matrix.html  
- Khronos Vulkan Loader architecture: https://github.com/KhronosGroup/Vulkan-Loader/blob/main/docs/LoaderInterfaceArchitecture.md  
- NVIDIA Jetson Linux r36.4 Quick Start (Orin modules): https://docs.nvidia.com/jetson/archives/r36.4/DeveloperGuide/IN/QuickStart.html  

### Runtimes and formats
- llama.cpp backends & goals: https://github.com/ggml-org/llama.cpp/blob/master/README.md  
- llama.cpp build (CUDA/HIP/Vulkan/CPU): https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md  
- llama-server (embeddings, rerank, offline, devices): https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md  
- GGUF specification: https://github.com/ggml-org/ggml/blob/master/docs/gguf.md  
- ONNX Runtime execution providers: https://onnxruntime.ai/docs/execution-providers/  

### Licenses / model cards
- Apache License 2.0: https://www.apache.org/licenses/LICENSE-2.0  
- Qwen2.5-7B-Instruct card (Apache-2.0): https://huggingface.co/Qwen/Qwen2.5-7B-Instruct  
- BAAI/bge-m3 (MIT): https://huggingface.co/BAAI/bge-m3  
- BAAI/bge-reranker-v2-m3 (Apache-2.0): https://huggingface.co/BAAI/bge-reranker-v2-m3  
- Llama 3.1 Community License: https://raw.githubusercontent.com/meta-llama/llama-models/main/models/llama3_1/LICENSE  
- Gemma Terms of Use: https://ai.google.dev/gemma/terms  
- Gemma 4 Apache 2.0 notice page: https://ai.google.dev/gemma/apache_2  

### Local repository evidence
- North Star framing: `.scratch/self-developing-living-database/issues/00-north-star-framing.md`  
- Wayfinder map constraints: `.scratch/self-developing-living-database/map.md`  
- Airgap living topology & degrade table: `docs/AIRGAP_LIVING.md`  
- ADR-0004 / ADR-0007 airgap one-product doctrine: `docs/ADR-0004-airgap-living-usp.md`, `docs/ADR-0007-one-product-living.md`  
- Boot VRAM ladder & CUDA/CPU binary probe: `boot.sh`  
- Energy observability doctrine: `docs/OBOLUS_ENERGY.md`  
- Embed/rerank split in config evidence: `config/gzmo-next.toml`, `docs/ARCHITECTURE_FIX_HANDOFF_2026-07-29.md`  
- Model license caution (TENNs NC): `docs/ADR-0008-edge-ssm-memory.md`  
