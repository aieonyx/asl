// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Capability tokens — signed proof of a granted capability edge.
// A token authorizes a specific (src_pd, dst_pd, capability) triple.

use asl_common::datatier::DataTier;

/// Capability types that can be granted between PDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CapabilityType {
    /// Read access to a PD's data
    Read        = 0x01,
    /// Write access to a PD's data
    Write       = 0x02,
    /// Execute a PD's sovereign function
    Execute     = 0x03,
    /// Delegate a capability to a third PD
    Delegate    = 0x04,
    /// Cross-tier data tier upgrade
    TierUpgrade = 0x05,
    /// Admin action authorization
    AdminAction = 0x06,
}

/// A capability grant token.
/// Authorizes src_pd to exercise capability_type on dst_pd.
#[derive(Debug, Clone, Copy)]
pub struct CapToken {
    pub src_pd:   u8,
    pub dst_pd:   u8,
    pub cap_type: CapabilityType,
    pub tier:     DataTier,
    /// Monotonic grant sequence — anti-replay
    pub seq:      u64,
    /// Signature stub — real Ed25519 in ASL-M5
    pub sig:      [u8; 64],
}

impl CapToken {
    pub fn new(
        src_pd: u8,
        dst_pd: u8,
        cap_type: CapabilityType,
        tier: DataTier,
        seq: u64,
        sig: [u8; 64],
    ) -> Self {
        Self { src_pd, dst_pd, cap_type, tier, seq, sig }
    }

    /// Validates token structure (stub — sig check in ASL-M5).
    pub fn is_structurally_valid(&self) -> bool {
        self.src_pd != self.dst_pd
            && self.seq > 0
            && self.sig.iter().any(|&b| b != 0)
    }

    /// Returns true if this token grants tier upgrade capability.
    pub fn grants_tier_upgrade(&self) -> bool {
        self.cap_type == CapabilityType::TierUpgrade
    }
}

/// Token validation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenResult {
    Valid,
    SelfGrant,
    ZeroSeq,
    ZeroSignature,
    InvalidCapType,
}

/// Validates a capability token.
pub fn validate(token: &CapToken) -> TokenResult {
    if token.src_pd == token.dst_pd {
        return TokenResult::SelfGrant;
    }
    if token.seq == 0 {
        return TokenResult::ZeroSeq;
    }
    if token.sig.iter().all(|&b| b == 0) {
        return TokenResult::ZeroSignature;
    }
    TokenResult::Valid
}
