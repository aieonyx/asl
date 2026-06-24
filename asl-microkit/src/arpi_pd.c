/*
 * ARPi-Broker PD — C shim for Microkit (ASL-M6)
 * Copyright (c) 2026 Edison Lepiten / AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

extern void asl_arpi_init(void);
extern void asl_arpi_notified(uint8_t channel);

void init(void) {
    asl_arpi_init();
}

void notified(microkit_channel ch) {
    asl_arpi_notified((uint8_t)ch);
}

microkit_msginfo protected(microkit_channel ch, microkit_msginfo msginfo) {
    (void)ch; (void)msginfo;
    return microkit_msginfo_new(0, 0);
}
