// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Admin action registry — every sovereign admin operation
// is a named, typed, monotonically-counted action.
// No unnamed or implicit admin operations exist.

/// Classes of admin actions in the sovereign stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AdminActionClass {
    /// PD commissioning or decommissioning
    PdLifecycle     = 0x01,
    /// Capability grant or revocation
    CapabilityMgmt  = 0x02,
    /// Trust graph modification
    TrustGraphEdit  = 0x03,
    /// Key rotation or ceremony
    KeyCeremony     = 0x04,
    /// DataTier boundary change
    TierBoundary    = 0x05,
    /// Emergency sovereign halt
    SovereignHalt   = 0xFF,
}

/// A single admin action request.
#[derive(Debug, Clone, Copy)]
pub struct AdminAction {
    /// Class of action being requested
    pub class:       AdminActionClass,
    /// Monotonic action counter — prevents replay
    pub counter:     u64,
    /// Target PD for this action (0xFF = system-wide)
    pub target_pd:   u8,
    /// Requesting PD identity
    pub requestor:   u8,
}

impl AdminAction {
    pub fn new(
        class: AdminActionClass,
        counter: u64,
        target_pd: u8,
        requestor: u8,
    ) -> Self {
        Self { class, counter, target_pd, requestor }
    }

    /// Returns true if this is a system-wide action.
    pub fn is_system_wide(&self) -> bool {
        self.target_pd == 0xFF
    }

    /// Returns true if this action class requires dual-key authorization.
    /// All classes require dual-key — this method documents the invariant.
    pub fn requires_dual_key(&self) -> bool {
        true // Inverted Admin Model: ALL actions require dual-key
    }
}

/// Action counter tracker — monotonic, per-requestor PD.
pub struct ActionCounter {
    entries: [(u8, u64); 16],
    count:   usize,
}

impl ActionCounter {
    pub const fn new() -> Self {
        Self { entries: [(0, 0); 16], count: 0 }
    }

    /// Register a requestor PD.
    pub fn register(&mut self, pd_id: u8) -> Result<(), CounterError> {
        if self.count >= 16 {
            return Err(CounterError::Full);
        }
        if self.find(pd_id).is_some() {
            return Err(CounterError::AlreadyRegistered);
        }
        self.entries[self.count] = (pd_id, 0);
        self.count += 1;
        Ok(())
    }

    /// Validate and advance counter for a requestor.
    /// Counter must be strictly greater than last seen.
    pub fn validate_and_advance(
        &mut self,
        pd_id: u8,
        counter: u64,
    ) -> Result<(), CounterError> {
        let count = self.count;
        for i in 0..count {
            if self.entries[i].0 == pd_id {
                if counter <= self.entries[i].1 {
                    return Err(CounterError::Replay {
                        last: self.entries[i].1,
                        got:  counter,
                    });
                }
                self.entries[i].1 = counter;
                return Ok(());
            }
        }
        Err(CounterError::UnknownPd)
    }

    /// Returns current counter value for a PD.
    pub fn current(&self, pd_id: u8) -> Option<u64> {
        self.find(pd_id).map(|(_, c)| *c)
    }

    fn find(&self, pd_id: u8) -> Option<&(u8, u64)> {
        self.entries[..self.count].iter().find(|(id, _)| *id == pd_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterError {
    Full,
    AlreadyRegistered,
    UnknownPd,
    Replay { last: u64, got: u64 },
}
