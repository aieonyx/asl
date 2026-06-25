/*
 * AXON-Bridge PD — C shim (ASL-M9)
 * Loads and executes AXON userspace runtime.
 * Copyright (c) 2026 Edison Lepiten / AIEONYX
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <microkit.h>

/* AXON compiled function — from asl_runtime.o */
extern int64_t axon_main(void);

/* Rust bridge validation */
extern void asl_axon_bridge_init(void);

/* int64 to decimal string */
static void put_i64(int64_t n) {
    if (n == 0) { microkit_dbg_putc('0'); return; }
    char tmp[20]; int i = 0;
    while (n > 0) { tmp[i++] = '0' + (n % 10); n /= 10; }
    while (i > 0) { microkit_dbg_putc(tmp[--i]); }
}

/* hex output */
static void put_hex(int64_t n) {
    const char *h = "0123456789ABCDEF";
    microkit_dbg_puts("0x");
    for (int i = 60; i >= 0; i -= 4) {
        microkit_dbg_putc(h[(n >> i) & 0xF]);
    }
}

void init(void) {
    microkit_dbg_puts("AXON-Bridge PD: initializing\n");
    asl_axon_bridge_init();
    microkit_dbg_puts("AXON-Bridge: loading AXON userspace runtime\n");
    microkit_dbg_puts("AXON-Bridge: calling axon_main()\n");

    int64_t result = axon_main();

    microkit_dbg_puts("AXON-Bridge: axon_main() returned: ");
    put_i64(result);
    microkit_dbg_puts(" (");
    put_hex(result);
    microkit_dbg_puts(")\n");

    if (result == 16723) {
        microkit_dbg_puts("AXON-Bridge: SOVEREIGN PROOF VERIFIED\n");
        microkit_dbg_puts("AXON-Bridge: 0x4153 = ASCII 'AS' = AIEONYX Sovereign\n");
        microkit_dbg_puts("AXON-Bridge: AXON userspace runtime VALIDATED on seL4\n");
    } else {
        microkit_dbg_puts("AXON-Bridge: WARNING — unexpected result\n");
    }
}

void notified(microkit_channel ch) { (void)ch; }
microkit_msginfo protected(microkit_channel ch, microkit_msginfo msginfo) {
    (void)ch; (void)msginfo;
    return microkit_msginfo_new(0, 0);
}
