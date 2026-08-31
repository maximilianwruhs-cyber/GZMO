# Edge hardware Pareto and reference node (as of 2026-08-31)

## Executive finding

For a **one physical node, runtime air-gapped** Self-developing Living Database, the honest purchasable Pareto frontier is dominated by **unified-memory capacity and bandwidth**, not advertised TOPS/TFLOPS. Sparse INT8/FP4 TOPS do **not** equal usable LLM extract/verify/code-candidate performance.

Four classes remain on the frontier with verified primary sources:

| Class | Representative purchasable SKUs | Dominant strength | Dominant weakness for GZMO |
| --- | --- | --- | --- |
| Low-power ARM/Pi-class + accelerator | Raspberry Pi 5 16GB + AI HAT+ 26 TOPS; Jetson Orin Nano Super Dev Kit | Cost, idle power, Pi production horizon to ≥2036 | 8–16 GB memory; Hailo vision/NPU-centric; weak concurrent extract+search |
| NVIDIA unified-memory edge | Jetson AGX Orin 64GB Dev Kit; Jetson AGX Thor Dev Kit (T5000); DGX Spark (GB10) | CUDA/JetPack maturity; large coherent/unified memory | Dev-kit vs production packaging; price; Arm ISA; Spark stock gaps |
| x86-64 integrated acceleration | Intel Core Ultra Series 2 (Lunar Lake V); AMD Ryzen AI Max+ 395 (Strix Halo) systems | x86 path; Strix Halo up to 128 GB LPDDR5x + wide iGPU | Lunar Lake max 32 GB; NPU TOPS oversell LLM roles; OEM variance |
| x86-64 + one discrete GPU | Workstation/DIY with GeForce RTX 5090 (32 GB GDDR7) | Peak discrete VRAM for forge isolation; mature CUDA on x86 | High wall power/noise; host RAM vs VRAM split; no ECC on GeForce |

**Defensible ladder seeds for ticket 07 (not a purchase decision):**

- **Minimum:** Jetson Orin Nano Super Dev Kit **or** Pi 5 16GB + AI HAT+ 26 TOPS + NVMe — storage/search + small embeddings; declare extract/verify/code roles degraded.
- **Release-reference:** **DGX Spark** (128 GB coherent unified) **or** **Jetson AGX Orin 64GB** (edge watts) **or** **Strix Halo 96–128 GB** complete system (x86). Choose on ISA, stock, and runtime qualification — not TOPS tables.
- **Forge:** **Jetson AGX Thor Dev Kit** (128 GB, 40–130 W) **and/or** **x86 + single RTX 5090 32 GB** for isolated candidate-evaluation headroom.

No SKU is the winner. Ticket 07 owns final ladder naming.

---

## Decision-relevant facts

### Workload translation (GZMO roles)

| Role | What bounds it | Implication |
| --- | --- | --- |
| **Storage / search** | CPU, DRAM, NVMe lanes + endurance | All classes can host hybrid retrieval if RAM covers working set; prefer NVMe over microSD. |
| **Embeddings / rerank** | Modest accelerator or CPU; bandwidth | Hailo, Intel NPU, Orin GPU/DLA, iGPU, dGPU can serve *small* embedding models; software packaging differs. |
| **Extraction / verification** | Weights + KV cache in fast memory; context | Needs tens of GB. Local horizon already flags ~32 GB VRAM for strong long-context tries (`research/opportunities/local-intel-32gb-128k.md`). Not on 8 GB Nano or Pi+Hailo alone. |
| **Local code-candidate evaluation** | Same + isolation headroom | Forge wants 64–128 GB unified **or** ≥24–32 GB discrete VRAM **plus** host RAM for the living DB writer. |

**Constitutional floors:** one physical node; runtime air gap; local containers OK; adaptive capability ladder with declared unavailable capabilities; operator-signed code/schema/model/security/capability expansion; non-compensable faithfulness/sovereignty/reliability/resource/audit/rollback floors ([map](../../.scratch/self-developing-living-database/map.md)).

---

### Class A — Low-power ARM / Pi-class + accelerators

#### A1. Raspberry Pi 5 + AI HAT+

