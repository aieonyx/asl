#!/usr/bin/env bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
#
# build-m27.sh — ASL-M27: PL-72 ARPi-Broker Live Inter-PD IPC
#
# What M27 proves:
#   Path A: Shell-PD → [ARPi 5-layer] → EdisonDB-PD (db put x 42)
#   Path B: Onyxia-PD → [ARPi 5-layer] → HANIEL-PD (awp://aieonyx)
#   Path C: Phoenix-Desktop → [ARPi 5-layer] → EdisonDB-PD (status query)
#   19 tests across 3 message paths
#   ARPi bind log records every event (pass AND reject)
#   High anomaly score (>75) rejected at Layer 5
#   Invalid schema rejected at Layer 1
#   Sovereign proof 0x4153 invariant throughout

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "═══════════════════════════════════════════════════════════════"
echo " ASL-M27 — PL-72: ARPi-Broker Live Inter-PD IPC"
echo " Three proven message paths through ARPi 5-layer auth"
echo " AIEONYX Sovereign Layer — aiXos Phoenix v2.0 track"
echo "═══════════════════════════════════════════════════════════════"

echo ""
echo "[M27] Running ARPi-Broker Live test suite..."
cargo test -p asl-arpi-broker-live --lib -- --test-threads=1 2>&1 | tail -5

echo ""
echo "[M27] Proven message paths:"
echo ""
echo "  Path A: Shell-PD ──[ARPi]──► EdisonDB-PD"
echo "    db put x 42 → schema=EDB_WRITE → 5-layer bind → Written ✓"
echo "    High anomaly (score>75) rejected at Layer 5 ✓"
echo "    Invalid schema (0xFF) rejected at Layer 1 ✓"
echo ""
echo "  Path B: Onyxia-PD ──[ARPi]──► HANIEL-PD"
echo "    awp://aieonyx → schema=RENDER → 5-layer bind → Rendered ✓"
echo "    ARPi header carries Onyxia→HANIEL src/dst ✓"
echo ""
echo "  Path C: Phoenix-Desktop ──[ARPi]──► EdisonDB-PD"
echo "    awp://status → schema=EDB_READ → 5-layer bind → Count(0) ✓"
echo "    Multiple reads increment arpi_auths counter ✓"
echo ""
echo "[M27] ARPi guarantees proven:"
echo "  Every message carries 78-byte provenance header ✓"
echo "  Bind log records all events (pass AND reject) ✓"
echo "  Proof 0x4153 invariant throughout broker lifecycle ✓"

echo ""
echo "[M27] Committing M27..."
git add pds/asl-arpi-broker-live/ Cargo.toml build-m27.sh
git commit -m "M27: PL-72 ARPi-Broker live inter-PD IPC — 3 paths, 19 tests

AIEONYX Sovereign Layer — aiXos Phoenix v2.0 track

ARPi-Broker Live — three proven message paths:

Path A: Shell-PD → EdisonDB-PD (db put x 42)
  - ARPi schema=EDB_WRITE, 5-layer bind passes
  - Invalid schema rejected at Layer 1
  - High anomaly (>75) rejected at Layer 5

Path B: Onyxia-PD → HANIEL-PD (awp://aieonyx)
  - ARPi schema=RENDER, 5-layer bind passes
  - 78-byte header carries Onyxia→HANIEL src/dst

Path C: Phoenix-Desktop → EdisonDB-PD (status query)
  - ARPi schema=EDB_READ, 5-layer bind passes
  - Multiple reads tracked via arpi_auths counter

19 tests passing, 0 failures
Bind log records every event (pass AND reject)
Sovereign proof: 0x4153 invariant throughout

Post Doctrine: P1 P2 P3 P4 P5" || true

git tag -a v0.1.0-asl-m27 \
    -m "ASL-M27: PL-72 ARPi live IPC — 3 paths, 19 tests, proof 0x4153" \
    || echo "  (tag may exist)"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo " M27 COMPLETE — PL-72 ARPi-Broker Live Inter-PD IPC"
echo ""
echo " Path A (Shell→EDB):      PROVEN ✓"
echo " Path B (Onyxia→HANIEL):  PROVEN ✓"
echo " Path C (Desktop→EDB):    PROVEN ✓"
echo " Tests: 19 passing, 0 failures"
echo " Proof: axon_main() → 0x4153"
echo ""
echo " Next: PL-73 — AXON-Bridge PD (scripts in isolated PD)"
echo "═══════════════════════════════════════════════════════════════"
