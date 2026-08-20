# GZMO Demo: Minimum Bare-Metal Living Lab

**Status:** Approved design
**Date:** 2026-08-20
**Target repository:** `maximilianwruhs-cyber/gzmo-demo` (private initially)

## Purpose

`gzmo-demo` is a deployment-only repository for demonstrating the full GZMO
living-memory loop on one physical, airgapped machine. It packages a pinned GZMO
release and its local dependencies; it does not fork, reduce, or rebrand GZMO.

The shortest successful demonstration is:

```text
folder ingest
  -> cited FTS + vector recall
  -> distill + verify + promote
  -> recall of the promoted fact, labeled separately from its source passage
```

This preserves the product invariants from ADR-0004 and ADR-0007:

- one living product, not a lite SKU;
- one overnight writer;
- no public network dependency in the core path;
- local MCP and loopback-only service endpoints;
- honest failure when a required living component is unavailable.

## Reference Environment

The sole supported initial profile is:

- one x86_64 Ubuntu 24.04 LTS host;
- 32 GB RAM;
- one NVIDIA GPU with a working proprietary driver;
- Docker Engine with the Compose plugin and NVIDIA container support;
- systemd;
- a USB drive prepared on a separate online Linux machine.

Containers are allowed on the physical host. GZMO and local model servers run as
systemd services; Redis, Qdrant, and Neo4j run through Docker Compose.

## Repository Boundary

`gzmo-demo` owns:

- the locked artifact manifest;
- online USB-bundle preparation;
- offline installation and upgrades;
- service and configuration templates;
- a small redistributable demo corpus;
- health, status, and end-to-end acceptance commands;
- operator documentation for the single supported profile.

It does not own:

- GZMO business logic or source releases;
- alternate hardware profiles;
- a second memory implementation;
- cloud inference fallbacks;
- public MCP transport;
- production fleet management;
- model binaries, secrets, or generated USB bundles in Git.

## Upstream GZMO Prerequisite

The current GZMO `ingest-dir` command extracts and verifies directly into the
vault. It does not preserve raw folder passages as a separately labeled,
same-sitting corpus. The existing Pi KB indexer is an auxiliary legacy path and
must not be copied into `gzmo-demo`.

Before the first demo bundle is locked, the main GZMO repository must release a
stable corpus contract:

```text
gzmo corpus ingest-dir <path> --json [--defer-distill]
```

The command must:

- chunk supported source files deterministically;
- store corpus passages in a local SQLite FTS index separate from promoted
  semantic facts;
- embed and upsert the same passage IDs into the configured Qdrant knowledge
  collection;
- preserve source path and chunk provenance;
- create a distill session for the normal verify/promote path, enqueueing it by
  default; `--defer-distill` leaves it unqueued for an explicit sole-writer
  one-shot run;
- return a JSON receipt containing passage counts, FTS and vector counts, source
  paths, the distill session ID, and whether it was enqueued.

`gzmo memory search --json` must label every result as either
`corpus_passage` or `promoted_fact` and report the retrieval channels that
contributed to its rank. A corpus result satisfies the demo contract only when
both `fts` and `vector` contributed. A promoted result must carry a fact ID and
evidence provenance.

The command, result schema, and tests belong in GZMO. `gzmo-demo` consumes the
first tagged GZMO release containing that contract; it does not implement or
patch the memory pipeline itself.

## Packaging Approach

The repository uses a pinned offline release bundle rather than a Git submodule,
an offline Rust build cache, Ansible, or Nix. This keeps the lab repository small
and makes the boundary between product code and deployment code explicit.

`bundle.lock` is the single version authority for the
`ubuntu-24.04-nvidia-32gb` profile. Each artifact entry records:

- logical name and role;
- upstream source;
- immutable version or digest;
- license identifier and redistribution notes;
- expected byte size;
- SHA-256 digest;
- destination inside the offline bundle.

The initial 32 GB NVIDIA profile follows the currently validated model families:

- Qwen3.6-35B-A3B quantized cognition model for Prime;
- Qwen3-Embedding-0.6B Q8 for 1024-dimensional embeddings.

The lock file must identify exact artifacts and hashes before a bundle can be
prepared. Model artifacts are downloaded during preparation and are never
committed to the repository.

## Repository Layout

```text
gzmo-demo/
  README.md
  LICENSE
  bundle.lock
  scripts/
    prepare-usb
    install
    status
    demo
    uninstall
  config/
    gzmo.toml.in
    living.env.in
  compose/
    compose.yaml
  systemd/
    gzmo-prime.service.in
    gzmo-embed.service.in
    gzmo-daemon.service.in
  fixtures/
    corpus/
    expected.json
  tests/
    static/
    installer/
  docs/
    AIRGAP_RUNBOOK.md
    TROUBLESHOOTING.md
```

Scripts are non-interactive by default, support `--help`, and emit actionable
errors. Required operator choices, such as the target USB mount and installation
prefix, are explicit flags rather than inferred destructive paths.

## Two-Phase Workflow

### Online preparation

`scripts/prepare-usb --device <mounted-path>`:

1. validates the host tools and free space;
2. resolves only entries already present in `bundle.lock`;
3. downloads the pinned GZMO release, model artifacts, OS packages, llama.cpp
   runtime, and OCI images;
