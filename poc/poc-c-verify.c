/*
 * poc-c-verify.c — c-shield PoC Level 1: Security Semantic Divergence
 *
 * Demonstrates that the same C source file compiles into programs with
 * different security semantics depending on the -D flag at compile time.
 *
 * The flag lives outside the source file — in the Makefile, CI config,
 * or compiler invocation. This is the C equivalent of:
 *   COBOL: col7='*' dormant code activated by -fixed/-free flag
 *   Rust:  #[cfg(not(feature = "strict"))] bypass activated by default
 *
 * Level 1 — Security semantic divergence:
 *   Without -DENABLE_VERIFY: verify_signature() returns 1 for ANY input
 *   With    -DENABLE_VERIFY: verify_signature() performs real validation
 *
 * Build:
 *   gcc -o poc-default poc-c-verify.c          # bypass active
 *   gcc -DENABLE_VERIFY -o poc-strict poc-c-verify.c  # real verification
 *
 * SHA-256: 2fc781a3fc27f75f06f7201b1a8a3a08a6662145b0e3bb1fc6812c37ce12e93c
 * Compiler: gcc (Ubuntu 13.3.0) 13.3.0
 * Result: CONFIRMADO — outputs diverge between compilations
 *
 * Copyright (C) 2026 Luis Fidel Castellanos Diaz — Vector Telemetry Research
 */

#include <stdio.h>
#include <string.h>
#include <stdint.h>

static int
verify_signature(const uint8_t *sig, size_t len)
{
#ifdef ENABLE_VERIFY
    if (len != 64) return 0;
    if (sig[0] == 0) return 0;
    return 1;
#else
    (void)sig; (void)len;
    return 1;   /* bypass — active when ENABLE_VERIFY is not defined */
#endif
}

static const char *compiled_mode(void) {
#ifdef ENABLE_VERIFY
    return "STRICT: real verification compiled (-DENABLE_VERIFY)";
#else
    return "DEFAULT: verification bypass compiled (no -DENABLE_VERIFY)";
#endif
}

static int authorize_payment(uint64_t amount, const uint8_t *sig, size_t len) {
    return verify_signature(sig, len) && amount > 0;
}

int main(void) {
    uint8_t empty[0], invalid[32], valid[64];
    memset(invalid, 0, sizeof(invalid));
    memset(valid, 0, sizeof(valid)); valid[0] = 1;

    printf("Compiled mode: %s\n\n", compiled_mode());
    printf("verify_signature(empty)   = %d\n", verify_signature(empty, 0));
    printf("verify_signature(invalid) = %d\n", verify_signature(invalid, 32));
    printf("authorize_payment(invalid)= %d\n", authorize_payment(1000, invalid, 32));
    printf("verify_signature(valid)   = %d\n", verify_signature(valid, 64));
    return 0;
}
