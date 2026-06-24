#!/bin/bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
#
# build-asl.sh — ASL-seL4 mKernel build script
# Usage: ./build-asl.sh [profile] [arch]
# Profiles: desktop (default), mobile, iot, server, router
# Arch: aarch64 (default), x86_64

set -e

PROFILE=${1:-desktop}
ARCH=${2:-aarch64}

case $ARCH in
    aarch64) TARGET="aarch64-unknown-none" ;;
    x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
    *) echo "Unknown arch: $ARCH"; exit 1 ;;
esac

echo "==> Building ASL-seL4 mKernel"
echo "    Profile : $PROFILE"
echo "    Arch    : $ARCH ($TARGET)"
echo "    Version : ASL v0.1.0 [seL4 15.0.0]"
echo ""

cargo build \
    --target "$TARGET" \
    --package asl-genesis \
    --features "$PROFILE" \
    "$@"

echo ""
echo "==> Build complete."
echo "    Copyright (c) 2026 Edison Lepiten / AIEONYX"
