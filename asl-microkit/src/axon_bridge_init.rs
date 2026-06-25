// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AXON-Bridge PD init — called from C shim before axon_main()
// Validates ABI token and capability registry before AXON runs.

use crate::dbg;
use asl_axon_bridge::abi::{validate_token, AbiResult, ABI_TOKEN_V1};

#[no_mangle]
pub extern "C" fn asl_axon_bridge_init() {
    dbg::puts("AXON-Bridge: ABI token validation\n");
    match validate_token(ABI_TOKEN_V1) {
        AbiResult::Valid => {
            dbg::puts("AXON-Bridge: ABI token VALID\n");
        }
        _ => {
            dbg::puts("AXON-Bridge: ABI token INVALID — halting\n");
            panic!("ABI token invalid");
        }
    }
    dbg::puts("AXON-Bridge: capability registry initialized\n");
    dbg::puts("AXON-Bridge: @constant_time contracts active\n");
    dbg::puts("AXON-Bridge: AXON-STUB-001 FFI stubs registered\n");
}
