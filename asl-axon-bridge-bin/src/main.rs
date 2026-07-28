// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// AXON-Bridge PD — seL4 binary entry point
#![no_std]
#![no_main]

#[repr(C, align(512))]
pub struct SeL4IpcBuffer { pub words: [usize; 64] }

#[no_mangle]
#[used]
pub static mut __sel4_ipc_buffer_obj: SeL4IpcBuffer = SeL4IpcBuffer { words: [0usize; 64] };

#[no_mangle]
#[link_section = ".rodata"]
pub static microkit_name: [u8; 12] = *b"AXON-Bridge\0";

#[no_mangle]
#[link_section = ".data"]
pub static microkit_passive: bool = false;

use core::fmt::Write;
const UART: *mut u8 = 0x09000000 as *mut u8;
const SOVEREIGN_PROOF: u32 = 0x4153;

struct Uart;
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() { unsafe { UART.write_volatile(b); } }
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn init() {
    let mut w = Uart;
    let _ = core::writeln!(w, "[AXON-Bridge] seL4 boot — proof=0x{:X}", SOVEREIGN_PROOF);
    assert_eq!(SOVEREIGN_PROOF, 0x4153);
}

#[no_mangle]
pub extern "C" fn notified(_ch: u8) {}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
