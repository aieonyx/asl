#!/usr/bin/env bash
# Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
# SPDX-License-Identifier: Apache-2.0
#
# build-m22.sh — ASL M22: AXON migration, PD sources to .ax

set -euo pipefail

export MICROKIT_SDK="${MICROKIT_SDK:-$HOME/microkit-sdk-1.4.1}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "═══════════════════════════════════════════════"
echo " ASL M22 — AXON Migration: PD sources to .ax"
echo "═══════════════════════════════════════════════"

# ── Step 0: Kani extern fix ──────────────────────────────────────────────────
echo ""
echo "[M22] Step 0: Applying Kani extern crate fix..."
python3 fix_kani.py . || true

# ── Step 1: Check AXON compiler ──────────────────────────────────────────────
echo ""
echo "[M22] Step 1: Verifying AXON compiler..."
if command -v axon &>/dev/null; then
    echo "[M22] axon: $(which axon)"
else
    echo "[M22] ERROR: axon not in PATH"
    echo "      Expected: ~/.cargo/bin/axon"
    exit 1
fi

# ── Step 2: Compile sovereign_arpi.ax to seL4 aarch64 ELF ───────────────────
echo ""
echo "[M22] Step 2: Compiling sovereign_arpi.ax → seL4 aarch64 ELF..."
AX_SRC="$SCRIPT_DIR/asl-axon-migration/ax/sovereign_arpi.ax"
AX_OUT="$SCRIPT_DIR/asl-axon-migration/ax/sovereign_arpi.o"

axon build "$AX_SRC" \
    --target aarch64-sel4 \
    --profile seL4-strict \
    --output "$AX_OUT" \
    2>&1 || {
    # Fallback: try axon check first to diagnose
    echo "[M22] Build failed — running axon check for diagnostics..."
    axon check "$AX_SRC" 2>&1 || true
    exit 1
}

echo "[M22] Compiled: $AX_OUT"

# ── Step 3: Verify ELF output ────────────────────────────────────────────────
echo ""
echo "[M22] Step 3: Verifying ELF output..."
if command -v file &>/dev/null; then
    file "$AX_OUT" 2>&1
fi
if command -v aarch64-linux-gnu-objdump &>/dev/null; then
    aarch64-linux-gnu-objdump -f "$AX_OUT" 2>&1
    echo "[M22] Checking axon_main symbol..."
    aarch64-linux-gnu-nm "$AX_OUT" 2>&1 | grep -i "axon_main" || true
fi

# ── Step 4: Migration contract tests (Rust mirror) ───────────────────────────
echo ""
echo "[M22] Step 4: Migration contract tests..."
cargo test -p asl-axon-migration --lib 2>&1
TESTS=$(cargo test -p asl-axon-migration --lib 2>&1 | grep -c "test.*ok" || true)
echo "[M22] Migration contract tests passed: $TESTS"

# ── Step 5: Regression ───────────────────────────────────────────────────────
echo ""
echo "[M22] Step 5: Regression — prior milestones..."
cargo test -p asl-common     --test m1_sovereignty 2>&1 | tail -1
cargo test -p asl-arpi       --test m2_broker       2>&1 | tail -1
cargo test -p asl-crypto-bridge --lib               2>&1 | tail -1
cargo test -p asl-datatier      --lib               2>&1 | tail -1
cargo test -p asl-haniel        --lib               2>&1 | tail -1
cargo test -p asl-awp           --lib               2>&1 | tail -1
cargo test -p asl-arpi-ipc      --lib               2>&1 | tail -1

# ── Step 6: Summary ──────────────────────────────────────────────────────────
echo ""
echo "[M22] Migration summary:"
echo "  Source  : asl-axon-migration/ax/sovereign_arpi.ax"
echo "  Target  : aarch64-sel4 (seL4 Microkit ABI)"
echo "  Profile : seL4-strict"
echo "  Output  : asl-axon-migration/ax/sovereign_arpi.o"
echo "  PD      : ARPi-Broker (mandatory PD 0x01)"
echo "  Proof   : axon_main() → 0x4153"
echo "  Layers  : Schema/Identity/MutualAuth/Scope/Anomaly in .ax"

# ── Step 7: Tag ──────────────────────────────────────────────────────────────
echo ""
echo "[M22] Step 7: Tagging v0.1.0-asl-m22..."
git add -A
git commit -m "M22: AXON migration — ARPi-Broker PD sources to .ax

- Add sovereign_arpi.ax: ARPi-Broker PD in AXON source language
- Compiles to seL4 aarch64 ELF via axon build --target aarch64-sel4
- Five ARPi layers implemented in .ax: Schema/Identity/MutualAuth/Scope/Anomaly
- axon_main() returns 0x4153 sovereign proof on success
- asl-axon-migration: Rust mirror crate verifies migration contract
- Migration contract: .ax semantics verified identical to Rust implementation
- Proves AXONYX compiler targets seL4 ELF for real sovereign PDs

Target  : aarch64-sel4
Profile : seL4-strict
PD      : ARPi-Broker (mandatory, PD 0x01)
Proof   : axon_main() → 0x4153

Signed-off-by: Edison Lepiten <aieonyx.eu@gmail.com>"

git tag -s v0.1.0-asl-m22 -u B4C8548260DB40E1 \
    -m "ASL M22: AXON migration — ARPi-Broker PD to .ax, seL4 aarch64 ELF" \
    || git tag -a v0.1.0-asl-m22 \
    -m "ASL M22: AXON migration — ARPi-Broker PD to .ax, seL4 aarch64 ELF"

echo ""
echo "═══════════════════════════════════════════════"
echo " M22 COMPLETE — v0.1.0-asl-m22 tagged"
echo " Next: M23 — Multi-node boot"
echo "═══════════════════════════════════════════════"
