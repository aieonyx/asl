#!/usr/bin/env bash
# Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
# SPDX-License-Identifier: Apache-2.0
#
# build-m21.sh — ASL M21: ARPi Live IPC, full 5-layer sovereign auth

set -euo pipefail

export MICROKIT_SDK="${MICROKIT_SDK:-$HOME/microkit-sdk-1.4.1}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "═══════════════════════════════════════════════"
echo " ASL M21 — ARPi Live IPC, Sovereign Auth"
echo "═══════════════════════════════════════════════"

# ── Step 0: Kani extern fix ──────────────────────────────────────────────────
echo ""
echo "[M21] Step 0: Applying Kani extern crate fix..."
python3 fix_kani.py . || true

# ── Step 1: Build ────────────────────────────────────────────────────────────
echo ""
echo "[M21] Step 1: Building asl-arpi-ipc..."
cargo build -p asl-arpi-ipc 2>&1

# ── Step 2: Unit tests ───────────────────────────────────────────────────────
echo ""
echo "[M21] Step 2: Unit tests — asl-arpi-ipc..."
cargo test -p asl-arpi-ipc --lib 2>&1
UNIT=$(cargo test -p asl-arpi-ipc --lib 2>&1 | grep -c "test.*ok" || true)
echo "[M21] Unit tests passed: $UNIT"

# ── Step 3: Integration tests ────────────────────────────────────────────────
echo ""
echo "[M21] Step 3: Integration tests — m21_arpi_ipc..."
cargo test -p asl-arpi-ipc --test m21_arpi_ipc 2>&1
INTEG=$(cargo test -p asl-arpi-ipc --test m21_arpi_ipc 2>&1 | grep -c "test.*ok" || true)
echo "[M21] Integration tests passed: $INTEG"

# ── Step 4: Regression ───────────────────────────────────────────────────────
echo ""
echo "[M21] Step 4: Regression — prior milestones..."
cargo test -p asl-common  --test m1_sovereignty 2>&1 | tail -1
cargo test -p asl-arpi    --test m2_broker       2>&1 | tail -1
cargo test -p asl-axon-bridge --test m5_bridge   2>&1 | tail -1
cargo test -p asl-crypto-bridge --lib            2>&1 | tail -1
cargo test -p asl-datatier  --lib                2>&1 | tail -1
cargo test -p asl-haniel    --lib                2>&1 | tail -1
cargo test -p asl-haniel    --test m19_haniel    2>&1 | tail -1
cargo test -p asl-awp       --lib                2>&1 | tail -1
cargo test -p asl-awp       --test m20_awp       2>&1 | tail -1

# ── Step 5: Summary ──────────────────────────────────────────────────────────
echo ""
echo "[M21] ARPi five-layer bind sequence:"
echo "  Layer 1 — Schema      : AXON message contract validation"
echo "  Layer 2 — Identity    : Ed25519 commissioning keypair"
echo "  Layer 3 — Mutual Auth : Both endpoints verified, no self-bind"
echo "  Layer 4 — Scope       : Monotonic capability token"
echo "  Layer 5 — Anomaly     : Aegis gate (threshold=75)"
echo ""
echo "[M21] Bind log: every event recorded (no silent failure path)"
echo "[M21] Header:   78-byte ARPi provenance prepended to all bound messages"

# ── Step 6: Tag ──────────────────────────────────────────────────────────────
echo ""
echo "[M21] Step 6: Tagging v0.1.0-asl-m21..."
git add -A
git commit -m "M21: ARPi live IPC — full 5-layer sovereign auth

- Add asl-arpi-ipc crate: ARPi Protocol PD (AXON Receptor Protocol Interface)
- Five-layer bind engine: Schema/Identity/MutualAuth/Scope/Anomaly
- Ed25519 identity proof (structural verification, full crypto at M22)
- Monotonic capability token enforcement (replay protection)
- Aegis anomaly gate (threshold=75, Flagged=50-74, Escalated=75+)
- 78-byte ARPi provenance header on every bound message
- Bind log: every attempt recorded — no silent failure path
- Self-bind rejected (same keypair cannot bind to itself)
- 0 failures

Signed-off-by: Edison Lepiten <aieonyx.eu@gmail.com>"

git tag -s v0.1.0-asl-m21 -u B4C8548260DB40E1 \
    -m "ASL M21: ARPi live IPC — full 5-layer sovereign auth" \
    || git tag -a v0.1.0-asl-m21 \
    -m "ASL M21: ARPi live IPC — full 5-layer sovereign auth"

echo ""
echo "═══════════════════════════════════════════════"
echo " M21 COMPLETE — v0.1.0-asl-m21 tagged"
echo " Next: M22 — AXON migration, PD sources to .ax"
echo "═══════════════════════════════════════════════"
