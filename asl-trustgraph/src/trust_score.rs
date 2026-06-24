// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Trust score — per-PD trust rating derived from:
//   - Number of valid capability grants received
//   - Number of revocations (reduces score)
//   - Whether the PD is mandatory (always max trust)
//
// Trust score is advisory — it informs ARPi routing priority
// but does not override capability enforcement.


/// Trust score range: 0 (untrusted) to 100 (fully trusted).
pub const MAX_TRUST: u8 = 100;
pub const MIN_TRUST: u8 = 0;
/// Mandatory PDs always have maximum trust.
pub const MANDATORY_TRUST: u8 = MAX_TRUST;
/// New optional PDs start at baseline trust.
pub const BASELINE_TRUST: u8 = 50;

/// Per-PD trust entry.
#[derive(Debug, Clone, Copy)]
pub struct TrustEntry {
    pub pd_id: u8,
    pub score: u8,
    pub grants_received: u32,
    pub revocations:     u32,
}

impl TrustEntry {
    pub fn new_mandatory(pd_id: u8) -> Self {
        Self { pd_id, score: MANDATORY_TRUST, grants_received: 0, revocations: 0 }
    }

    pub fn new_optional(pd_id: u8) -> Self {
        Self { pd_id, score: BASELINE_TRUST, grants_received: 0, revocations: 0 }
    }

    /// Record a grant received — increases trust score slightly.
    pub fn record_grant(&mut self) {
        self.grants_received += 1;
        self.score = self.score.saturating_add(2).min(MAX_TRUST);
    }

    /// Record a revocation — decreases trust score.
    pub fn record_revocation(&mut self) {
        self.revocations += 1;
        self.score = self.score.saturating_sub(10);
    }

    /// Returns true if PD is considered trusted (score >= 50).
    pub fn is_trusted(&self) -> bool {
        self.score >= BASELINE_TRUST
    }
}

/// Trust score registry.
pub struct TrustRegistry {
    entries: [TrustEntry; 16],
    count:   usize,
}

impl TrustRegistry {
    pub const fn new() -> Self {
        Self {
            entries: [TrustEntry { pd_id: 0, score: 0, grants_received: 0, revocations: 0 }; 16],
            count: 0,
        }
    }

    /// Register a PD with appropriate baseline trust.
    pub fn register(&mut self, pd_id: u8, mandatory: bool) -> Result<(), TrustError> {
        if self.count >= 16 {
            return Err(TrustError::RegistryFull);
        }
        if self.find(pd_id).is_some() {
            return Err(TrustError::AlreadyRegistered);
        }
        self.entries[self.count] = if mandatory {
            TrustEntry::new_mandatory(pd_id)
        } else {
            TrustEntry::new_optional(pd_id)
        };
        self.count += 1;
        Ok(())
    }

    /// Returns trust score for a PD.
    pub fn score(&self, pd_id: u8) -> Option<u8> {
        self.find(pd_id).map(|e| e.score)
    }

    /// Returns true if PD is trusted.
    pub fn is_trusted(&self, pd_id: u8) -> bool {
        self.find(pd_id).map(|e| e.is_trusted()).unwrap_or(false)
    }

    /// Record grant received by a PD.
    pub fn record_grant(&mut self, pd_id: u8) -> Result<(), TrustError> {
        let count = self.count;
        self.entries[..count]
            .iter_mut()
            .find(|e| e.pd_id == pd_id)
            .map(|e| e.record_grant())
            .ok_or(TrustError::UnknownPd)
    }

    /// Record revocation against a PD.
    pub fn record_revocation(&mut self, pd_id: u8) -> Result<(), TrustError> {
        let count = self.count;
        self.entries[..count]
            .iter_mut()
            .find(|e| e.pd_id == pd_id)
            .map(|e| e.record_revocation())
            .ok_or(TrustError::UnknownPd)
    }

    pub fn registered_count(&self) -> usize { self.count }

    fn find(&self, pd_id: u8) -> Option<&TrustEntry> {
        self.entries[..self.count].iter().find(|e| e.pd_id == pd_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustError {
    RegistryFull,
    AlreadyRegistered,
    UnknownPd,
}
