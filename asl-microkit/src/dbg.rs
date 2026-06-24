// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Debug output via microkit_dbg_putc

pub fn puts(s: &str) {
    extern "C" {
        fn microkit_dbg_putc(c: u8);
    }
    for b in s.bytes() {
        unsafe { microkit_dbg_putc(b) }
    }
}
