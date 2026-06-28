#!/usr/bin/env bash
# Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
# SPDX-License-Identifier: Apache-2.0
#
# build-m23.sh — ASL M23: Multi-node boot, Phoenix Desktop sovereign ready

set -euo pipefail

export MICROKIT_SDK="${MICROKIT_SDK:-$HOME/microkit-sdk-1.4.1}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "═══════════════════════════════════════════════"
echo " ASL M23 — Multi-Node Boot: Phoenix Desktop"
echo "═══════════════════════════════════════════════"

# ── Step 0: Kani extern fix ──────────────────────────────────────────────────
echo ""
echo "[M23] Step 0: Applying Kani extern crate fix..."
python3 fix_kani.py . || true

# ── Step 1: Build ────────────────────────────────────────────────────────────
echo ""
echo "[M23] Step 1: Building asl-multinode..."
cargo build -p asl-multinode 2>&1

# ── Step 2: Unit tests ───────────────────────────────────────────────────────
echo ""
echo "[M23] Step 2: Unit tests — asl-multinode..."
cargo test -p asl-multinode --lib 2>&1
UNIT=$(cargo test -p asl-multinode --lib 2>&1 | grep -c "test.*ok" || true)
echo "[M23] Unit tests passed: $UNIT"

# ── Step 3: Integration tests ────────────────────────────────────────────────
echo ""
echo "[M23] Step 3: Integration tests — m23_multinode..."
cargo test -p asl-multinode --test m23_multinode 2>&1
INTEG=$(cargo test -p asl-multinode --test m23_multinode 2>&1 | grep -c "test.*ok" || true)
echo "[M23] Integration tests passed: $INTEG"

# ── Step 4: Phoenix boot demo ────────────────────────────────────────────────
echo ""
echo "[M23] Step 4: Running Phoenix desktop boot demo..."
bash demo-m23.sh

# ── Step 5: Regression ───────────────────────────────────────────────────────
echo ""
echo "[M23] Step 5: Regression — prior milestones..."
cargo test -p asl-common   --test m1_sovereignty 2>&1 | tail -1
cargo test -p asl-arpi     --test m2_broker       2>&1 | tail -1
cargo test -p asl-haniel   --lib                  2>&1 | tail -1
cargo test -p asl-awp      --lib                  2>&1 | tail -1
cargo test -p asl-arpi-ipc --lib                  2>&1 | tail -1

# ── Step 6: Summary ──────────────────────────────────────────────────────────
echo ""
echo "[M23] Phoenix Desktop topology:"
echo "  Node 0 PRIMARY    : Phoenix OS + 6 mandatory PDs"
echo "  Node 1 BASTION    : ARPi + TrustGraph + SOMA"
echo "  Node 2 DATASTORE  : DataTier-Enforcer + EdisonDB"
echo "  Node 3 RENDERER   : HANIEL 1280×720 + AXON-Bridge"
echo "  Channels          : 3 ARPi inter-node (star topology)"
echo "  Sovereign proof   : 0x4153 on all nodes"

# ── Step 7: Tag ──────────────────────────────────────────────────────────────
echo ""
echo "[M23] Step 7: Tagging v0.1.0-asl-m23..."
git add -A
git commit -m "M23: Multi-node boot — Phoenix Desktop sovereign ready

- Add asl-multinode crate: sovereign mesh boot coordinator
- Phoenix desktop topology: 4 nodes (PRIMARY/BASTION/DATASTORE/RENDERER)
- Five-phase boot: Offline→Booting→Proving→PdsReady→MeshLinked→Online
- axon_main() → 0x4153 verified on all 4 nodes
- 13 PDs distributed across 4 nodes
- 3 inter-node ARPi channels (star topology, primary↔all)
- demo-m23.sh: reproducible Phoenix desktop boot demo
- 0 failures

Phoenix Desktop stack:
  seL4 15.0.0 / Microkit SDK 1.4.1
  AXONYX compiler (1606+ tests)
  ARPi 5-layer sovereign auth
  AWP sovereign network protocol
  HANIEL render surface 1280×720 ARGB8888
  EdisonDB WAL+MVCC AES-256-GCM

Signed-off-by: Edison Lepiten <aieonyx.eu@gmail.com>"

git tag -s v0.1.0-asl-m23 -u B4C8548260DB40E1 \
    -m "ASL M23: Multi-node boot — Phoenix Desktop sovereign ready" \
    || git tag -a v0.1.0-asl-m23 \
    -m "ASL M23: Multi-node boot — Phoenix Desktop sovereign ready"

echo ""
echo "═══════════════════════════════════════════════"
echo " M23 COMPLETE — v0.1.0-asl-m23 tagged"
echo " Next: M24 — ASL v1.0 release"
echo "═══════════════════════════════════════════════"
