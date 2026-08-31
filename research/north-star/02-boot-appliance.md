# 02 — Boot media, immutable appliance, and persistence

**Research date:** 2026-08-31  
**Ticket:** `.scratch/self-developing-living-database/issues/02-boot-appliance.md`  
**Scope:** Production-grade designs to boot or deploy GZMO from portable media across ARM64/x86-64 edge hardware while preserving airgap, durable state, secure recovery, and unattended operation. No image build.

## Executive finding

**One physical portable medium can carry both ARM64 and x86-64 boot paths, but one PE/UKI binary cannot.** UEFI loads architecture-specific PE images (`Machine` = AMD64 `0x8664` or ARM64 `0xaa64`). systemd ships distinct stubs (`linuxx64.efi.stub`, `linuxaa64.efi.stub`). A FAT ESP can hold both `BOOTX64.EFI` / `BOOTAA64.EFI` and dual UKIs; firmware executes only the native binary. That is dual packaging, not a single cross-arch image.

**For GZMO’s one-node airgap living appliance, the production-shaped pattern is:** signed, immutable OS (UKI + dm-verity or bootc/ostree composefs) installed to **internal NVMe**, with **LUKS2** data/model/candidate volumes unlocked by **TPM2 PCR policy** (plus recovery key), **A/B or multi-slot** OS updates with automatic boot assessment, and portable media used as **installer / recovery / offline signed-bundle carrier**—not as the sole durable runtime home for large model corpora unless the operator deliberately chooses “everything-on-portable-NVMe.”

**Explicit rejects:**
- Arbitrary hot-plug autorun on an untrusted host without a preinstalled trusted agent (map out-of-scope; supply-chain and privilege escalation risk).
- Claiming a single PE/kernel image “just works” on both arches or on every ARM board (DTB/firmware/U-Boot variance).
- Network-required day-2 update defaults (bootc/Mender/Ubuntu Core store paths) without an offline bundle path.

Research narrows architecture options for ticket 08; it does not choose the final operator-owned layout.

## Decision-relevant facts

### 1. Firmware and multi-architecture realities

