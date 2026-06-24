// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// InvertedAdmin — the central admin enforcement engine.
// Combines DevMode rejection, dual-key authorization,
// and action counter into a single sovereign gate.

use crate::action::{ActionCounter, AdminAction, CounterError};
use crate::devmode;
use crate::dual_key::{AuthToken, DualKeyError, DualKeyRegistry};

/// Result of an admin authorization attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminResult {
    /// Action authorized — proceed.
    Authorized,
    /// DevMode detected — unconditionally rejected.
    DevModeRejected,
    /// Dual-key authorization failed.
    DualKeyFailed(DualKeyError),
    /// Action counter replay or unknown PD.
    CounterFailed(CounterError),
}

/// The Inverted Admin enforcement engine.
pub struct InvertedAdmin {
    registry: DualKeyRegistry,
    counters: ActionCounter,
    /// Total actions authorized (monotonic).
    authorized: u64,
    /// Total actions rejected (monotonic).
    rejected: u64,
}

impl InvertedAdmin {
    pub const fn new() -> Self {
        Self {
            registry:   DualKeyRegistry::new(),
            counters:   ActionCounter::new(),
            authorized: 0,
            rejected:   0,
        }
    }

    /// Register a key slot for dual-key authorization.
    pub fn register_key_slot(
        &mut self,
        slot_id: u8,
        fingerprint: u64,
    ) -> Result<(), DualKeyError> {
        self.registry.register_slot(slot_id, fingerprint)
    }

    /// Register a PD as an admin requestor.
    pub fn register_requestor(&mut self, pd_id: u8) -> Result<(), CounterError> {
        self.counters.register(pd_id)
    }

    /// Authorize an admin action.
    /// Validation order:
    ///   1. DevMode unconditional rejection
    ///   2. Action counter anti-replay
    ///   3. Dual-key authorization
    pub fn authorize(
        &mut self,
        action: &AdminAction,
        token_a: &AuthToken,
        token_b: &AuthToken,
    ) -> AdminResult {
        // Step 1: DevMode check — unconditional
        if devmode::is_active() {
            self.rejected += 1;
            return AdminResult::DevModeRejected;
        }

        // Step 2: Action counter
        match self.counters.validate_and_advance(
            action.requestor,
            action.counter,
        ) {
            Err(e) => {
                self.rejected += 1;
                return AdminResult::CounterFailed(e);
            }
            Ok(()) => {}
        }

        // Step 3: Dual-key authorization
        match self.registry.authorize(token_a, token_b, action.counter) {
            Err(e) => {
                self.rejected += 1;
                return AdminResult::DualKeyFailed(e);
            }
            Ok(()) => {}
        }

        self.authorized += 1;
        AdminResult::Authorized
    }

    pub fn authorized_count(&self) -> u64 { self.authorized }
    pub fn rejected_count(&self) -> u64 { self.rejected }
    pub fn slot_count(&self) -> usize { self.registry.slot_count() }
}
