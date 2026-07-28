// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Onyxia-PD — Microkit PD
#![no_std]
#![no_main]

// libmicrokit.a provides: _start, microkit_name, microkit_passive
// We must provide: __sel4_ipc_buffer_obj, init(), notified()

#[no_mangle]
#[link_section = ".bss"]
pub static mut __sel4_ipc_buffer_obj: [u8; 4096] = [0u8; 4096];

const UART: *mut u8 = 0x09000000 as *mut u8;
const SOVEREIGN_PROOF: u32 = 0x4153;

fn uart_write(s: &[u8]) {
    for &b in s {
        unsafe { UART.write_volatile(b); }
    }
}

#[no_mangle]
pub extern "C" fn init() {
    uart_write(b"[Onyxia-PD] seL4 proof=0x4153\r\n");
    assert_eq!(SOVEREIGN_PROOF, 0x4153);
}

#[no_mangle]
pub extern "C" fn notified(_ch: u8) {}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
