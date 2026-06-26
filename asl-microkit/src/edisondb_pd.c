/*
 * EdisonDB PD — C shim (ASL-M12)
 * Sovereign data tier: WAL+MVCC + DataTier enforcement
 * Copyright (c) 2026 Edison Lepiten / AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

extern void asl_edisondb_init(void);
extern void asl_edisondb_notified(uint8_t ch);
extern uint64_t asl_edisondb_protected(uint8_t ch, uint64_t msginfo);

void init(void) { asl_edisondb_init(); }
void notified(microkit_channel ch) { asl_edisondb_notified((uint8_t)ch); }
microkit_msginfo protected(microkit_channel ch, microkit_msginfo msginfo) {
    uint64_t raw = msginfo.words[0];
    uint64_t result = asl_edisondb_protected((uint8_t)ch, raw);
    return microkit_msginfo_new(result, 0);
}
