#!/bin/bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
# build-m8.sh — ASL-M8: Network Driver PD
set -e

export MICROKIT_SDK=${MICROKIT_SDK:-~/microkit-sdk-1.4.1}
BOARD=qemu_virt_aarch64
CONFIG=debug
SDK_DIR=$MICROKIT_SDK/board/$BOARD/$CONFIG
BUILD_DIR=build/m8
TARGET=aarch64-unknown-none
CC=aarch64-linux-gnu-gcc

mkdir -p $BUILD_DIR

echo "==> ASL-M8: Building six sovereign PD binaries"

cargo build \
    --package asl-microkit \
    --lib \
    --target $TARGET \
    --release 2>&1

RUST_LIB="target/$TARGET/release/libasl_microkit.a"

build_pd() {
    local PD_NAME=$1
    local C_SHIM=$2
    echo "--> Building $PD_NAME..."
    $CC -nostdlib -ffreestanding \
        -I${SDK_DIR}/include -I${SDK_DIR}/include/sel4 \
        -c asl-microkit/src/${C_SHIM} \
        -o $BUILD_DIR/${PD_NAME}_shim.o
    $CC -nostdlib \
        -T${SDK_DIR}/lib/microkit.ld \
        -L${SDK_DIR}/lib \
        $BUILD_DIR/${PD_NAME}_shim.o \
        $RUST_LIB -lmicrokit -lgcc \
        -o $BUILD_DIR/${PD_NAME}.elf
    echo "    OK: $BUILD_DIR/${PD_NAME}.elf"
}

if [[ "$1" == "build" || "$1" == "all" || "$1" == "" ]]; then
    build_pd "asl-genesis-pd"  "genesis_pd.c"
    build_pd "asl-arpi-pd"     "arpi_pd.c"
    build_pd "asl-input-pd"    "input_pd.c"
    build_pd "asl-storage-pd"  "storage_pd.c"
    build_pd "asl-usb-pd"      "usb_pd.c"
    build_pd "asl-network-pd"  "network_pd.c"

    echo ""
    echo "--> Running Microkit tool..."
    $MICROKIT_SDK/bin/microkit \
        asl-microkit/system/asl_m8.system \
        --search-path $BUILD_DIR \
        --board $BOARD --config $CONFIG \
        -o $BUILD_DIR/asl_m8.img \
        -r $BUILD_DIR/asl_m8_report.txt
    echo "    OK: $BUILD_DIR/asl_m8.img"
fi

if [[ "$1" == "run" || "$1" == "all" ]]; then
    echo ""
    echo "==> Booting ASL-M8 on QEMU aarch64..."
    timeout 10 qemu-system-aarch64 \
        -machine virt,virtualization=on,highmem=off \
        -cpu cortex-a53 -m 2G -nographic \
        -device loader,file=${BUILD_DIR}/asl_m8.img,addr=0x70000000,cpu-num=0 \
        2>&1 || true
    echo ""
    echo "==> Boot sequence complete."
fi
