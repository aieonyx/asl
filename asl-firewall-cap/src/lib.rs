// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// asl-firewall-cap — stub pending its milestone implementation.
// ASL-M1: structure only. Logic implemented in assigned milestone.

#![no_std]

use asl_common::pd::PdId;

pub fn pd_id() -> PdId {
    // Each crate returns its own PdId — filled in per milestone.
    PdId::Genesis // placeholder — overridden in each crate's milestone
}
