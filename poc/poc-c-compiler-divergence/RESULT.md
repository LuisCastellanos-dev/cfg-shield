# VTR Compiler Context Divergence — PoC v2

## Experimental context

- FreeBSD 14.4-RELEASE-p8
- Clang 19.1.7 (LLVM)
- GCC 14.2.0 (FreeBSD Ports)
- Subject: subject2.c — signed integer overflow UB

## Results

| Compiler | Flag | INT_MAX+1 > INT_MAX |
|----------|------|---------------------|
| clang    | -O0  | 0 (false)           |
| clang    | -O2  | 1 (true)            |
| gcc14    | -O0  | 1 (true)            |
| gcc14    | -O2  | 1 (true)            |

## Classification: CONFIRMADO

clang -O0 evaluates signed overflow at runtime (wraps to negative).
clang -O2 applies UB optimization: assumes overflow cannot occur,
eliminates the comparison as always-true constant.

Same source file. Same compiler. Different optimization flag.
Different observable runtime behavior.

## Mechanism

C standard defines signed integer overflow as Undefined Behavior.
Under -O2, clang is permitted to assume UB does not occur and optimize
accordingly. Under -O0, no such assumption is applied.

This is a distinct category from the ASan case (observability divergence):
here the program behavior itself changes, not only its diagnostic visibility.