| Attribute | Verified fact | Source |
| --- | --- | --- |
| SKU / list price | Pi 5 **16GB $305**; 8GB $175; 4GB $110 | [Pi 5](https://www.raspberrypi.com/products/raspberry-pi-5/), [brief Apr 2026](https://pip.raspberrypi.com/documents/RP-008348-DS-raspberry-pi-5-product-brief.pdf) |
| CPU ISA | BCM2712, **4× Cortex-A76 @ 2.4 GHz** | Same |
| RAM | LPDDR4X-4267 up to **16 GB** (no ECC on Pi 5 board) | Same |
| Storage | microSD SDR104; **PCIe 2.0 x1** via M.2 HAT for NVMe | Same |
| Power | **5 V / 5 A** USB-C PD recommended | Same |
| Thermals | Active cooling recommended under load | Same |
| Horizon | Production until **≥ January 2036** | Same |
| AI HAT+ | Hailo-8 **26 TOPS** **$110**; Hailo-8L **13 TOPS** **$70**; 0–50 °C; production **≥ Jan 2030** | [AI HAT+](https://www.raspberrypi.com/products/ai-hat/), [HAT brief](https://datasheets.raspberrypi.com/ai-hat-plus/raspberry-pi-ai-hat-plus-product-brief.pdf) |
| Hailo-8 | **26 TOPS**; vendor **typical ~2.5 W**; on-chip memory; TF/PyTorch/ONNX | [Hailo-8](https://hailo.ai/products/ai-accelerators/hailo-8-ai-accelerator/) |
| Secure boot / TPM | No discrete TPM claimed on Pi 5 board product pages | Pi product pages |
| Complete-node $ | Boards only ~**$415** (16GB+26 TOPS HAT); full node (PSU/cooler/NVMe/case) **no single official MSRP** | List prices only |

**GZMO fit:** Credible **minimum** for storage/search and small embeddings. **Disqualified as sole full-product host** for strong extract/verify/code-eval without declaring those capabilities unavailable.

#### A2. Raspberry Pi Compute Module 5

| Attribute | Verified fact | Source |
| --- | --- | --- |
| Module | BCM2712 @ 2.4 GHz; **2/4/8/16 GB LPDDR4-4267 with ECC**; optional eMMC; PCIe Gen2 x1; from **$67.50** | [CM5](https://www.raspberrypi.com/products/compute-module-5/) |
| Horizon | Production until **≥ Jan 2036** | Same |
| Form | Requires carrier; not a complete node alone | Same |

**GZMO fit:** Industrial/repairable Pi-class embedding; still LLM memory-limited.

#### A3. NVIDIA Jetson Orin Nano Super Developer Kit

| Attribute | Verified fact | Source |
| --- | --- | --- |
| AI (marketing) | **67 INT8 TOPS** sparse (Super boost from prior 40) | [Nano Super](https://www.nvidia.com/en-us/autonomous-machines/embedded-systems/jetson-orin/nano-super-developer-kit/) |
| GPU / CPU | Ampere **1024 CUDA / 32 Tensor**; **6× Cortex-A78AE** | Same; [Orin](https://www.nvidia.com/en-us/autonomous-machines/embedded-systems/jetson-orin/) |
| Memory | **8 GB** 128-bit LPDDR5, **102 GB/s** | Same |
| Power | **7–25 W** | Same |
| Storage | microSD + external NVMe | Same |
| Official kit price | NVIDIA states **USD $249** | [Buy Jetson](https://developer.nvidia.com/embedded/buy-jetson) |
| Channel kit | Seeed bundle **~$449**, in stock (2026-08-31 observation) | [Seeed Nano Super](https://www.seeedstudio.com/NVIDIAr-Jetson-Orintm-Nano-Super-Developer-Kit-Bundle.html) |
| Security | Jetson Linux: Secure Boot, OP-TEE, LUKS, **firmware TPM**, rollback protection | [Jetson Security r36.4](https://docs.nvidia.com/jetson/archives/r36.4/DeveloperGuide/SD/Security.html) |
| ECC | Not AGX Orin Industrial ECC path | Orin family table |

**GZMO fit:** Best **CUDA-native minimum**. Still **8 GB unified** — embeddings yes; concurrent living-DB + strong extract **no** without degradation.

---

### Class B — NVIDIA unified-memory edge systems

#### B1. Jetson AGX Orin 64GB Developer Kit / module

| Attribute | Verified fact | Source |
| --- | --- | --- |
| AI | Up to **275 sparse INT8 TOPS** (GPU+DLA; see dense/sparse breakdown on Orin page) | [Orin specs](https://www.nvidia.com/en-us/autonomous-machines/embedded-systems/jetson-orin/) |
| GPU / CPU | **2048-core Ampere + 64 Tensor**; **12× Cortex-A78AE @ ≤2.2 GHz** | Same |
| Memory | **64 GB** 256-bit LPDDR5, **204.8 GB/s** unified | Same |
| Module power | **15–60 W** (Industrial 15–75 W, ECC memory) | Same |
| Channel price | Seeed AGX Orin 64GB Dev Kit bundle **~$3,499**, in stock (session) | [Seeed AGX Orin](https://www.seeedstudio.com/NVIDIAr-Jetson-AGX-Orintm-64GB-Developer-Kit-Bundle.html) |
| Security | Jetson secure boot / fTPM / LUKS | Jetson Security guide |
| Production note | Dev kits are for **development/prototyping**; production uses modules + carrier | [Buy Jetson FAQ note](https://developer.nvidia.com/embedded/buy-jetson) |

**GZMO fit:** Strong **reference-class** seed: 64 GB unified at edge power. Arm + Jetson Linux qualification required. **Module distributor MSRP not verified** this session (Arrow timeouts).

#### B2. Jetson AGX Thor Developer Kit (T5000)

| Attribute | Verified fact | Source |
| --- | --- | --- |
| AI | Up to **2070 TFLOPS FP4 sparse** (130 W measured claim) | [Jetson Thor](https://www.nvidia.com/en-us/autonomous-machines/embedded-systems/jetson-thor/) |
| GPU | **2560-core Blackwell**, 5th-gen Tensor, MIG | Same |
| CPU | **14-core Arm Neoverse-V3AE**, ≤2.6 GHz | Same |
| Memory | **128 GB** 256-bit LPDDR5X, **273 GB/s** | Same |
| Power | **40–130 W** | Same |
| Dev kit storage | **1 TB NVMe** M.2 on kit | Same |
| Networking | Kit: 5 GbE + QSFP28 (4×25 GbE) | Same |
| Channel price | Seeed Thor Dev Kit bundle **$5,499**, **In Stock** (session) | [Seeed Thor](https://www.seeedstudio.com/NVIDIAr-Jetson-AGX-Thortm-Developer-Kit-Bundle.html) |
| T4000 | **64 GB**, **1200 FP4 TFLOPS sparse**, **40–70 W** | Thor page |

**GZMO fit:** **Forge or high reference** on edge power with 128 GB unified. Do not treat 2070 FP4 TFLOPS as LLM speed. Thor-specific secure-boot fuse docs should be re-read on selected JetPack at flash time (Orin r36.4 guide confirmed; Thor parity not re-verified line-by-line).

#### B3. NVIDIA DGX Spark (GB10)

| Attribute | Verified fact | Source |
| --- | --- | --- |
| Architecture | GB10 Grace Blackwell; coherent CPU+GPU memory (NVLink-C2C) | [DGX Spark](https://www.nvidia.com/en-us/products/workstations/dgx-spark/), [datasheet](https://dam-cdn.nvd.orangelogic.com/AssetLink/3lhuar5pc56pn7se4c7ahsskw20xw8h5.pdf) |
| AI | Up to **1 PFLOP FP4** theoretical with sparsity | Same |
| CPU | **20-core Arm** (10× X925 + 10× A725) | Same |
| Memory | **128 GB LPDDR5x coherent unified**, 256-bit, **273 GB/s** | Same |
| Storage | **4 TB NVMe M.2 self-encrypting** | Same |
| Power | **240 W** system consumption; **GB10 TDP 140 W** | Same |
| Noise | Idle **LWA,m 19 dB** / operating **35 dB** (ECMA-109, June 2025 method on page) | Product page |
| Form | 150×150×50.5 mm, 1.2 kg | Same |
| OS | DGX OS; NVIDIA AI stack | Same |
| Vendor model claims | Marketing: inference up to ~200B params FP4; fine-tune ~70B — **vendor estimates**, not GZMO benchmarks | Datasheet |
| Channel price | Seeed **$3,999**; **Out of stock** (session) | [Seeed DGX Spark](https://www.seeedstudio.com/NVIDIA-DGX-Spark-p-6611.html) |

**GZMO fit:** Strong **release-reference seed**: complete node, large unified memory, quiet desktop, SED bulk storage. Constraints: Arm + DGX OS, 240 W wall, intermittent stock. Dual-Spark networking is multi-box (out of topology if required).

---

### Class C — x86-64 with integrated acceleration

#### C1. Intel Core Ultra Series 2 — Lunar Lake V (example: Ultra 7 258V)

| Attribute | Verified fact | Source |
| --- | --- | --- |
| CPU | 8 cores (4P+4LPE), ≤**4.8 GHz**; base **17 W**, turbo **37 W** | [ARK 258V](https://www.intel.com/content/www/us/en/products/sku/240957/intel-core-ultra-7-processor-258v-12m-cache-up-to-4-80-ghz/specifications.html) |
| Memory ceiling | **Max 32 GB** LPDDR5X-8533 | Same |
| NPU | **47 TOPS INT8** | Same |
| iGPU | Arc 140V **64 TOPS INT8**; platform peak **115 TOPS INT8** | Same |
| PCIe | Up to **8 lanes** (5.0/4.0 mix) | Same |
| Runtimes | OpenVINO, ONNX RT, DirectML, Windows ML | Same |

**GZMO fit:** Efficient x86 minimum/reference-lite if ≤32 GB total platform memory is accepted. **Memory ceiling disqualifies** “32 GB VRAM-class” extract horizons without a discrete GPU (then Class D). Complete OEM system prices **not pinned**.

#### C2. AMD Ryzen AI Max+ 395 (Strix Halo)

| Attribute | Verified fact | Source |
| --- | --- | --- |
| CPU | **16× Zen 5**, 32 threads; boost **5.1 GHz** | [Max+ 395](https://www.amd.com/en/products/processors/laptop/ryzen/ai-300-series/amd-ryzen-ai-max-plus-395.html) |
| TDP | Default **55 W**; **cTDP 45–120 W** | Same |
| Memory | **256-bit LPDDR5x**, **max 128 GB**, LPDDR5x-8000 | Same |
| Graphics | **Radeon 8060S**, **40** graphics cores, ≤2900 MHz | Same |
| AI marketing | **≤50 NPU TOPS**; **≤126 overall TOPS** | Same |
| I/O | USB4×2; PCIe 4.0, 16 usable lanes | Same |
| OS | Windows 11, RHEL, Ubuntu x86_64 | Same |

**GZMO fit:** Leading **x86 integrated** contender: up to **128 GB** system memory shared with wide iGPU. ROCm/Vulkan/llama.cpp maturity must be qualified in ticket 03. Complete-node price **OEM-dependent**; named mini-PC configure pages not verified this session (fetch failures).

---

### Class D — x86-64 + one discrete GPU

#### D1. GeForce RTX 5090 in a single-GPU workstation

| Attribute | Verified fact | Source |
| --- | --- | --- |
| Memory | **32 GB GDDR7**, 512-bit | [RTX 5090](https://www.nvidia.com/en-us/geforce/graphics-cards/50-series/rtx-5090/) |
| Compute | **21760** CUDA; 5th-gen Tensor (**3352 AI TOPS** marketing); Blackwell | Same |
| Board power | **TGP 575 W**; recommended system power **1000 W** (config-dependent) | Same |
| Form | ~304 mm length class dual-slot FE design | Same |
| ECC | **GeForce is not ECC workstation memory** | Product class |
| MSRP | NVIDIA page placeholder “Starting at $XXX” in fetched content — **street price channel-variable; MSRP not verified here** | Same |
| Software | CUDA ecosystem on x86 | NVIDIA developer stack |

**GZMO fit:** **Forge-class** discrete memory for isolated eval. Host still needs separate system RAM for DB writer. Wall power/noise dominate. Workstation ECC GPU alternative **not pinned** (workstation GPU hub 404 this session).

---

## Options and trade-offs

### Comparison matrix

| Candidate | ISA | Fast memory | Mem BW | Node power (order) | Accelerator SW | Secure boot / TPM | ECC | Repairability | Complete $ (verified) | Stock (session) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Pi 5 16G + AI HAT+ 26 | Arm A76 | 16 GB LPDDR4X | low vs Jetson | ~board+HAT 5–25 W class | Hailo RT | Weak appliance TPM | No (Pi 5) / CM5 RAM ECC | Excellent | Boards ~$415; node DIY | Good |
| Orin Nano Super kit | Arm A78AE | **8 GB** LPDDR5 | 102 GB/s | 7–25 W | JetPack/CUDA | Jetson SB + fTPM | No | Module/kit | NVIDIA **$249** claim; Seeed ~$449 | Good |
| AGX Orin 64G kit | Arm A78AE | **64 GB** LPDDR5 | 204.8 GB/s | 15–60 W module | JetPack/CUDA | Same | Industrial SKU yes | Module ecosystem | Seeed ~**$3499** | Good |
| AGX Thor kit T5000 | Arm V3AE | **128 GB** LPDDR5X | 273 GB/s | 40–130 W | JetPack/CUDA Blackwell | Jetson security lineage | Not consumer-ECC marketed | Dev kit | Seeed **$5499** | In stock |
| DGX Spark | Arm GB10 | **128 GB** coherent | 273 GB/s | **240 W** wall | DGX OS + NVIDIA AI | SED NVMe | Not GeForce ECC claim | Sealed mini | Seeed **$3999** | **OOS** |
| Intel Ultra 7 258V system | x86 | **≤32 GB** LPDDR5X | platform | ~8–37 W CPU | OpenVINO/NPU/iGPU | OEM TPM typical | Rare on client | OEM-dependent | **Unavailable** here | Broad |
| Ryzen AI Max+ 395 system | x86 | **≤128 GB** LPDDR5x | 256-bit | 45–120 W cTDP | ROCm/Vulkan — qualify | OEM TPM typical | Platform-dependent | DIY/Framework-class better | **OEM unavailable** here | Growing |
| x86 + RTX 5090 | x86 + dGPU | **32 GB GDDR7** + host RAM | GDDR7 high; host separate | **~575 W GPU** + host | CUDA mature | OEM TPM | GeForce no ECC | High (desktop) | GPU MSRP **unverified** | Channel |

### Trade-off axes (ticket 07)

1. **Memory vs watts:** Orin 64G / Thor / Spark / Strix Halo win “model fits”; Pi/Nano/Lunar Lake win joules and cost.
2. **ISA & runtime:** CUDA-on-Arm vs CUDA-on-x86 vs OpenVINO/ROCm — ticket 03 freezes contracts per profile.
3. **Complete appliance vs kit:** DGX Spark is complete; Jetson kits need carriers for field; Pi is DIY; x86 mini PCs vary.
4. **Reliability:** Prefer ECC (CM5 RAM, AGX Orin Industrial, workstation ECC GPU if selected) for non-compensable reliability floor.
5. **Sovereignty / air-gap:** No cloud license check at runtime; offline JetPack/DGX OS and open weights (ticket 03).
6. **Noise:** DGX Spark publishes low acoustics; 5090 workstations need mitigation for shared spaces.

### Disqualifiers

| Item | Why |
| --- | --- |
| ≤8 GB unified as sole full-product node | Cannot host living DB + strong extract/verify concurrently |
| Hailo-only as faithfulness path | Vision/NPU product; not LLM memory substitute |
| Lunar Lake without dGPU as “32 GB VRAM horizon” host | 32 GB is entire system, shared with OS and DB |
| Multi-node Spark pair as topology requirement | Violates one-physical-node constitution |
| Cloud-bound AI PC stacks | Breaks runtime air-gap |
| Equating sparse FP4 PFLOPS/TOPS to tokens/s | Misleading for operator decisions |
| Announced-only / unpriced SKUs as release gates | Not verified purchasable complete nodes here |

### Defensible Pareto set (seeds only)

| Profile | Seed A | Seed B | Degraded if chosen poorly |
| --- | --- | --- | --- |
| **Minimum** | Orin Nano Super + NVMe | Pi 5 16GB + AI HAT+ 26 + NVMe | Strong extract, long-context verify, heavy code-eval |
| **Release reference** | DGX Spark 128 GB | AGX Orin 64 GB **or** Strix Halo 96–128 GB | CUDA-only vs ROCm-only packs; Arm vs x86 ops skill; Spark stock |
| **Forge** | Thor 128 GB edge | x86 + RTX 5090 32 GB (or 24 GB class if new 5090 unavailable) | Power/noise; GeForce non-ECC |

Final naming and “what every release tests” belong to **ticket 07**.

---

## Constraints for GZMO

1. **One node:** single chassis/module; no required companion box.
2. **Air gap:** boot and run models/storage with no phone-home; verify OEM AI features offline.
3. **Capability ladder:** minimum may omit extract/code-eval; reference declares model classes; forge adds isolation headroom.
4. **Signed expansion:** larger GPU/module swap is operator-signed capability expansion.
5. **Resource floors:** ticket 07 sets RAM/SSD endurance/power caps so the living writer is not starved by models.
6. **Storage endurance:** pin NVMe TBW for audit ledger + vector rebuild; microSD is not a durability floor.
7. **Repairability:** Pi and desktop x86 high; soldered LPDDR and sealed Spark/Thor lower — document spares.
8. **Prices:** Seeed figures are **2026-08-31 channel observations**; NVIDIA/Pi list prices are vendor lists; they move. No purchase recommendation.

---

## Unknowns

- Arrow/distributor **module MSRPs** for Thor T5000/T4000 and AGX Orin (pages timed out).
- **DGX Spark** restock cadence and fully offline update story.
- **Thor** secure-boot fuse doc parity vs Orin r36.4 (not line-audited).
- **Strix Halo** end-to-end local LLM rates under 96–128 GB — no benchmarks (forbidden).
- Named **Strix Halo complete OEM SKUs** and prices (configure pages failed).
- **RTX professional ECC** forge alternative SKU and MSRP.
- **Idle watts** for complete assembled nodes.
- Industrial Jetson carriers with sealed TPM/watchdog for 24/7 air-gap field units.
- Exact NVMe TBW BOM operators will buy.

---

## Primary sources

### NVIDIA
- [Jetson Thor](https://www.nvidia.com/en-us/autonomous-machines/embedded-systems/jetson-thor/)
- [Jetson Orin specs](https://www.nvidia.com/en-us/autonomous-machines/embedded-systems/jetson-orin/)
- [Orin Nano Super](https://www.nvidia.com/en-us/autonomous-machines/embedded-systems/jetson-orin/nano-super-developer-kit/)
- [Jetson modules](https://developer.nvidia.com/embedded/jetson-modules)
- [Buy Jetson](https://developer.nvidia.com/embedded/buy-jetson)
- [DGX Spark](https://www.nvidia.com/en-us/products/workstations/dgx-spark/)
- [DGX Spark datasheet](https://dam-cdn.nvd.orangelogic.com/AssetLink/3lhuar5pc56pn7se4c7ahsskw20xw8h5.pdf)
- [RTX 5090](https://www.nvidia.com/en-us/geforce/graphics-cards/50-series/rtx-5090/)
- [Jetson Linux Security r36.4](https://docs.nvidia.com/jetson/archives/r36.4/DeveloperGuide/SD/Security.html)
- [Thor Dev Kit user guide](https://docs.nvidia.com/jetson/agx-thor-devkit/user-guide/latest/index.html)

### Channel observations (2026-08-31; not MSRP authority)
- [Seeed Thor Dev Kit](https://www.seeedstudio.com/NVIDIAr-Jetson-AGX-Thortm-Developer-Kit-Bundle.html) — $5,499, in stock
- [Seeed DGX Spark](https://www.seeedstudio.com/NVIDIA-DGX-Spark-p-6611.html) — $3,999, OOS
- [Seeed AGX Orin 64GB kit](https://www.seeedstudio.com/NVIDIAr-Jetson-AGX-Orintm-64GB-Developer-Kit-Bundle.html) — ~$3,499
- [Seeed Orin Nano Super](https://www.seeedstudio.com/NVIDIAr-Jetson-Orintm-Nano-Super-Developer-Kit-Bundle.html) — ~$449

### Raspberry Pi / Hailo
- [Pi 5](https://www.raspberrypi.com/products/raspberry-pi-5/)
- [Pi 5 brief](https://pip.raspberrypi.com/documents/RP-008348-DS-raspberry-pi-5-product-brief.pdf)
- [AI HAT+](https://www.raspberrypi.com/products/ai-hat/)
- [AI HAT+ brief](https://datasheets.raspberrypi.com/ai-hat-plus/raspberry-pi-ai-hat-plus-product-brief.pdf)
- [CM5](https://www.raspberrypi.com/products/compute-module-5/)
- [Hailo-8](https://hailo.ai/products/ai-accelerators/hailo-8-ai-accelerator/)

### Intel / AMD
- [Core Ultra hub](https://www.intel.com/content/www/us/en/products/details/processors/core-ultra.html)
- [Ultra 7 258V ARK](https://www.intel.com/content/www/us/en/products/sku/240957/intel-core-ultra-7-processor-258v-12m-cache-up-to-4-80-ghz/specifications.html)
- [Ryzen AI Max+ 395](https://www.amd.com/en/products/processors/laptop/ryzen/ai-300-series/amd-ryzen-ai-max-plus-395.html)

### Local
- [North Star map](../../.scratch/self-developing-living-database/map.md)
- [32GB–128k horizon](../opportunities/local-intel-32gb-128k.md)
- Consumer: `.scratch/self-developing-living-database/issues/07-choose-hardware-ladder.md`

---

*Research date: 2026-08-31. No hardware purchased, no models downloaded, no benchmarks executed.*