| Fact | Source |
|------|--------|
| PE COFF `Machine` identifies target CPU; image runs only on that machine or an emulator. AMD64 = `0x8664`, ARM64 LE = `0xaa64`. | [PE Format — Machine Types](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format) |
| UKI = single UEFI PE combining stub + `.linux` + optional initrd/cmdline/DTB/SBAT/PCR sigs; signed as one Secure Boot object. | [UAPI.5 Unified Kernel Image](https://uapi-group.org/specifications/specs/unified_kernel_image/) |
| systemd-stub is shipped **per architecture**: `linuxx64.efi.stub`, `linuxia32.efi.stub`, `linuxaa64.efi.stub`. | [systemd-stub(7)](https://www.freedesktop.org/software/systemd/man/latest/systemd-stub.html) |
| PE Addons must match local CPU; non-native `Machine` candidates are skipped. | [UAPI.5 PE Addons](https://uapi-group.org/specifications/specs/unified_kernel_image/) |
| Boot Loader Spec Type #1/`architecture` key and Type #2 UKIs: loaders **ignore non-matching arch** entries so one `$BOOT` can list multi-arch entries. | [UAPI.1 Boot Loader Specification](https://uapi-group.org/specifications/specs/boot_loader_specification/) |
| UKI may embed `.dtb` / multiple `.dtbauto` matched by firmware DT compatible or SMBIOS HWIDs—critical for diverse ARM boards, largely irrelevant on generic x86 ACPI. | [systemd-stub](https://www.freedesktop.org/software/systemd/man/latest/systemd-stub.html), [UAPI.5](https://uapi-group.org/specifications/specs/unified_kernel_image/) |
| ESP is typically VFAT; BLS recommends `$BOOT` as XBOOTLDR or ESP; Type #2 UKIs live under `/EFI/Linux/`. | [UAPI.1](https://uapi-group.org/specifications/specs/boot_loader_specification/) |
| Many ARM edge boards still boot via **U-Boot** (with or without UEFI), board-specific device trees, and vendor Secure Boot—not a uniform UEFI+TPM story. Production mechanisms that assume full UEFI+TPM (Ubuntu Core FDE, systemd PCR binding) **do not automatically support every board**. | [Ubuntu Core FDE](https://documentation.ubuntu.com/core/explanation/full-disk-encryption/) (explicit non-UEFI+TPM board-specific path); [RAUC bootloader matrix](https://rauc.readthedocs.io/en/latest/) (U-Boot/barebox/GRUB/EFI) |

**Cross-architecture blockers (hard):**
1. **ISA + PE Machine mismatch** — one PE cannot execute on both x86-64 and aarch64.
2. **Kernel + modules + userspace** must match architecture; container/OCI images are arch-tagged.
3. **DeviceTree / board enablement** on ARM (and some RISC-adjacent edges) is per-board; x86 mostly ACPI.
4. **Secure Boot trust anchors** differ (Microsoft/OEM db vs custom MOK vs vendor fuse keys).
5. **TPM2 availability and PCR semantics** differ; external I2C/SPI TPMs are not first-class in Ubuntu Core FDE docs.
6. **Boot firmware class:** UEFI vs U-Boot-only vs hybrid.

**What “one physical image” can mean honestly:**
- **Allowed:** one GPT/USB stick with dual ESP payload (both BOOTX64 and BOOTAA64, both UKIs, shared signed offline bundles as arch-specific blobs, shared documentation).
- **Not allowed:** one kernel PE, one rootfs without multi-arch userspace, or one Secure Boot signature covering foreign-arch code execution.

### 2. Portable media role comparison

| Role | What media holds | Durable state | Airgap fit | Unattended recovery | Wear / power-loss | GZMO fit |
|------|------------------|---------------|------------|---------------------|-------------------|----------|
| **A. Boot-only / recovery** | ESP + recovery UKI profiles (factory-reset, storagetm, rescue); optional signed offline bundles | Internal disk only | Excellent — media is cold | Strong if recovery UKI + unlock path exist | Low media wear | **Preferred companion** to internal install |
| **B. Installer media** | Live/installer OS; writes internal disk then ejects | Internal after install | Excellent for bring-up | Install once; runtime independent | Write burst at install | **Preferred bring-up** |
| **C. Immutable runtime on USB + internal encrypted data** | Read-only root on portable; `/var`/models on internal LUKS | Split | Good if USB is RO verified | Needs dual-device failure policy | USB for OS reads; internal for writes | Viable for “sealed OS stick” labs; awkward for daily edge (USB dependency) |
| **D. Everything-on-portable-NVMe** | Full OS + LUKS data + models on one high-endurance portable NVMe | All on stick | Excellent physical portability | Stick is the node; host is chassis | **Endurance and sudden-unplug** dominate | Viable **forge/lab** or true portable node; weak if chassis has better internal NVMe and stick is consumer USB |

**Contrast notes:**
- **A/B** match map language: portable media for boot/deploy; precise persistence is design-owned.
- **C** matches “immutable USB runtime + host data” — requires host disk discovery, binding identity to chassis TPM, and policy if stick is missing.
- **D** matches “single physical node is the stick” — simplifies topology but couples endurance, theft, and connector reliability to the entire living database.

**Rejected:** hot-plug autorun of GZMO on arbitrary running foreign OS without preinstalled trusted agent (map out-of-scope). Safe patterns require reboot into signed installer/recovery or an already-enrolled on-box agent that verifies signatures before any apply.

### 3. Production immutable / A-B mechanisms (primary docs)

None of these magically support every board; each has a firmware/bootloader contract.

#### UKI + Secure Boot + TPM measurements
- UKI signs kernel+initrd+cmdline (+ optional DTB) as one PE; Secure Boot covers the combination.
- systemd-stub measures UKI sections into **PCR 11**; credentials/addons into PCR 12; sysext into PCR 13.
- Multi-profile UKIs support regular / factory-reset / storage-target profiles in one signed file.
- Sources: [UAPI.5](https://uapi-group.org/specifications/specs/unified_kernel_image/), [systemd-stub](https://www.freedesktop.org/software/systemd/man/latest/systemd-stub.html).

#### dm-verity
- Kernel device-mapper target: read-only block device, per-block hash tree, root hash; optional FEC; corruption → I/O error.
- Pairs with signed root hash in UKI cmdline or verity signature partitions via systemd-repart.
- Source: [kernel dm-verity](https://www.kernel.org/doc/html/latest/admin-guide/device-mapper/verity.html).

#### systemd-repart + systemd-sysupdate
- **repart:** grows/adds partitions on first boot (including second root for A/B), optional LUKS Format/Encrypt, Verity partitions, factory-reset mode deleting partitions marked `FactoryReset=`.
- **sysupdate:** A/B/C… partition or directory updates; transfer definitions; cryptographic verify default on; `--offline` / `acquire` then `update --offline` for airgap; does **not** create partitions (use repart first).
- Sources: [systemd-repart(8)](https://www.freedesktop.org/software/systemd/man/latest/systemd-repart.html), [systemd-sysupdate(8)](https://www.freedesktop.org/software/systemd/man/latest/systemd-sysupdate.html).

#### Automatic Boot Assessment (rollback)
- Boot entries named with `+triesLeft[-triesDone]`; loader decrements; success path reaches `boot-complete.target` → `systemd-bless-boot` clears counters; exhaustion falls back to previous entry.
- Source: [Automatic Boot Assessment](https://systemd.io/AUTOMATIC_BOOT_ASSESSMENT/), [UAPI.1 boot counting](https://uapi-group.org/specifications/specs/boot_loader_specification/).

#### bootc / ostree
- Transactional OS updates via OCI container images; ostree backend; recommended **composefs** read-only `/`; `/var` shared persistent; `/etc` 3-way merge or transient.
- `bootc install to-disk` / `to-filesystem`; optional `--block-setup tpm2-luks` via systemd-cryptenroll.
- Day-2 default fetches from image reference registry — **must override** with local/offline policy for airgap (`--target-imgref`, mirrored content, or disable network update).
- Sources: [bootc book](https://bootc-dev.github.io/bootc/), [filesystem](https://bootc-dev.github.io/bootc/filesystem.html), [install](https://bootc-dev.github.io/bootc/bootc-install.html).

#### RAUC
- Signed mandatory bundles (x.509); A/B or recovery+normal slots; bootloader adapters (GRUB, barebox, U-Boot, EFI); USB stick or server; mark-good/mark-bad; factory bring-up partitioning out of scope (points at systemd-repart).
- Strong embedded/Yocto fit; not a full desktop distro story.
- Sources: [RAUC docs](https://rauc.readthedocs.io/en/latest/), [basics](https://rauc.readthedocs.io/en/latest/basic.html).

#### Mender
- Client/server fleet updater; A/B OS artifacts; standalone or managed mode; Secure Boot integration docs exist for Debian/Yocto paths.
- **Managed mode polls a server** — poor default for runtime airgap unless standalone offline artifacts only.
- Source: [Mender introduction](https://docs.mender.io/overview/introduction).

#### Ubuntu Core
- Snap-confined immutable core; partitions `ubuntu-seed` / `ubuntu-boot` / encrypted `ubuntu-save` + `ubuntu-data`; FDE when UEFI Secure Boot + TPM 2.0 (+ IOMMU); grade `secured` mandates FDE+SB; recovery modes: install, run, recover, factory-reset; recovery key prompt if TPM unseal fails.
- Board enablement via gadget/kernel snaps; non-UEFI+TPM needs custom hooks.
- Model/store assertions and refresh control are Canonical-ecosystem heavy; airgap needs carefully controlled brand store / offline assertions.
- Sources: [FDE](https://documentation.ubuntu.com/core/explanation/full-disk-encryption/), [recovery modes](https://documentation.ubuntu.com/core/explanation/recovery-modes/), [security/sandboxing](https://documentation.ubuntu.com/core/explanation/security-and-sandboxing/).

### 4. Encryption, secrets, anti-rollback, slots, wear

#### LUKS2 + TPM2 + recovery keys
- **systemd-cryptenroll** enrolls TPM2, FIDO2, PKCS#11, passphrases, and **computer-generated recovery keys** (high entropy, QR-friendly) into LUKS2 JSON tokens only (not LUKS1).
- TPM2 binding via PCR list and/or **signed PCR policies** (`.pcrsig` / public key) so vendor-signed kernel updates can unseal without re-encrypting on every UKI build.
- Recommended PCR combos often include 7 (Secure Boot policy), 11 (kernel-boot / UKI), 14 (shim MOK)—not brittle PCRs 0/2 that change on every firmware byte.
- Sources: [systemd-cryptenroll(1)](https://www.freedesktop.org/software/systemd/man/latest/systemd-cryptenroll.html), UKI `.pcrsig` in [UAPI.5](https://uapi-group.org/specifications/specs/unified_kernel_image/).

#### Signed offline bundles
- **sysupdate:** verify downloads; offline acquire/update path.
- **RAUC:** mandatory bundle signatures; optional encryption to recipients; USB install.
- **bootc/ostree:** signed commits / containers; airgap via local registry or archive side-load (operator must design; default is pull by ref).
- **Ubuntu Core:** assertion-signed snaps; offline strategy is product-specific.
- GZMO constitution: code/schema/model/security/capability expansion remain **operator-signed**—OS bundle signing is necessary but not sufficient for living evolution authority.

#### Anti-rollback
- Boot counting + bless (systemd) or RAUC/Mender mark-good after health checks.
- Secure Boot **SBAT** and dbx for boot-component revocation (UKI `.sbat`).
- dm-verity root hash pinned in signed UKI prevents silent rootfs rollback to different content without also rolling kernel measurement (if PCR-bound).
- **Version counters in TPM NV** or signed minimum-version policies are additional patterns (not fully specified here; treat as design option for ticket 06/08).
- Application-level: refuse promoting evolution candidates tagged below last-good monotonic epoch (GZMO authority plane—not OS).

#### Recovery keys and unattended recovery
- Always enroll a **recovery key** separate from TPM for chassis move, TPM clear, or PCR breakage (Ubuntu Core documents recovery-key prompt when TPM lacks valid key; systemd-cryptenroll recovery-key feature).
- Unattended recovery on airgap edge: prefer **automatic fallback to last-known-good OS slot** without human at console; reserve recovery-key entry for disaster and physical-attested maintenance.
- Multi-profile UKI factory-reset / recover profiles + repart factory-reset for controlled wipe.
- Ubuntu Core distinguishes **recover** (temp FS, data intact), **factory reset** (keeps ubuntu-save essentials), **install** (wipes save)—useful taxonomy for GZMO runbooks.

#### Data / model / candidate slots
- Separate GPT partitions (or LUKS subvolumes) recommended:
  - `sysA` / `sysB` — immutable OS (+ verity)
  - `esp` / `xbootldr` — UKIs only
  - `var` or `data` — living vault, Redis/Qdrant/Neo4j state (if retained), audit
  - `models` — large weights; high capacity; optional separate LUKS
  - `candidates` — shadow evaluation area; wipeable; never auto-promoted to OS
- bootc/ostree: OS deployments under `/ostree`; app data in `/var` deliberately **not** rolled back with OS.
- systemd-repart can create missing data partitions on first boot from minimal image seed.

#### Storage wear and power-loss
- **dm-verity / composefs / erofs root:** read-heavy OS path reduces write amp on OS media.
- **f2fs/btrfs/ext4 + barriers:** data partitions need ordered journals; sudden USB unplug on role D risks corruption—prefer internal power-loss-protected NVMe for SoT.
- **Portable NVMe endurance:** vendor TBW varies widely; treat consumer sticks as insufficient for continuous vector DB write amplification without evidence (ticket 01 hardware brief owns device picks).
- **sysupdate/repart:** incomplete downloads recognized and flushed before retry (sysupdate robustness claim).
- Avoid placing high-churn WAL/DB on SD/eMMC without endurance class validation.

### 5. First-boot hardware discovery and unattended operation

- **systemd-repart** on initrd adjusts partition table to disk size; can encrypt empty volumes with deferred passphrase/TPM enroll.
- UKI **`.hwids` / `.dtbauto`** select board-specific DT and firmware blobs inside one arch image family.
- bootc install discovers block devices when run privileged from installer environment.
- Living layer (GZMO): capability ladder should inventory CPU/RAM/accelerator/TPM/disk after mount and **declare unavailable capabilities** rather than silent degrade (map constraint)—boot research only supplies hooks (smbios, sysfs, `systemd-detect-virt`, accelerator device nodes).
- Unattended: enable boot assessment health gates tied to living readiness (e.g., order critical units before `boot-complete.target`) so bad OS+config combos revert; do not bless boot solely because pid1 reached multi-user if vault/engines failed hard floors.

## Options and trade-offs

### Option 1 — Internal NVMe appliance + portable installer/recovery (recommended shape)

**Shape:** Installer USB (multi-arch payloads) → installs arch-matching immutable OS to internal disk → LUKS2 data/model/candidate partitions → eject media. Recovery USB for rescue profiles and offline signed bundles.

| Pros | Cons |
|------|------|
| Matches one-physical-node with best endurance disk | Requires internal storage on reference hardware |
| Clear trust: chassis TPM binds data | Two media roles to version (installer + recovery can be one stick) |
| OS A/B without USB dependency | Not “pocket the whole brain” portable |
| Aligns with bootc install, sysupdate, Ubuntu Core-like layouts | |

### Option 2 — Immutable portable OS + internal data (split)

**Shape:** Verity root on USB always inserted; TPM-bound LUKS on internal NVMe for state.

| Pros | Cons |
|------|------|
| OS physically removable | Boot fails if stick missing/damaged |
| Fast OS reimage | USB connector reliability; performance |
| | Binding identity across chassis+stick is subtle |

### Option 3 — Everything on portable NVMe

**Shape:** Single stick is the node; host is dumb compute/power.

| Pros | Cons |
|------|------|
| Maximum mobility of sovereignty boundary | Endurance, theft, unplug, thermal |
| Simple mental model | Host GPU/NPU drivers still chassis-specific—stick may not boot every host |
| | Still need **per-arch** images on stick |

### Option 4 — Ubuntu Core as substrate

| Pros | Cons |
|------|------|
| Integrated FDE, recovery modes, confinement | Snap/store/assertion gravity; airgap complexity |
| Mature recovery taxonomy | Board enablement cost; grade/safety matrix |
| | Less natural for custom Rust living daemon + compose sidecars without snap packaging work |

### Option 5 — RAUC/Mender on Yocto-ish image

| Pros | Cons |
|------|------|
| Excellent embedded A/B + USB bundles | Heavier custom distro ownership |
| U-Boot-friendly | Mender server antithetical to runtime airgap if managed |
| | Duplicates effort vs systemd/bootc on UEFI x86/ARM servers |

### Option 6 — bootc/ostree appliance (Fedora/CentOS bootc-class)

| Pros | Cons |
|------|------|
| OCI delivery familiar to GZMO container world | Default day-2 network pull |
| composefs immutability; install tpm2-luks | ostree `/var` semantics vs desire to ship model trees in image carefully |
| | Arch-specific images; Secure Boot integration is distro-dependent |

**Trade-off summary for GZMO:** Prefer **Option 1** mechanisms drawn from **UKI + dm-verity or bootc composefs + systemd-repart/sysupdate + cryptenroll + boot assessment**, with **RAUC-style signed USB bundles** as the offline transport metaphor even if the implementation is sysupdate transfers. Treat Ubuntu Core as a reference architecture for FDE/recovery taxonomy, not a mandatory substrate. Use multi-arch **media**, single-arch **installations**.

## Constraints for GZMO

1. **One physical node / runtime airgap** — no required cloud pull for boot or core metabolism; offline signed bundles only for code/OS/model expansion.
2. **Local containers allowed** — sidecars may run on immutable host; host OS still needs A/B and verity story.
3. **Adaptive capability ladder** — first boot must inventory and declare; missing TPM ⇒ degraded encryption policy must be explicit (refuse `secured` grade equivalent or require passphrase-only with operator acknowledgment).
4. **Memory evolution autonomous; code/schema/model/security/capability operator-signed** — OS auto-rollback ≠ autonomous promotion of living code.
5. **Non-compensable floors** — faithfulness, sovereignty, reliability, resource, audit, rollback: boot assessment and data partition separation serve reliability/rollback floors; LUKS+TPM serve sovereignty; audit log volume must survive OS rollback (place under persistent data, not ephemeral OS slot).
6. **No hot-plug autorun** on foreign hosts.
7. **No single PE for both arches** — ship dual-arch installer media; pin each installation to discovered arch.
8. **Candidate slots** isolated from production vault; evaluable offline; wipeable without OS reinstall.
9. **Recovery key** escrow is operator procedure (airgap: printed/QR in sealed storage)—design must not soft-lose the only unlock secret in TPM.
10. **CT101 / living-appliance evidence** (`docs/LIVING_APPLIANCE.md`, `docs/AIRGAP_LIVING.md`) is runtime shape of sidecars, not yet a signed boot story—do not confuse compose pin with firmware trust.

## Unknowns

- Exact Secure Boot key ownership model for GZMO-branded keys vs OEM db (operator decision).
- Whether release-reference hardware (ticket 01) guarantees UEFI+TPM2+IOMMU on both arches or mixes U-Boot ARM SKUs.
- Target root filesystem (erofs+verity vs composefs ostree vs squashfs RAUC) once hardware ladder is fixed.
- Whether models live in LUKS partition, ostree unbound images, or container volume—storage ticket 05/10 coupling.
- Concrete PCR policy set per hardware profile after measuring real boot chains (must be lab-measured, not copied blindly).
- Portable NVMe TBW class for role D if ever selected (hardware evidence pending).
- Legal/operational recovery-key custody for multi-operator environments.
- bootc offline update UX maturity on non-Fedora bases as of 2026-08-31 (CLI stable; distro coverage uneven per bootc docs).
- NVIDIA Jetson / discrete GPU secure boot chains if forge profile needs them (board-specific; Ubuntu Core has Jetson deploy docs but not a free generic).

## Primary sources

### Specifications and firmware
- [UAPI.5 Unified Kernel Image](https://uapi-group.org/specifications/specs/unified_kernel_image/)
- [UAPI.1 Boot Loader Specification](https://uapi-group.org/specifications/specs/boot_loader_specification/)
- [PE Format — Machine Types](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format) (Microsoft Learn; page updated 2025-07-14, crawled 2026-08-31)
- [Linux TPM PCR Registry (UAPI.7)](https://uapi-group.org/specifications/specs/linux_tpm_pcr_registry/) (via cryptenroll references)

### systemd / kernel
- [systemd-stub(7)](https://www.freedesktop.org/software/systemd/man/latest/systemd-stub.html) (systemd 261.2 docs)
- [systemd-repart(8)](https://www.freedesktop.org/software/systemd/man/latest/systemd-repart.html)
- [systemd-sysupdate(8)](https://www.freedesktop.org/software/systemd/man/latest/systemd-sysupdate.html)
- [systemd-cryptenroll(1)](https://www.freedesktop.org/software/systemd/man/latest/systemd-cryptenroll.html)
- [Automatic Boot Assessment](https://systemd.io/AUTOMATIC_BOOT_ASSESSMENT/)
- [dm-verity](https://www.kernel.org/doc/html/latest/admin-guide/device-mapper/verity.html)

### Image-based OS and updaters
- [bootc documentation](https://bootc-dev.github.io/bootc/)
- [bootc filesystem](https://bootc-dev.github.io/bootc/filesystem.html)
- [bootc install](https://bootc-dev.github.io/bootc/bootc-install.html)
- [RAUC](https://rauc.readthedocs.io/en/latest/) · [RAUC basics](https://rauc.readthedocs.io/en/latest/basic.html)
- [Mender introduction](https://docs.mender.io/overview/introduction)
- [Ubuntu Core full disk encryption](https://documentation.ubuntu.com/core/explanation/full-disk-encryption/)
- [Ubuntu Core recovery modes](https://documentation.ubuntu.com/core/explanation/recovery-modes/)
- [Ubuntu Core security and sandboxing](https://documentation.ubuntu.com/core/explanation/security-and-sandboxing/)

### Local GZMO doctrine (context, not boot implementation)
- `GZMO/.scratch/self-developing-living-database/map.md` — topology, no hot-plug autorun, authority floors
- `GZMO/.scratch/self-developing-living-database/issues/00-north-star-framing.md` — approved one-node airgap framing
- `GZMO/docs/ADR-0004-airgap-living-usp.md`, `GZMO/docs/AIRGAP_LIVING.md`, `GZMO/docs/LIVING_APPLIANCE.md` — living runtime shape on one box
