# Security Policy

## Scope

cfg-shield is a static analysis tool for detecting feature flag semantic drift
in Rust crates. It operates on source code and build configuration only.

## What this tool does NOT claim

- This tool does not demonstrate remote code execution (RCE)
- This tool does not demonstrate privilege escalation or network exploitation
- Findings are not CVEs unless independently verified and coordinated with affected maintainers

## Classification framework

Findings produced by this tool are classified under the VTR three-tier framework:

- CONFIRMADO: divergence and its relevant consequence are directly observable and reproducible under documented conditions
- PROBABLE: condition is present and consequence is plausible but not yet demonstrated
- OBSERVADO: documented behavior that does not constitute an actionable finding under current methodology

Classification reflects strength of evidence, not severity of impact.

## Reporting

If you identify a security issue in cfg-shield itself, please open a GitHub issue
or contact the maintainer directly via GitHub.

## Reference

VTR-RES-002: DOI 10.5281/zenodo.22063208
