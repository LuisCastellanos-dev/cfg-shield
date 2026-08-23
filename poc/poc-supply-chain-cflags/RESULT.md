# VTR Supply Chain Context Divergence -- PoC

## Experimental context

- FreeBSD 14.4-RELEASE-p8
- Clang 19.1.7 (system cc)
- Subject: libvalidate (dependency) + main (consumer)
- Variable: CFLAGS injected into dependency build context

## Structure

- libvalidate/validate.c: input validation function with #ifdef DISABLE_CHECK gate
- main.c: consumer program that calls validate_input()
- The consumer source is identical in both builds

## Results

Without CFLAGS=-DDISABLE_CHECK (clean build):
  validate_input(  -1) = 0  (rejected)
  validate_input(   0) = 1  (accepted)
  validate_input(  50) = 1  (accepted)
  validate_input( 100) = 1  (accepted)
  validate_input( 150) = 0  (rejected)

With CFLAGS=-DDISABLE_CHECK injected into dependency build:
  validate_input(  -1) = 1  (accepted -- BYPASSED)
  validate_input(   0) = 1  (accepted)
  validate_input(  50) = 1  (accepted)
  validate_input( 100) = 1  (accepted)
  validate_input( 150) = 1  (accepted -- BYPASSED)

## Classification: CONFIRMADO

The validation check is completely disabled in the consumer binary
when CFLAGS=-DDISABLE_CHECK is injected into the dependency compilation
context. The consumer source (main.c) is identical in both builds.
The dependency source (validate.c) is identical in both builds.
Only the compilation context of the dependency differs.

## Mechanism

This demonstrates supply chain compilation context divergence:
a flag injected into the build context of a dependency propagates
into the behavior of the final artifact without any change to source
code at any level of the dependency graph.

This is distinct from the previous C cases:
- Section 4.3 (ASan): same defect, different observability
- Section 4.4 (UB): same source, different optimization behavior
- This case: same source at all levels, validation bypassed via
  dependency build context injection

## VTR Classification

Category: Supply chain compilation context divergence
Classification: CONFIRMADO
Date: 2026-08-23
Platform: FreeBSD 14.4-RELEASE-p8 / Clang 19.1.7
