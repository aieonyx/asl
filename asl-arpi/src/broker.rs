// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ARPi-Broker — central message broker for all ASL IPC.
//
// BrokerResult encodes the outcome of every message dispatch.
// The broker validates header, checks route, advances sequence,
// enforces tier gate, then returns the dispatch result.

use asl_common::arpi::ArpiHeader;
use asl_common::datatier::DataTier;
use asl_common::pd::PdId;

use crate::route::{RouteTable, FASTPATH_PAYLOAD_BYTES};
use crate::sequence::{SequenceTable, SeqError};
use crate::tier_gate::{self, TierGateResult};

/// Result of a broker dispatch attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrokerResult {
    /// Message dispatched on fastpath.
    FastPath,
    /// Message dispatched on slowpath (payload too large for fastpath).
    SlowPath,
    /// Message rejected — invalid ARPi magic.
    InvalidMagic,
    /// Message rejected — unknown route.
    NoRoute,
    /// Message rejected — replay detected.
    ReplayDetected,
    /// Message rejected — cross-tier upgrade without grant.
    TierViolation,
    /// Message rejected — unknown source PD.
    UnknownSource,
}

/// The ARPi-Broker instance.
/// One broker exists per ASL node — initialized at commissioning.
pub struct ArpiBroker {
    routes:    RouteTable,
    sequences: SequenceTable,
    /// Total messages dispatched (monotonic).
    dispatched: u64,
    /// Total messages rejected (monotonic).
    rejected:   u64,
}

impl ArpiBroker {
    /// Creates a new uninitialized broker.
    pub const fn new() -> Self {
        Self {
            routes:     RouteTable::new(),
            sequences:  SequenceTable::new(),
            dispatched: 0,
            rejected:   0,
        }
    }

    /// Registers all six mandatory PDs and their core routes.
    /// Called by GENESIS during commissioning — before any IPC begins.
    pub fn commission(&mut self) -> Result<(), CommissionError> {
        // Register all mandatory PDs in sequence table
        let mandatory = [
            PdId::Genesis         as u8,
            PdId::ArpiBroker      as u8,
            PdId::DataTierEnforcer as u8,
            PdId::TrustGraphGate  as u8,
            PdId::InvertedAdmin   as u8,
            PdId::AxonBridge      as u8,
        ];
        for pd in mandatory {
            self.sequences.register(pd)
                .map_err(|_| CommissionError::SequenceRegistrationFailed)?;
        }

        // Register core mandatory routes (fastpath where payload fits)
        // GENESIS → ARPi-Broker (bootstrap only)
        self.routes.register(
            PdId::Genesis as u8,
            PdId::ArpiBroker as u8,
            true,
        ).map_err(|_| CommissionError::RouteRegistrationFailed)?;

        // ARPi-Broker → all mandatory PDs
        let targets = [
            PdId::DataTierEnforcer as u8,
            PdId::TrustGraphGate  as u8,
            PdId::InvertedAdmin   as u8,
            PdId::AxonBridge      as u8,
        ];
        for dst in targets {
            self.routes.register(
                PdId::ArpiBroker as u8,
                dst,
                true,
            ).map_err(|_| CommissionError::RouteRegistrationFailed)?;
        }

        // DataTier-Enforcer → ARPi-Broker (control messages)
        self.routes.register(
            PdId::DataTierEnforcer as u8,
            PdId::ArpiBroker as u8,
            true,
        ).map_err(|_| CommissionError::RouteRegistrationFailed)?;

        // TrustGraph-Gate → ARPi-Broker (grant signals)
        self.routes.register(
            PdId::TrustGraphGate as u8,
            PdId::ArpiBroker as u8,
            true,
        ).map_err(|_| CommissionError::RouteRegistrationFailed)?;

        // InvertedAdmin → ARPi-Broker (admin signals)
        self.routes.register(
            PdId::InvertedAdmin as u8,
            PdId::ArpiBroker as u8,
            true,
        ).map_err(|_| CommissionError::RouteRegistrationFailed)?;

        // AxonBridge → ARPi-Broker (userspace messages)
        self.routes.register(
            PdId::AxonBridge as u8,
            PdId::ArpiBroker as u8,
            true,
        ).map_err(|_| CommissionError::RouteRegistrationFailed)?;

        Ok(())
    }

    /// Dispatches a message through the broker.
    ///
    /// Validation order (fail-fast):
    ///   1. ARPi header magic
    ///   2. Route exists
    ///   3. Sequence anti-replay
    ///   4. DataTier gate
    ///   5. Fastpath vs slowpath decision
    pub fn dispatch(
        &mut self,
        header: &ArpiHeader,
        payload_len: usize,
        grant_token: Option<&[u8]>,
    ) -> BrokerResult {
        // Step 1: Header magic
        if !header.is_valid_magic() {
            self.rejected += 1;
            return BrokerResult::InvalidMagic;
        }

        // Step 2: Route lookup
        if self.routes.lookup(header.src_pd, header.dst_pd).is_err() {
            self.rejected += 1;
            return BrokerResult::NoRoute;
        }

        // Step 3: Sequence anti-replay
        let seq = { header.seq };
        match self.sequences.validate_and_advance(header.src_pd, seq) {
            Err(SeqError::UnknownPd) => {
                self.rejected += 1;
                return BrokerResult::UnknownSource;
            }
            Err(SeqError::ReplayDetected { .. }) => {
                self.rejected += 1;
                return BrokerResult::ReplayDetected;
            }
            Err(_) => {
                self.rejected += 1;
                return BrokerResult::UnknownSource;
            }
            Ok(()) => {}
        }

        // Step 4: DataTier gate
        let src_tier = DataTier::from_u8(header.data_tier);
        // dst_tier defaults to Noise for broker-internal routing
        let dst_tier = DataTier::Noise;
        match tier_gate::check(src_tier, dst_tier, grant_token) {
            TierGateResult::RequiresGrant => {
                self.rejected += 1;
                return BrokerResult::TierViolation;
            }
            TierGateResult::Allow | TierGateResult::GrantAccepted => {}
        }

        // Step 5: Fastpath decision
        self.dispatched += 1;
        if payload_len <= FASTPATH_PAYLOAD_BYTES
            && self.routes.is_fastpath(header.src_pd, header.dst_pd)
        {
            BrokerResult::FastPath
        } else {
            BrokerResult::SlowPath
        }
    }

    /// Total messages successfully dispatched.
    pub fn dispatched_count(&self) -> u64 { self.dispatched }

    /// Total messages rejected.
    pub fn rejected_count(&self) -> u64 { self.rejected }

    /// Number of registered routes.
    pub fn route_count(&self) -> usize { self.routes.route_count() }

    /// Number of registered sequence entries.
    pub fn sequence_count(&self) -> usize { self.sequences.registered_count() }
}

/// Errors during broker commissioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommissionError {
    SequenceRegistrationFailed,
    RouteRegistrationFailed,
}
