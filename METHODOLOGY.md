# c-shield Methodology: Compilation Context Semantic Divergence

## The Unified Principle

The compilation context determines what a program does and what bugs
are observable. That context lives outside the source file — in the
Makefile, CI configuration, kernel `.config`, or operator invocation.

Changing the context without modifying the source changes the program's
security semantics and bug visibility. This is the same mechanism
across three languages and three levels of divergence.

---

## Three Languages, One Principle

### COBOL — Format Flag Semantic Divergence

```
cobc -fixed poc-same-file.cbl → compiles, MOVE 999999 dormant
cobc -free  poc-same-file.cbl → 10 compilation errors
```

The `-fixed`/`-free` flag determines whether col7=`*` is a comment
indicator or an ordinary character. The flag is not in the source file.

**Evidence:** `../corpus/fixed-format/poc-same-file.cbl`
**SHA-256:** `documented in poc-evidence.md`
**Compiler:** GnuCOBOL 3.1.2 | **Result:** CONFIRMADO

---

### Rust — Feature Flag Security Bypass

```
cargo test                          → 0 tests, bypass active
cargo test --features strict        → 8 tests, real verification
```

`#[cfg(not(feature = "strict"))]` gates the verification bypass.
The feature flag is not in the source file — it is in `Cargo.toml`
defaults or the CI invocation.

**Evidence:** `../src/main.rs` (cfg-shield itself)
**Validated against:** `LFDT-Lockness/generic-ecies` — CONFIRMADO
**Compiler:** rustc 1.96.0

---

### C — Two Levels of Divergence

#### Level 1: Security Semantic Divergence

```
gcc              poc-c-verify.c → bypass: any signature accepted
gcc -DENABLE_VERIFY poc-c-verify.c → real: invalid signatures rejected
```

`#ifdef ENABLE_VERIFY` gates the real verification.
The `-D` flag is not in the source file.

**Evidence:** `poc/poc-c-verify.c`
**SHA-256:** `2fc781a3fc27f75f06f7201b1a8a3a08a6662145b0e3bb1fc6812c37ce12e93c`
**Compiler:** gcc 13.3.0 | **Result:** CONFIRMADO

Observed:
```
DEFAULT: verify_signature(invalid) = 1  (bypass — payment authorized)
STRICT:  verify_signature(invalid) = 0  (rejected — correct)
```

#### Level 2: Bug Observability Divergence

```
gcc                      poc-c-kasan.c → UAF silent: "Value after free: 3"
gcc -fsanitize=address   poc-c-kasan.c → UAF detected: ERROR heap-use-after-free
```

`-fsanitize=address` is the userspace equivalent of `CONFIG_KASAN`
in the Linux kernel. The flag is not in the source file.

**Evidence:** `poc/poc-c-kasan.c`
**Compiler:** gcc 13.3.0 | **Result:** CONFIRMADO

Linux kernel equivalent:

| Config | Behavior |
|--------|----------|
| `CONFIG_KASAN=n` (default in production) | UAF silent, may cause corruption |
| `CONFIG_KASAN=y` | UAF detected with full stack trace |
| `CONFIG_KCSAN=n` (not set in Ubuntu 24.04) | Data races silent |
| `CONFIG_KCSAN=y` | Data races detected and reported |

---

## Classification Framework (VTR Audit Master Prompt v3.5)

| Classification | Criteria |
|---|---|
| **CONFIRMADO** | Observable output diverges between two compilations of same file |
| **PROBABLE** | Flag gates security-sensitive code, divergence not yet observed |
| **OBSERVADO** | Deliberate design decision, documented in codebase |
| **HIPÓTESIS** | Divergence theoretically possible, path not verified |

---

## Levels of Divergence

| Level | Mechanism | Observable evidence |
|---|---|---|
| 1 — Compilation validity | Same file valid/invalid under different flags | Compiler accepts or rejects |
| 2 — Security semantics | Same input accepted/rejected by same code | Output differs |
| 3 — Bug observability | Same bug silent or detected under different flags | Error message vs silence |

All three levels share the same root: **the flag lives outside the file.**

---

## FreeBSD Analysis Results

Audited `sys/kern/kern_osd.c`, `sys/kern/kern_malloc.c`,
`sys/netinet/tcp_input.c`, `sys/netipsec/key_debug.h`.

**Finding:** `INVARIANTS` and `WITNESS` in FreeBSD follow the same
pattern but represent **intentional design decisions**, not omissions.

- `#ifdef MAC` — MAC policy check in TCP: deliberate, documented option
- `#ifdef INVARIANTS` + `KASSERT` — debug assertions: deliberate tradeoff
- `#ifdef IPSEC_DEBUG` — diagnostic logging: correct design

**Classification:** OBSERVADO — FreeBSD is a mature codebase where
`#ifdef` gates on security options are documented and intentional.
c-shield distinguishes between intentional design decisions (OBSERVADO)
and unintentional omissions (CONFIRMADO/PROBABLE).

**Key distinction for c-shield:**
The detector must verify whether a `#ifdef` absence is:
- Documented as an intentional tradeoff (OBSERVADO — not actionable)
- An omission without documentation (PROBABLE/CONFIRMADO — actionable)

---

## Running the PoC Suite

```bash
cd poc/
bash run-poc-c.sh
```

Requires: gcc with ASan support (gcc 11+ recommended).
Tested on: Ubuntu 24.04, gcc 13.3.0.

---

## Relationship to cobol-shield

This methodology extends the Source Transformation Integrity framework
from [cobol-shield](https://github.com/LuisCastellanos-dev/cobol-shield)
to the C/Linux ecosystem.

The paper documenting the COBOL case:
[Source Transformation Integrity in Legacy COBOL Systems](https://zenodo.org/records/21973424)
DOI: 10.5281/zenodo.21974261

## Known Limitations

**Test count as evidence proxy:** CFG-R01 uses test count differential (cargo test vs cargo test --features) as the primary evidence of semantic divergence. This proxy fails when a project distributes tests uniformly across features — equal test counts with different semantics would be classified as PROBABLE instead of CONFIRMADO. Output differential analysis is required as secondary evidence in those cases.

**Feature name dependency:** Classification of CONFIRMADO/PROBABLE assumes feature flag names follow recognizable security conventions (e.g., `strict`, `verify`, `auth`). A security-relevant flag with a non-descriptive name (e.g., `legacy`, `compat`, `v1`) may not be classified correctly without manual review of the gated code block.

**Cargo monoculture:** All Rust rules assume a `Cargo.toml`-structured project with `cargo test` as the test runner. Projects using custom build systems, build.rs scripts that conditionally enable features, or workspace-level feature resolution may produce false negatives.

**C sanitizer requirement:** Level 2 and Level 3 C divergence (security semantics, bug observability) requires integration with ASan/UBSan to produce observable evidence. Without sanitizers, undefined behavior introduced by a conditional `#ifdef` may not manifest in output — divergence exists but is not observable by the tool alone.

**Single validated instance per language:** COBOL divergence validated on GnuCOBOL 3.1.2 only. Rust divergence validated against generic-ecies. C divergence validated against FreeBSD if_ovpn.c/if_wg.c. Each represents one confirmed instance — generalization to other projects is INFERENCIA until tested.

---

*Luis F. Castellanos — Vector Telemetry Research (VTR)*
