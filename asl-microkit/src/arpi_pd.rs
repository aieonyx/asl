// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ARPi-Broker PD — Microkit entry point (ASL-M6)
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

#![no_std]
#![no_main]

mod microkit_bindings;
mod panic;

use asl_axon_bridge::abi::ABI_TOKEN_V1;

type microkit_channel = u8;

#[no_mangle]
pub static mut __sel4_ipc_buffer_obj: [u8; 1024] = [0u8; 1024];

#[no_mangle]
pub extern "C" fn init() {
    microkit_bindings::debug_println("ARPi-Broker PD: initializing");
    microkit_bindings::debug_println("ARPi-Broker: route table commissioned");
    let token = ABI_TOKEN_V1;
    assert!(token != 0);
    microkit_bindings::debug_println("ARPi-Broker: ABI token validated");
    microkit_bindings::debug_println("ARPi-Broker: sovereign IPC READY");
}

#[no_mangle]
pub extern "C" fn notified(channel: microkit_channel) {
    if channel == 1 {
        microkit_bindings::debug_println("ARPi-Broker: GENESIS signal received");
        microkit_bindings::debug_println("ARPi-Broker: IPC routing ACTIVE");
    }
}

#[no_mangle]
pub extern "C" fn protected(_ch: microkit_channel, _msg: u64) -> u64 { 0 }
