#!/bin/bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
# build-m12.sh — ASL-M12: EdisonDB PD
set -e

export MICROKIT_SDK=${MICROKIT_SDK:-~/microkit-sdk-1.4.1}
BOARD=qemu_virt_aarch64
CONFIG=debug
SDK_DIR=$MICROKIT_SDK/board/$BOARD/$CONFIG
BUILD_DIR=build/m12
TARGET=aarch64-unknown-none
CC=aarch64-linux-gnu-gcc

mkdir -p $BUILD_DIR

echo "==> ASL-M12: Building 9 sovereign PDs + EdisonDB"

if [[ "$1" != "run" ]]; then
    axon build \
        --profile seL4-strict \
        --target aarch64-sel4 \
        -o $BUILD_DIR/asl_runtime \
        asl-microkit/axon/asl_runtime.ax 2>&1

    aarch64-linux-gnu-objcopy \
        --globalize-symbol=axon_main \
        --globalize-symbol=validate_tiers \
        --globalize-symbol=validate_arpi \
        --globalize-symbol=validate_soma \
        $BUILD_DIR/asl_runtime.o \
        $BUILD_DIR/asl_runtime_global.o

    cargo build \
        --package asl-microkit \
        --lib \
        --target $TARGET \
        --release 2>&1

    RUST_LIB="target/$TARGET/release/libasl_microkit.a"

    build_pd() {
        local PD_NAME=$1
        local C_SHIM=$2
        local EXTRA_OBJ=${3:-}
        echo "--> Building $PD_NAME..."
        $CC -nostdlib -ffreestanding \
            -I${SDK_DIR}/include -I${SDK_DIR}/include/sel4 \
            -c asl-microkit/src/${C_SHIM} \
            -o $BUILD_DIR/${PD_NAME}_shim.o
        $CC -nostdlib \
            -T${SDK_DIR}/lib/microkit.ld \
            -L${SDK_DIR}/lib \
            $BUILD_DIR/${PD_NAME}_shim.o \
            $EXTRA_OBJ \
            $RUST_LIB -lmicrokit -lgcc \
            -o $BUILD_DIR/${PD_NAME}.elf
        echo "    OK: $BUILD_DIR/${PD_NAME}.elf"
    }

    build_pd "asl-genesis-pd"     "genesis_pd.c"
    build_pd "asl-arpi-pd"        "arpi_pd.c"
    build_pd "asl-input-pd"       "input_pd.c"
    build_pd "asl-storage-pd"     "storage_pd.c"
    build_pd "asl-usb-pd"         "usb_pd.c"
    build_pd "asl-network-pd"     "network_pd.c"
    build_pd "asl-haniel-pd"      "haniel_pd.c"
    build_pd "asl-edisondb-pd"    "edisondb_pd.c"
    build_pd "asl-axon-bridge-pd" "axon_bridge_pd.c" \
             "$BUILD_DIR/asl_runtime_global.o"

    echo ""
    echo "--> Running Microkit tool..."
    $MICROKIT_SDK/bin/microkit \
        asl-microkit/system/asl_m12.system \
        --search-path $BUILD_DIR \
        --board $BOARD --config $CONFIG \
        -o $BUILD_DIR/asl_m12.img \
        -r $BUILD_DIR/asl_m12_report.txt
    echo "    OK: $BUILD_DIR/asl_m12.img"
fi

if [[ "$1" == "run" || "$1" == "all" ]]; then
    echo ""
    echo "==> Booting ASL-M12 on QEMU aarch64..."
    timeout 15 qemu-system-aarch64 \
        -machine virt,virtualization=on,highmem=off \
        -cpu cortex-a53 -m 2G -nographic -smp 4 \
        -device loader,file=${BUILD_DIR}/asl_m12.img,addr=0x70000000,cpu-num=0 \
        2>&1 || true
    echo ""
    echo "==> ASL-M12 boot complete."
fi
