// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Protection Domain identity constants.
// Six mandatory sovereign PDs — unconditional across all profiles.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PdId {
    Genesis         = 0x01,
    ArpiBroker      = 0x02,
    DataTierEnforcer= 0x03,
    TrustGraphGate  = 0x04,
    InvertedAdmin   = 0x05,
    AxonBridge      = 0x06,
    // Optional PDs — profile-gated
    GpuCap          = 0x10,
    PowerMgmt       = 0x11,
    NetworkRouting  = 0x12,
    FirewallCap     = 0x13,
    TouchSensor     = 0x14,
}

impl PdId {
    /// Returns true if this PD is mandatory across all profiles.
    pub fn is_mandatory(self) -> bool {
        (self as u8) < 0x10
    }
}
