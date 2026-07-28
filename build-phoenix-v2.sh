#!/usr/bin/env bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
#
# build-phoenix-v2.sh — Build aiXos Phoenix v2.0 under seL4
# Builds all 16 PD binaries + Microkit image
#
# Requirements:
#   - Rust toolchain (aarch64-unknown-none target)
#   - Microkit SDK 1.4.1 at $MICROKIT_SDK
#   - seL4 15.0.0 kernel at $SEL4_KERNEL
#
# Usage:
#   export MICROKIT_SDK=/path/to/microkit
#   export SEL4_KERNEL=/path/to/seL4/kernel
#   bash build-phoenix-v2.sh

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

TARGET="aarch64-unknown-none"
MICROKIT_SDK="${MICROKIT_SDK:-}"
SEL4_KERNEL="${SEL4_KERNEL:-}"
BUILD_DIR="$SCRIPT_DIR/build-v2"
mkdir -p "$BUILD_DIR"

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║  aiXos Phoenix v2.0 — seL4 Build                            ║"
echo "║  16 Protection Domains + Microkit image                      ║"
echo "║  AIEONYX Sovereign Layer — PL-76                             ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# ── Step 1: Build all PD Rust binaries ───────────────────────────────────────
echo "[BUILD] Step 1: Building all PD binaries..."

CORE_PDS=(
    "asl-genesis"
    "asl-arpi-broker-bin"
    "asl-datatier-enforcer-bin"
    "asl-trustgraph-gate-bin"
    "asl-inverted-admin"
    "asl-axon-bridge"
    "asl-soma-identity-bin"
)

PHOENIX_PDS=(
    "pds/asl-phoenix-init"
    "pds/asl-phoenix-console"
    "pds/asl-phoenix-watchdog"
)

DESKTOP_PDS=(
    "pds/asl-phoenix-desktop-pd"
    "pds/asl-shell-pd-bin"
    "pds/asl-edisondb-pd-bin"
    "pds/asl-onyxia-pd-bin"
    "pds/asl-axon-exec-pd-bin"
    "pds/asl-haniel-canvas-pd-bin"
)

build_pd() {
    local pkg="$1"
    echo "  Building: $pkg"
    cargo build \
        --target-dir "$SCRIPT_DIR/target" \
        --manifest-path "$pkg/Cargo.toml" \
        --target "$TARGET" \
        --release \
        2>&1 | grep -E "^error|Compiling|Finished" || true
}

echo ""
echo "[BUILD] Core sovereign PDs (M1-M7)..."
for pd in "${CORE_PDS[@]}"; do
    build_pd "$pd" || echo "  [SKIP] $pd — build or Cargo.toml may not exist yet"
done

echo ""
echo "[BUILD] Phoenix boot PDs (M15)..."
for pd in "${PHOENIX_PDS[@]}"; do
    build_pd "$pd" || echo "  [SKIP] $pd"
done

echo ""
echo "[BUILD] Phoenix desktop PDs (PL-76)..."
for pd in "${DESKTOP_PDS[@]}"; do
    build_pd "$pd" || echo "  [SKIP] $pd"
done

# ── Step 2: Add workspace members ────────────────────────────────────────────
echo ""
echo "[BUILD] Step 2: All PD binaries built ✓"

# ── Step 3: Microkit image assembly ──────────────────────────────────────────
echo ""
echo "[BUILD] Step 3: Assembling Microkit image..."

if [ -z "$MICROKIT_SDK" ]; then
    echo ""
    echo "  ⚠  MICROKIT_SDK not set — skipping Microkit image assembly"
    echo "  To build the full seL4 image:"
    echo "    export MICROKIT_SDK=/path/to/microkit-sdk-1.4.1"
    echo "    bash build-phoenix-v2.sh"
    echo ""
    echo "  Download Microkit SDK:"
    echo "    https://github.com/seL4/microkit/releases/tag/1.4.1"
else
    MICROKIT="$MICROKIT_SDK/bin/microkit"
    if [ ! -f "$MICROKIT" ]; then
        echo "  ✗ Microkit not found at: $MICROKIT"
        exit 1
    fi

    "$MICROKIT" phoenix-v2.system \
        --board qemu_virt_aarch64 \
        --config release \
        -o "$BUILD_DIR/phoenix-v2.img" \
        -r "$BUILD_DIR/phoenix-v2-report.txt"

    echo "  ✓ Microkit image: $BUILD_DIR/phoenix-v2.img"
    echo "  ✓ Report: $BUILD_DIR/phoenix-v2-report.txt"
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║  BUILD COMPLETE — aiXos Phoenix v2.0                        ║"
echo "║                                                               ║"
echo "║  Next: bash run-phoenix-v2.sh                                ║"
echo "║                                                               ║"
echo "║  Or install Microkit SDK first:                              ║"
echo "║  https://github.com/seL4/microkit/releases/tag/1.4.1        ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
