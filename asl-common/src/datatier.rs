// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// EdisonDB data tier classification.
// DataTier-Enforcer PD uses these to gate cross-tier data flows.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DataTier {
    /// Noise — public, non-sensitive operational data.
    Noise    = 0x00,
    /// Personal — user-owned data requiring consent for access.
    Personal = 0x01,
    /// Critical — encrypted vault data; no plaintext outside vault PD.
    Critical = 0x02,
}

impl DataTier {
    /// Returns true if a flow from `src` to `dst` requires
    /// an explicit signed grant from the TrustGraph-Gate.
    pub fn requires_grant(src: DataTier, dst: DataTier) -> bool {
        dst > src
    }
}
