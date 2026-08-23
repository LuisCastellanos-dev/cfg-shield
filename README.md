# cfg-shield

**Feature flag semantic drift detector for Rust crates**

Part of VTR Research Methodology | DOI: 10.5281/zenodo.22063208
Status: Reproducible | License: Apache-2.0 | Track: Critical Infrastructure

## What it does

Detects `#[cfg(feature)]` gates that change security-relevant behavior outside
the source file. The same Rust crate with `default = []` passes `cargo test`
with 0 tests executed. With explicit `--features`, 8 tests run. cfg-shield
makes that coverage gap observable and reproducible.

## How to reproduce in 3 commands

    git clone https://github.com/LuisCastellanos-dev/cfg-shield
    cd cfg-shield
    cargo test && cargo test --features full

## What it is NOT

This tool does not claim or demonstrate remote code execution, privilege
escalation, or network exploitation. Findings are classified as CONFIRMADO,
PROBABLE, or OBSERVADO per VTR methodology -- classification reflects strength
of evidence, not severity of impact.

---

## Traction

- FreeBSD commit rGa841961da752 merged into base system
- IBM Bank-of-Z Issue #205 -- systemic timestamp defect identified and reported upstream
- SCaLE 24x submission: Same Source, Different Program: Compilation Context as a Security Variable -- pending November 2026
- Preprint: DOI 10.5281/zenodo.22063208 -- https://doi.org/10.5281/zenodo.22063208
