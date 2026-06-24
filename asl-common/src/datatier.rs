// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// EdisonDB data tier classification.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DataTier {
    Noise    = 0x00,
    Personal = 0x01,
    Critical = 0x02,
}

impl DataTier {
    /// Returns true if a flow from src to dst requires a signed grant.
    pub fn requires_grant(src: DataTier, dst: DataTier) -> bool {
        dst > src
    }

    /// Converts a raw u8 to DataTier.
    /// Unknown values default to Critical (safe default — most restrictive).
    pub fn from_u8(val: u8) -> DataTier {
        match val {
            0x00 => DataTier::Noise,
            0x01 => DataTier::Personal,
            _    => DataTier::Critical,
        }
    }
}
