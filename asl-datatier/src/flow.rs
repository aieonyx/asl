// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Data flow descriptor — describes a data movement between PDs.
// Every data flow is checked against the tier boundary rules
// before it is permitted.

use asl_common::datatier::DataTier;

/// A data flow between two PDs.
#[derive(Debug, Clone, Copy)]
pub struct DataFlow {
    /// Source PD identity
    pub src_pd:   u8,
    /// Destination PD identity
    pub dst_pd:   u8,
    /// Tier of the data being moved
    pub src_tier: DataTier,
    /// Tier of the destination context
    pub dst_tier: DataTier,
    /// Size of data in bytes
    pub size:     usize,
}

impl DataFlow {
    pub fn new(
        src_pd: u8,
        dst_pd: u8,
        src_tier: DataTier,
        dst_tier: DataTier,
        size: usize,
    ) -> Self {
        Self { src_pd, dst_pd, src_tier, dst_tier, size }
    }

    /// Returns true if this flow crosses a tier boundary upward.
    pub fn is_tier_upgrade(&self) -> bool {
        DataTier::requires_grant(self.src_tier, self.dst_tier)
    }

    /// Returns true if this is a same-PD internal flow.
    pub fn is_internal(&self) -> bool {
        self.src_pd == self.dst_pd
    }

    /// Returns true if this flow involves Critical tier data.
    pub fn involves_critical(&self) -> bool {
        self.src_tier == DataTier::Critical || self.dst_tier == DataTier::Critical
    }

    /// Returns true if destination context is less sensitive than source.
    pub fn is_downgrade(&self) -> bool {
        self.dst_tier < self.src_tier
    }
}

/// Result of a flow check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowResult {
    /// Flow permitted — same tier or downgrade.
    Permitted,
    /// Flow requires a signed grant from TrustGraph-Gate.
    RequiresGrant,
    /// Flow permitted via validated grant.
    PermittedWithGrant,
    /// Flow blocked — Critical data cannot leave vault PD.
    CriticalVaultViolation,
    /// Flow blocked — self-routing not meaningful.
    SelfFlow,
}
