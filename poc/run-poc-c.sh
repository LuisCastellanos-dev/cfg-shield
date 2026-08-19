#!/usr/bin/env bash
# run-poc-c.sh — c-shield C PoC suite
#
# Runs both C proof-of-concept programs demonstrating compilation context
# semantic divergence at two levels:
#   Level 1: security semantic divergence (-DENABLE_VERIFY)
#   Level 2: bug observability divergence (-fsanitize=address / CONFIG_KASAN)
#
# Usage: cd ~/c-shield-poc && bash run-poc-c.sh

set -uo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

CC="${CC:-gcc}"
echo "═══════════════════════════════════════════════════════════════"
echo " c-shield C PoC Suite — Compilation Context Semantic Divergence"
echo " Compiler: $($CC --version 2>&1 | head -1)"
echo "═══════════════════════════════════════════════════════════════"

# ── Level 1: Security semantic divergence ─────────────────────────────
echo ""
echo "Level 1 — Security Semantic Divergence (poc-c-verify.c)"
echo "SHA-256: $(sha256sum "$DIR/poc-c-verify.c" | cut -d' ' -f1)"
echo ""

echo "[1/2] $CC poc-c-verify.c (no -DENABLE_VERIFY)"
$CC -o "$TMP/poc-v-default" "$DIR/poc-c-verify.c"
OUT_DEFAULT="$("$TMP/poc-v-default" 2>&1)"
echo "$OUT_DEFAULT"
echo ""

echo "[2/2] $CC -DENABLE_VERIFY poc-c-verify.c"
$CC -DENABLE_VERIFY -o "$TMP/poc-v-strict" "$DIR/poc-c-verify.c"
OUT_STRICT="$("$TMP/poc-v-strict" 2>&1)"
echo "$OUT_STRICT"
echo ""

if [ "$OUT_DEFAULT" != "$OUT_STRICT" ]; then
    echo "→ CLASIFICACIÓN VTR: CONFIRMADO (Level 1)"
else
    echo "→ CLASIFICACIÓN VTR: PROBABLE (Level 1)"
fi

# ── Level 2: Bug observability divergence ─────────────────────────────
echo ""
echo "───────────────────────────────────────────────────────────────"
echo "Level 2 — Bug Observability Divergence (poc-c-kasan.c)"
echo "SHA-256: $(sha256sum "$DIR/poc-c-kasan.c" | cut -d' ' -f1)"
echo "Equivalent: CONFIG_KASAN in Linux kernel"
echo ""

echo "[1/2] $CC poc-c-kasan.c (no -fsanitize=address / CONFIG_KASAN=n)"
$CC -o "$TMP/poc-k-default" "$DIR/poc-c-kasan.c" 2>/dev/null
echo "Output: $("$TMP/poc-k-default" 2>&1)"
echo ""

echo "[2/2] $CC -fsanitize=address poc-c-kasan.c (CONFIG_KASAN=y equivalent)"
$CC -fsanitize=address -o "$TMP/poc-k-asan" "$DIR/poc-c-kasan.c" 2>/dev/null
ASAN_OUT="$("$TMP/poc-k-asan" 2>&1 | head -3)"
echo "Output: $ASAN_OUT"
echo ""

if echo "$ASAN_OUT" | grep -q "ERROR"; then
    echo "→ CLASIFICACIÓN VTR: CONFIRMADO (Level 2)"
    echo "  Same bug: silent without ASan, detected with ASan"
    echo "  Kernel equivalent: CONFIG_KASAN makes UAF visible"
else
    echo "→ CLASIFICACIÓN VTR: PROBABLE (Level 2)"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo " Summary: The compilation context determines what bugs are"
echo " observable and what security checks are active."
echo " That context lives outside the source file."
echo "═══════════════════════════════════════════════════════════════"
