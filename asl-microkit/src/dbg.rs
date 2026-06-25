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

pub fn put_u8(n: u8) {
    put_u32(n as u32);
}

pub fn put_u32(n: u32) {
    if n == 0 {
        puts("0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 10usize;
    let mut val = n;
    while val > 0 {
        i -= 1;
        buf[i] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    for &b in &buf[i..] {
        extern "C" {
            fn microkit_dbg_putc(c: u8);
        }
        unsafe { microkit_dbg_putc(b) }
    }
}
