/*
 * USB Driver PD — C shim (ASL-M7)
 * SOMA hardware identity read path — USB-A root key
 * Copyright (c) 2026 Edison Lepiten / AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

extern void asl_usb_init(void);
extern void asl_usb_notified(uint8_t ch);

void init(void) { asl_usb_init(); }
void notified(microkit_channel ch) { asl_usb_notified((uint8_t)ch); }
microkit_msginfo protected(microkit_channel ch, microkit_msginfo msginfo) {
    (void)ch; (void)msginfo;
    return microkit_msginfo_new(0, 0);
}
