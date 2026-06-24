// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Cross-tier grant registry — stores and validates signed grants
// from TrustGraph-Gate that permit tier boundary crossings.
//
// ASL-M4: structural grant model with stub signature validation.
// ASL-M5: real Ed25519 grant token verification wired here.

use asl_common::datatier::DataTier;

/// Maximum grants stored simultaneously.
pub const MAX_GRANTS: usize = 32;

/// A signed cross-tier grant from TrustGraph-Gate.
#[derive(Debug, Clone, Copy)]
pub struct TierGrant {
    /// PD that may perform the tier upgrade
    pub grantee_pd: u8,
    /// Source tier permitted
    pub src_tier:   DataTier,
    /// Destination tier permitted
    pub dst_tier:   DataTier,
    /// Monotonic grant sequence
    pub seq:        u64,
    /// Signature stub — real Ed25519 in ASL-M5
    pub sig:        [u8; 64],
    pub active:     bool,
}

impl TierGrant {
    pub fn new(
        grantee_pd: u8,
        src_tier: DataTier,
        dst_tier: DataTier,
        seq: u64,
        sig: [u8; 64],
    ) -> Self {
        Self { grantee_pd, src_tier, dst_tier, seq, sig, active: true }
    }

    /// Validates grant structure (stub).
    pub fn is_valid(&self) -> bool {
        self.active
            && self.seq > 0
            && DataTier::requires_grant(self.src_tier, self.dst_tier)
            && self.sig.iter().any(|&b| b != 0)
    }
}

/// Grant registry — stores active cross-tier grants.
pub struct GrantRegistry {
    grants: [Option<TierGrant>; MAX_GRANTS],
    count:  usize,
    /// Monotonic counter — next expected grant seq
    next_seq: u64,
}

impl GrantRegistry {
    pub const fn new() -> Self {
        Self {
            grants:   [None; MAX_GRANTS],
            count:    0,
            next_seq: 1,
        }
    }

    /// Register a new cross-tier grant.
    pub fn register(&mut self, grant: TierGrant) -> Result<(), GrantError> {
        if self.count >= MAX_GRANTS {
            return Err(GrantError::RegistryFull);
        }
        if !grant.is_valid() {
            return Err(GrantError::InvalidGrant);
        }
        if grant.seq < self.next_seq {
            return Err(GrantError::ReplayDetected);
        }
        // Find empty slot
        for slot in self.grants.iter_mut() {
            if slot.is_none() {
                *slot = Some(grant);
                self.count += 1;
                self.next_seq = grant.seq + 1;
                return Ok(());
            }
        }
        Err(GrantError::RegistryFull)
    }

    /// Look up a valid grant for a specific flow.
    pub fn lookup(
        &self,
        grantee_pd: u8,
        src_tier: DataTier,
        dst_tier: DataTier,
    ) -> Option<&TierGrant> {
        self.grants.iter()
            .filter_map(|g| g.as_ref())
            .find(|g| {
                g.active
                    && g.grantee_pd == grantee_pd
                    && g.src_tier == src_tier
                    && g.dst_tier == dst_tier
            })
    }

    /// Revoke all grants for a PD (called on PD decommission).
    pub fn revoke_all(&mut self, grantee_pd: u8) -> usize {
        let mut revoked = 0;
        for slot in self.grants.iter_mut() {
            if let Some(g) = slot {
                if g.grantee_pd == grantee_pd {
                    g.active = false;
                    revoked += 1;
                }
            }
        }
        self.count = self.count.saturating_sub(revoked);
        revoked
    }

    pub fn active_count(&self) -> usize { self.count }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantError {
    RegistryFull,
    InvalidGrant,
    ReplayDetected,
}
