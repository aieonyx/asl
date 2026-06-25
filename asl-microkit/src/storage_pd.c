/*
 * Storage Driver PD — C shim (ASL-M7)
 * Copyright (c) 2026 Edison Lepiten / AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

extern void asl_storage_init(void);
extern void asl_storage_notified(uint8_t ch);
extern uint64_t asl_storage_protected(uint8_t ch, uint64_t msginfo);

void init(void) { asl_storage_init(); }
void notified(microkit_channel ch) { asl_storage_notified((uint8_t)ch); }
microkit_msginfo protected(microkit_channel ch, microkit_msginfo msginfo) {
    /* Extract raw word from msginfo struct for Rust dispatch */
    uint64_t raw = msginfo.words[0];
    uint64_t result = asl_storage_protected((uint8_t)ch, raw);
    return microkit_msginfo_new(result, 0);
}
