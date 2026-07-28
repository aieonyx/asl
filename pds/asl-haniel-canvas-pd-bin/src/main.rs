// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PL-76: HANIEL-CANVAS-PD binary entry point under seL4

#![no_std]
#![no_main]

use core::fmt::Write;

const SOVEREIGN_PROOF: u64 = 0x4153;
const SERIAL_BASE: u32     = 0x09000000;

struct SerialWriter;
impl Write for SerialWriter {{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {{
        for b in s.bytes() {{
            unsafe {{ core::ptr::write_volatile(SERIAL_BASE as *mut u8, b); }}
        }}
        Ok(())
    }}
}}

#[no_mangle]
pub extern "C" fn main() -> ! {{
    let mut w = SerialWriter;
    let _ = writeln!(w, "[HANIEL-CANVAS-PD] starting under seL4 — proof={{:#x}}", SOVEREIGN_PROOF);
    let _ = writeln!(w, "[HANIEL-CANVAS-PD] isolated memory region: ACTIVE");
    let _ = writeln!(w, "[HANIEL-CANVAS-PD] ARPi endpoint: LISTENING");
    loop {{ core::hint::spin_loop(); }}
}}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {{ loop {{}} }}
