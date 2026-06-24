// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Microkit bindings — correct symbols from libmicrokit.a + microkit.h

#![allow(non_snake_case)]

/// Write a single character via microkit_dbg_putc (in libmicrokit.a)
pub fn debug_putc(c: u8) {
    extern "C" {
        fn microkit_dbg_putc(c: u8);
    }
    unsafe { microkit_dbg_putc(c) }
}

/// Write a string via microkit_dbg_puts (in libmicrokit.a)
pub fn debug_println(s: &str) {
    extern "C" {
        fn microkit_dbg_puts(s: *const u8);
    }
    // microkit_dbg_puts expects null-terminated string
    // We write char by char via putc to avoid allocation
    for b in s.bytes() {
        debug_putc(b);
    }
    debug_putc(b'\n');
}

pub fn debug_print(s: &str) {
    for b in s.bytes() {
        debug_putc(b);
    }
}
