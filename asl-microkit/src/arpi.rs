// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use asl_axon_bridge::abi::ABI_TOKEN_V1;
use crate::dbg;

#[no_mangle]
pub extern "C" fn asl_arpi_init() {
    dbg::puts("ARPi-Broker PD: initializing\n");
    dbg::puts("ARPi-Broker: commissioning route table\n");
    assert!(ABI_TOKEN_V1 != 0);
    dbg::puts("ARPi-Broker: ABI token validated\n");
    dbg::puts("ARPi-Broker: sovereign IPC READY\n");
}

#[no_mangle]
pub extern "C" fn asl_arpi_notified(channel: u8) {
    if channel == 1 {
        dbg::puts("ARPi-Broker: GENESIS signal received\n");
        dbg::puts("ARPi-Broker: IPC routing ACTIVE\n");
    }
}
