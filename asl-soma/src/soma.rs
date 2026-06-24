// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// SOMA — Sovereign Identity engine.
// Combines composite identity, three-key threshold,
// and data-leaves binding into one sovereign gate.

use asl_common::datatier::DataTier;
use crate::composite::{CompositeHash, IdentityLayers, IdentityError};
use crate::threshold::{ThresholdRegistry, ThresholdResult, KeySlotId, ThresholdError};
use crate::binding::{BindingEngine, BoundPacketHeader, BindingError};

/// Result of SOMA commissioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SomaResult {
    Ready,
    IdentityIncomplete,
    ThresholdIncomplete,
    BindingFailed,
}

/// The SOMA-Identity PD engine.
pub struct Soma {
    layers:    Option<IdentityLayers>,
    hash:      Option<CompositeHash>,
    threshold: ThresholdRegistry,
    binding:   BindingEngine,
    commissioned: bool,
}

impl Soma {
    pub const fn new() -> Self {
        Self {
            layers:       None,
            hash:         None,
            threshold:    ThresholdRegistry::new(),
            binding:      BindingEngine::new(),
            commissioned: false,
        }
    }

    /// Commission SOMA with identity layers and key fingerprints.
    pub fn commission(
        &mut self,
        layers: IdentityLayers,
        os_fp: u64,
        edb_fp: u64,
        owner_fp: u64,
    ) -> Result<SomaResult, SomaError> {
        // Step 1: Derive composite hash
        let hash = CompositeHash::derive(&layers)
            .map_err(|e| SomaError::Identity(e))?;

        // Step 2: Enroll threshold keys
        self.threshold.enroll(KeySlotId::OsKey, os_fp)
            .map_err(|e| SomaError::Threshold(e))?;
        self.threshold.enroll(KeySlotId::EdisonDbKey, edb_fp)
            .map_err(|e| SomaError::Threshold(e))?;
        self.threshold.enroll(KeySlotId::OwnerKey, owner_fp)
            .map_err(|e| SomaError::Threshold(e))?;

        // Step 3: Register node hash in binding engine
        self.binding.register_node_hash(hash)
            .map_err(|_| SomaError::Binding(BindingError::InvalidHash))?;

        self.layers = Some(layers);
        self.hash = Some(hash);
        self.commissioned = true;
        Ok(SomaResult::Ready)
    }

    /// Stamp an outgoing data packet.
    pub fn stamp(&mut self, tier: DataTier) -> Result<BoundPacketHeader, SomaError> {
        if !self.commissioned {
            return Err(SomaError::NotCommissioned);
        }
        self.binding.stamp_outgoing(tier)
            .map_err(|e| SomaError::Binding(e))
    }

    /// Verify an incoming bound packet.
    pub fn verify(&mut self, header: &BoundPacketHeader) -> Result<(), SomaError> {
        self.binding.verify_incoming(header)
            .map_err(|e| SomaError::Binding(e))
    }

    /// Check if presented key fingerprints meet the threshold.
    pub fn check_threshold(&self, presented: &[u64]) -> ThresholdResult {
        self.threshold.check_threshold(presented)
    }

    pub fn is_commissioned(&self) -> bool { self.commissioned }
    pub fn composite_hash(&self) -> Option<&CompositeHash> { self.hash.as_ref() }
    pub fn threshold_complete(&self) -> bool { self.threshold.is_complete() }
    pub fn bound_count(&self) -> u64 { self.binding.bound_count() }
    pub fn violation_count(&self) -> u64 { self.binding.violation_count() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SomaError {
    NotCommissioned,
    Identity(IdentityError),
    Threshold(ThresholdError),
    Binding(BindingError),
}