4. exports container images as archives;
5. writes a complete bundle manifest with hashes and sizes;
6. verifies the finished bundle from the USB contents;
7. never installs or starts lab services.

Preparation must not overwrite an unrelated non-empty destination. Re-running
against the same bundle version is idempotent.

### Offline installation

`scripts/install --bundle <mounted-path>`:

1. verifies every file before changing the host;
2. validates the supported OS, architecture, GPU, driver, Docker, Compose,
   NVIDIA container runtime, systemd, ports, and disk space;
3. refuses installation if another living writer owns the target vault;
4. stages a versioned release under `/opt/gzmo-demo/releases/<bundle-id>`;
5. loads pinned OCI images and installs local packages from the bundle;
6. renders configuration with all service addresses on `127.0.0.1`;
7. installs Prime, embedding, and GZMO daemon systemd units;
8. creates persistent state under `/var/lib/gzmo`;
9. starts sidecars and host services in dependency order;
10. runs health checks before switching `/opt/gzmo-demo/current`;
11. records the installed bundle ID and artifact digests.

A failed install does not move the active symlink and therefore leaves the
previous working release intact. Re-running the same install resumes safely.

## Runtime Topology

```text
one physical host
  systemd
    Prime / llama.cpp       127.0.0.1:8000
    embeddings / llama.cpp  127.0.0.1:8081
    gzmo daemon             sole writer
  Docker Compose
    Redis                   127.0.0.1:6379
    Qdrant                  127.0.0.1:6333
    Neo4j                   127.0.0.1:7687
  persistent state
    /var/lib/gzmo/vault.db
    /var/lib/gzmo/honeypot/
  local clients
    gzmo CLI
    gzmo-living MCP over stdio
```

The Compose file must bind published ports explicitly to loopback. Host firewall
guidance is defense in depth, not a substitute for loopback binding.

## Demo Contract

`scripts/demo` is deterministic and safe to repeat:

1. checks full living health and bundle identity;
2. creates an isolated demo session and fixture namespace;
3. ingests the bundled fixture corpus through
   `gzmo corpus ingest-dir --json --defer-distill`;
4. queries for a fixture-specific claim;
5. requires a `corpus_passage` result and evidence that both FTS and vector
   channels participated;
6. uses the returned distill session through the normal distill pipeline;
7. waits for verify and promote with a bounded timeout;
8. queries again and requires a promoted fact;
9. confirms that the corpus passage and promoted fact have distinct labels;
10. writes text and JSON reports without corpus content or secrets.

The script does not bypass production logic by inserting directly into the
corpus index, vault, honeypot, or Qdrant. It may stop the daemon temporarily,
run existing one-shot distill/promote/embed commands as the sole writer, and
restart the daemon under a cleanup trap. Any missing product contract must be
added to the main GZMO repository and released before `gzmo-demo` consumes it.

## Failure Semantics

The installer and demo fail closed for:

- unsupported hardware or operating system;
- incomplete bundles or digest mismatches;
- incompatible GZMO and bundle contract versions;
- unavailable GPU acceleration;
- non-loopback service configuration;
- occupied required ports;
- missing NVIDIA container support;
- an existing living-writer claim;
- unhealthy Redis, Qdrant, Neo4j, Prime, embeddings, or GZMO daemon;
- keyword-only recall when hybrid recall is required;
- promotion timeout or missing evidence labels.

No failure may trigger a cloud request or be reported as living success.
Warnings are permitted only for conditions outside the acceptance contract and
must remain visible in both report formats.

## Verification

### Continuous integration

CI runs without a GPU and checks:

- lock-file schema, uniqueness, immutable references, licenses, and digest shape;
- shell syntax and repository formatting using tools already declared by the
  repository;
- rendered configuration for loopback-only endpoints;
- Compose configuration validity;
- systemd template validity;
- installer behavior against fixture bundles, including hash failure,
  unsupported-host failure, idempotence, and active-release preservation.

### Hardware acceptance

The supported target runs one command that combines:

- bundle and installed-release integrity;
- NVIDIA and model-server readiness;
- sidecar health;
- sole-writer ownership;
- the complete demo contract.

The command exits nonzero unless every required check passes. Its JSON report
includes bundle ID, artifact versions, service states, phase timings, and check
results, but excludes secrets and corpus text.

## Security and Data Handling

- Secrets are entered or generated only on the offline host and stored in a
  root-readable environment file outside Git.
- Downloaded artifacts are accepted only when their hashes match `bundle.lock`.
- The USB preparation report preserves upstream provenance.
- Services have no wildcard listener in the supported configuration.
- The installer does not modify unrelated Docker workloads or systemd units.
- Uninstall removes only paths and units recorded in the install manifest;
  persistent vault data requires a separate explicit purge flag.

## Acceptance Criteria

The first release of `gzmo-demo` is complete when:

1. a clean online Linux machine can prepare the locked USB bundle;
2. a clean supported host can install it without network access;
3. all runtime endpoints are local-only;
4. reboot restores the full stack without manual ordering;
5. a second writer is refused;
6. the pinned GZMO release exposes the corpus-ingest and labeled-search
   contracts;
7. the demo proves cited hybrid corpus recall, normal-path verified promotion,
   and promoted-fact recall;
8. tampered or incomplete bundles are rejected before host changes;
9. a failed upgrade preserves the previous active release;
10. the hardware acceptance command produces passing text and JSON reports.
