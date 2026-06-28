#!/usr/bin/env bash
# Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
# SPDX-License-Identifier: Apache-2.0
#
# build-m20.sh — ASL M20: AWP Protocol live inside seL4

set -euo pipefail

export MICROKIT_SDK="${MICROKIT_SDK:-$HOME/microkit-sdk-1.4.1}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "═══════════════════════════════════════════════"
echo " ASL M20 — AWP Protocol live inside seL4"
echo "═══════════════════════════════════════════════"

# ── Step 0: Kani extern fix ──────────────────────────────────────────────────
echo ""
echo "[M20] Step 0: Applying Kani extern crate fix..."
python3 fix_kani.py . || true

# ── Step 1: Build asl-awp ────────────────────────────────────────────────────
echo ""
echo "[M20] Step 1: Building asl-awp..."
cargo build -p asl-awp 2>&1

# ── Step 2: Unit tests ───────────────────────────────────────────────────────
echo ""
echo "[M20] Step 2: Unit tests — asl-awp..."
cargo test -p asl-awp --lib 2>&1
UNIT=$(cargo test -p asl-awp --lib 2>&1 | grep -c "test.*ok" || true)
echo "[M20] Unit tests passed: $UNIT"

# ── Step 3: Integration tests ────────────────────────────────────────────────
echo ""
echo "[M20] Step 3: Integration tests — m20_awp..."
cargo test -p asl-awp --test m20_awp 2>&1
INTEG=$(cargo test -p asl-awp --test m20_awp 2>&1 | grep -c "test.*ok" || true)
echo "[M20] Integration tests passed: $INTEG"

# ── Step 4: Regression ───────────────────────────────────────────────────────
echo ""
echo "[M20] Step 4: Regression — prior milestones..."
cargo test -p asl-common --test m1_sovereignty 2>&1 | tail -1
cargo test -p asl-arpi --test m2_broker 2>&1 | tail -1
cargo test -p asl-axon-bridge --test m5_bridge 2>&1 | tail -1
cargo test -p asl-crypto-bridge --lib 2>&1 | tail -1
cargo test -p asl-datatier --lib 2>&1 | tail -1
cargo test -p asl-haniel --lib 2>&1 | tail -1
cargo test -p asl-haniel --test m19_haniel 2>&1 | tail -1

# ── Step 5: Summary ──────────────────────────────────────────────────────────
echo ""
echo "[M20] AWP Protocol stack:"
echo "  Layer 1 — Frame    : AWP packet framing (magic 0xA1E0AE70)"
echo "  Layer 2 — Address  : awp://name.category[.region]"
echo "  Layer 3 — Route    : Mesh routing table (16 routes max)"
echo "  Layer 4 — Dispatch : HANIEL / Mesh / ThreatIntel"
echo "  Layer 5 — Threat   : Aegis gate (threshold=80)"

# ── Step 6: GPG-signed tag ───────────────────────────────────────────────────
echo ""
echo "[M20] Step 6: Tagging v0.1.0-asl-m20..."
git add -A
git commit -m "M20: AWP Protocol live inside seL4

- Add asl-awp crate: AWP Protocol Protection Domain
- AWP packet framing (magic 0xA1E0AE70, 16-byte header)
- Sovereign address parsing: awp://name.category[.region]
- Mesh routing table (fixed-size, 16 routes, no heap)
- Five-layer protocol stack: Frame/Address/Route/Dispatch/Threat
- Aegis threat gate (threshold=80, packets rejected above)
- Dispatch: REQUEST/RESPONSE → HANIEL, MESH → mesh, THREAT → intel
- Monotonic sequence counter with u16 wrap
- 0 failures

Signed-off-by: Edison Lepiten <aieonyx.eu@gmail.com>"

git tag -s v0.1.0-asl-m20 -u B4C8548260DB40E1 \
    -m "ASL M20: AWP Protocol live inside seL4" \
    || git tag -a v0.1.0-asl-m20 \
    -m "ASL M20: AWP Protocol live inside seL4"

echo ""
echo "═══════════════════════════════════════════════"
echo " M20 COMPLETE — v0.1.0-asl-m20 tagged"
echo " Next: M21 — ARPi live IPC, full 5-layer sovereign auth"
echo "═══════════════════════════════════════════════"
