/*
 * poc-c-kasan.c — c-shield PoC Level 2: Bug Observability Divergence
 *
 * Demonstrates that the same C source file produces different bug
 * observability depending on the -fsanitize=address flag at compile time.
 * This is the userspace equivalent of CONFIG_KASAN in the Linux kernel.
 *
 * Level 2 — Bug observability divergence:
 *   Without -fsanitize=address (CONFIG_KASAN=n):
 *     use-after-free is silent — returns garbage value, no error
 *   With    -fsanitize=address (CONFIG_KASAN=y):
 *     use-after-free is detected and reported with exact location
 *
 * Build:
 *   gcc -o poc-default poc-c-kasan.c                     # UAF silent
 *   gcc -fsanitize=address -o poc-asan poc-c-kasan.c     # UAF detected
 *
 * Observed (gcc 13.3.0, Ubuntu 24.04):
 *   Without ASan: "Value after free: 3"  (garbage, no error)
 *   With    ASan: ERROR: AddressSanitizer: heap-use-after-free
 *                 READ of size 1 at 0x502000000010 thread T0
 *
 * Result: CONFIRMADO — same bug, different observability
 *
 * Linux kernel equivalent:
 *   CONFIG_KASAN=n: kernel use-after-free is silent, may cause silent corruption
 *   CONFIG_KASAN=y: kernel use-after-free is detected with full stack trace
 *
 * Copyright (C) 2026 Luis Fidel Castellanos Diaz — Vector Telemetry Research
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    char *buf = malloc(16);
    memset(buf, 'A', 16);
    free(buf);

    /*
     * Use-after-free:
     *   Without -fsanitize=address: silent — prints garbage value
     *   With    -fsanitize=address: ERROR: heap-use-after-free detected
     */
    printf("Value after free: %c\n", buf[0]);

    return 0;
}
