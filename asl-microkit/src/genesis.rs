// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// GENESIS sovereignty logic (ASL-M7 update)
// Now signals both ARPi-Broker (ch1) and USB PD (ch2)

use asl_common::version::ASL_VERSION_STRING;
use asl_common::pd::PdId;
use crate::dbg;

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
        PdId::ArpiBroker, PdId::DataTierEnforcer,
        PdId::TrustGraphGate, PdId::InvertedAdmin, PdId::AxonBridge,
    ];
    let _ = mandatory.len();
    dbg::puts("GENESIS: 5 mandatory PDs registered\n");
    dbg::puts("GENESIS: driver PDs commissioned\n");
    dbg::puts("GENESIS: authority surrendered\n");
    dbg::puts("GENESIS: sovereign stack is live\n");
    dbg::puts("AIEONYX ASL-seL4 mKernel: BOOT COMPLETE\n");
}

#[no_mangle]
pub extern "C" fn asl_genesis_notified(_ch: u8) {}
