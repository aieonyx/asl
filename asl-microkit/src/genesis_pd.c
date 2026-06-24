/*
 * GENESIS PD — C shim for Microkit (ASL-M6)
 * Calls Rust sovereignty logic from asl-genesis-pd lib.
 * Copyright (c) 2026 Edison Lepiten / AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

/* Rust sovereignty functions from asl-genesis-pd */
extern void asl_genesis_init(void);
extern void asl_genesis_notified(uint8_t channel);

void init(void) {
    asl_genesis_init();
}

void notified(microkit_channel ch) {
    asl_genesis_notified((uint8_t)ch);
}

microkit_msginfo protected(microkit_channel ch, microkit_msginfo msginfo) {
    (void)ch; (void)msginfo;
    return microkit_msginfo_new(0, 0);
}
