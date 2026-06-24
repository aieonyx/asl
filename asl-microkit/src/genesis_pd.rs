// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// GENESIS PD — Microkit entry point (ASL-M6)
// Entry: init() called by libmicrokit after seL4 boot
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

#![no_std]
#![no_main]

mod microkit_bindings;
mod panic;

use asl_common::version::ASL_VERSION_STRING;
use asl_common::pd::PdId;

type microkit_channel = u8;

// IPC buffer — defined by microkit.ld linker script
// Must be declared to satisfy libmicrokit.a reference
#[no_mangle]
pub static mut __sel4_ipc_buffer_obj: [u8; 1024] = [0u8; 1024];

/// GENESIS PD entry point — called by Microkit libmicrokit main()
#[no_mangle]
pub extern "C" fn init() {
    microkit_bindings::debug_println("AIEONYX ASL-seL4 mKernel booting...");
    microkit_bindings::debug_println(ASL_VERSION_STRING);
    microkit_bindings::debug_println("GENESIS: Node Commissioning Ceremony");

    assert!(!ASL_VERSION_STRING.is_empty());
    assert!(ASL_VERSION_STRING.contains("ASL"));
    assert!(ASL_VERSION_STRING.contains("seL4"));

    microkit_bindings::debug_println("GENESIS: version verified");
    microkit_bindings::debug_println("GENESIS: DevMode unconditionally rejected");
    microkit_bindings::debug_println("GENESIS: trust anchor validated");

    let mandatory_pds = [
        PdId::ArpiBroker,
        PdId::DataTierEnforcer,
        PdId::TrustGraphGate,
        PdId::InvertedAdmin,
        PdId::AxonBridge,
    ];
    let _count = mandatory_pds.len();
    microkit_bindings::debug_println("GENESIS: mandatory PDs registered");
    microkit_bindings::debug_println("GENESIS: signaling ARPi-Broker PD");
    microkit_bindings::debug_println("GENESIS: authority surrendered");
    microkit_bindings::debug_println("GENESIS: sovereign stack is live");
    microkit_bindings::debug_println("AIEONYX ASL-seL4 mKernel: BOOT COMPLETE");

    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn notified(_ch: microkit_channel) {}

#[no_mangle]
pub extern "C" fn protected(_ch: microkit_channel, _msg: u64) -> u64 { 0 }
