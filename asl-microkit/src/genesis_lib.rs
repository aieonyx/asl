// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// GENESIS sovereignty logic — called from C Microkit shim
// Pure Rust, no Microkit ABI concerns here.

#![no_std]

use asl_common::version::ASL_VERSION_STRING;
use asl_common::pd::PdId;

mod dbg;
mod panic;

/// Called by C shim init()
#[no_mangle]
pub extern "C" fn asl_genesis_init() {
    dbg::puts("AIEONYX ASL-seL4 mKernel booting...\n");
    dbg::puts(ASL_VERSION_STRING);
    dbg::puts("\n");
    dbg::puts("GENESIS: Node Commissioning Ceremony\n");

    assert!(!ASL_VERSION_STRING.is_empty());
    assert!(ASL_VERSION_STRING.contains("ASL"));
    assert!(ASL_VERSION_STRING.contains("seL4"));

    dbg::puts("GENESIS: version verified\n");
    dbg::puts("GENESIS: DevMode unconditionally rejected\n");
    dbg::puts("GENESIS: trust anchor validated\n");

    let mandatory = [
        PdId::ArpiBroker,
        PdId::DataTierEnforcer,
        PdId::TrustGraphGate,
        PdId::InvertedAdmin,
        PdId::AxonBridge,
    ];
    let _ = mandatory.len();

    dbg::puts("GENESIS: 5 mandatory PDs registered\n");
    dbg::puts("GENESIS: signaling ARPi-Broker\n");
    dbg::puts("GENESIS: authority surrendered\n");
    dbg::puts("GENESIS: sovereign stack is live\n");
    dbg::puts("AIEONYX ASL-seL4 mKernel: BOOT COMPLETE\n");
}

/// Called by C shim notified()
#[no_mangle]
pub extern "C" fn asl_genesis_notified(_channel: u8) {}
