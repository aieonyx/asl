/*
 * HANIEL Engine PD — C shim (ASL-M11)
 * First sovereign display surface on seL4.
 * Copyright (c) 2026 Edison Lepiten / AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

extern void asl_haniel_init(void);
extern void asl_haniel_notified(uint8_t ch);
extern uint64_t asl_haniel_protected(uint8_t ch, uint64_t msginfo);

void init(void) { asl_haniel_init(); }
void notified(microkit_channel ch) { asl_haniel_notified((uint8_t)ch); }
microkit_msginfo protected(microkit_channel ch, microkit_msginfo msginfo) {
    uint64_t raw = msginfo.words[0];
    uint64_t result = asl_haniel_protected((uint8_t)ch, raw);
    return microkit_msginfo_new(result, 0);
}
