# Lab Architecture — Compilation Context and Feature Flag Analysis

## Design Rationale

This laboratory is designed to preserve and record the system, toolchain,
package, and runtime conditions under which Rust and C artifacts are compiled,
executed, and analyzed.

cfg-shield detects feature flag semantic drift: cases where `#[cfg(feature)]`
gates in a Rust crate change security-relevant behavior outside the source
file. The laboratory provides the environment in which those gates can be
activated, observed, and compared under documented conditions.

The core principle is identical to the one that motivates the tool itself:
the same source file can produce different programs depending on the
compilation context. The laboratory is designed not to hide that variability,
but to make it observable and reproducible under recorded conditions.

## Laboratory Architecture — Three Planes

The laboratory is organized into three planes with distinct methodological
functions. The nodes have deliberately separated roles. This separation
reduces the risk of conflating development, experimental execution, and
cross-context reproduction within a single system context.

| Plane | Node | System | Type | Methodological Function |
|-------|------|--------|------|------------------------|
| Development | luiswizard-hp-prodesk-600-g1-sff | Linux Mint | Bare Metal | Development, git, documentation, orchestration |
| Experimental | dell-bsd | FreeBSD 14.4 | Bare Metal | Controlled evidence generation under a documented native context |
| Validation | parrot | Parrot OS | VM (libvirt/KVM) | Structurally separate experimental context for cross-context reproduction when relevant |

Private overlay connectivity between all three nodes is provided by Tailscale
(`--accept-dns=false` on all nodes; DNS managed independently per host).

```text
Linux Mint — Development Plane
    │
    │  Development, git, documentation, orchestration
    ▼
FreeBSD 14.4 — Experimental Plane
    │
    │  Native toolchain, Jails, ZFS
    │  Compilation and execution under documented system conditions
    ▼
Source
  + Cargo.toml / Cargo.lock
  + Feature set
  + Rust toolchain version
  + Target / OS / ABI
  + Resolved dependencies
  + Build flags and environment variables
        │
        ▼
Observed Artifact + Evidence
        │
        ├──► Classification under documented conditions
        │
        └──► Cross-context reproduction
             on Parrot OS when relevant
```

## Why FreeBSD for Rust and C Analysis

The FreeBSD experimental environment allows the toolchain, package,
filesystem, and runtime conditions relevant to the experiment to be
observed and recorded directly. The objective is not to eliminate
abstraction, but to avoid treating relevant parts of the experimental
context as unobserved infrastructure.

FreeBSD Jails provide an additional isolation boundary between experiments
while retaining a documented relationship with the native FreeBSD host
environment. Jail configuration is therefore treated as part of the recorded
experimental context rather than as invisible infrastructure.

The relevant context includes:

- FreeBSD host version and patch level
- Jail configuration and package manifest
- Rust toolchain version
- C compiler version and flags, where applicable
- Cargo.toml and Cargo.lock revisions
- Cargo feature set used for each build
- Resolved dependency versions
- Target architecture and ABI
- Source revision or commit identifier
- Build commands and relevant environment variables

ZFS snapshots allow a defined laboratory state to be preserved before an
experiment:

```
zfs snapshot zroot/jails/cfg-lab@pre-experiment-YYYY-MM-DD
```

This does not by itself establish complete reproducibility or forensic
chain of custody. It provides a mechanism for preserving laboratory state
that can be combined with recorded toolchain versions, package manifests,
configuration, source revisions, hashes, build logs, and other metadata
appropriate to the experiment.

## Parrot OS — Structurally Separate Experimental Context

Parrot OS provides a structurally separate experimental context within the
laboratory architecture from which artifacts and observed behavior can be
examined using a different system context and separately selected tooling.

Cross-context reproduction is used as additional validation where relevant.
It is not assumed to be required for every confirmed finding, because some
compilation-context divergences are inherently environment-specific.

Parrot OS provides an independent experimental context within the laboratory
infrastructure. It does not constitute third-party validation.

## Relationship with cfg-shield

```text
Rust / C Source
      │
      ├──► cfg-shield
      │      Static Analysis
      │      Feature Gate Detection
      │
      └──► FreeBSD — Experimental Plane
             Compilation / Execution
             Documented Native Context
                   │
                   ▼
             Observed Artifact + Evidence
                   │
                   ├──► Classification under documented conditions
                   │
                   └──► Cross-context reproduction
                        on Parrot OS when relevant
```

## Classification Framework

Findings produced in this laboratory are classified under the three-tier
framework implemented in cfg-shield:

- **CONFIRMADO** — the condition and its relevant consequence are directly
  observable and reproducible under the documented conditions relevant to
  the experiment.

- **PROBABLE** — the condition is present and a relevant consequence is
  technically plausible, but that consequence has not yet been directly
  demonstrated.

- **OBSERVADO** — a documented or intentional context-dependent behavior
  that does not constitute an actionable finding under the current
  methodology.

Reproducibility must be demonstrated by each experiment under conditions
appropriate to that experiment. The laboratory architecture provides
separation of roles, state preservation, and mechanisms that support
reproducibility. It does not guarantee reproducibility in the abstract.


## Laboratory Observation: Package Context Divergence on Parrot OS

**Date:** 2026-08-21
**Classification:** OBSERVADO

During integration of Parrot OS into the laboratory Tailscale overlay network,
the official Tailscale installation script (install.sh) automatically
detected the host as debian bullseye and configured the package source
accordingly:

    Installing Tailscale for debian bullseye, using method apt
    deb [signed-by=/usr/share/keyrings/tailscale-archive-keyring.gpg] https://pkgs.tailscale.com/stable/debian bullseye main

The actual host is Parrot OS Echo, whose base is Debian 13 (trixie), not
bullseye (Debian 11). The installation completed without error and the
resulting binary (1.102.3) is functional.

This is classified OBSERVADO: the divergence between declared and actual
package context is observable and reproducible, but no security-relevant
behavioral difference between the bullseye and trixie package variants has
been demonstrated.

**Reference:** Tailscale 1.102.3 installed on Parrot OS Echo (Debian 13 base)
via tailscale.com/install.sh, 2026-08-21.
Upstream issue: github.com/tailscale/tailscale/issues/20960

## Research Connection

This laboratory provides the practical infrastructure for the research
presented in:

*Same Source, Different Program: Compilation Context as a Security Variable*

Tools: github.com/LuisCastellanos-dev/cfg-shield,
github.com/LuisCastellanos-dev/cobol-shield

Preprint: DOI 10.5281/zenodo.21974261
