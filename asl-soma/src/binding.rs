// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Data-leaves binding — every outgoing data packet carries
// the composite hashcode of the originating node.
//
// When data leaves the sovereign node, it is stamped with:
//   1. The composite hashcode (32 bytes)
//   2. The data tier of the payload
//   3. A monotonic binding sequence
//
// At the destination, the binding is verified against the
// sender's registered composite hash in EdisonDB.
// Mismatch = data rejected as unauthorized exfiltration.

use asl_common::datatier::DataTier;
use crate::composite::{CompositeHash, COMPOSITE_HASH_SIZE};

/// Owner-configurable binding mode per transfer.
/// Critical tier always uses Full — cannot be overridden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    /// Full sovereign binding — composite hash + three-key threshold enforced.
    /// Default for Critical and Personal tier data.
    Full,
    /// Provenance only — hash stamped but threshold not enforced.
    /// Allows sharing outside AIEONYX mesh. Default for Noise tier.
    Provenance,
    /// Open — no binding. Requires dual-key admin authorization + audit log.
    /// Never permitted for Critical tier data.
    Open,
}

impl BindingMode {
    /// Returns the minimum permitted mode for a given data tier.
    /// Critical data can never be Open or Provenance.
    pub fn minimum_for_tier(tier: asl_common::datatier::DataTier) -> Self {
        match tier {
            asl_common::datatier::DataTier::Critical => BindingMode::Full,
            asl_common::datatier::DataTier::Personal => BindingMode::Full,
            asl_common::datatier::DataTier::Noise    => BindingMode::Provenance,
        }
    }

    /// Returns true if this mode satisfies the minimum for the tier.
    pub fn satisfies_tier(self, tier: asl_common::datatier::DataTier) -> bool {
        let min = Self::minimum_for_tier(tier);
        (self as u8) <= (min as u8)
    }
}


/// A bound data packet header — prepended to every outgoing payload.
/// Total size: 32 (hash) + 1 (tier) + 8 (seq) + 2 (magic) = 43 bytes.
#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Eq)]
#[repr(C)]
pub struct BoundPacketHeader {
    /// Magic: 0xA1E0 (AIEONYX binding marker)
    pub magic:     u16,
    /// Data tier of the payload
    pub tier:      u8,
    /// Padding
    _pad:          [u8; 5],
    /// Monotonic binding sequence
    pub seq:       u64,
    /// Composite hashcode of originating node
    pub hash:      [u8; COMPOSITE_HASH_SIZE],
}

impl BoundPacketHeader {
    pub const MAGIC: u16 = 0xA1E0;
    pub const SIZE: usize = core::mem::size_of::<BoundPacketHeader>();

    pub fn new(tier: DataTier, seq: u64, hash: &CompositeHash) -> Self {
        Self {
            magic: Self::MAGIC,
            tier:  tier as u8,
            _pad:  [0u8; 5],
            seq,
            hash:  *hash.as_bytes(),
        }
    }

    pub fn is_valid_magic(&self) -> bool {
        self.magic == Self::MAGIC
    }

    /// Returns true if the embedded hash is non-zero.
    pub fn has_valid_hash(&self) -> bool {
        self.hash.iter().any(|&b| b != 0)
    }
}

// Compile-time size check
const _: () = assert!(
    core::mem::size_of::<BoundPacketHeader>() == 48,
    "BoundPacketHeader must be 48 bytes"
);

/// Binding engine — stamps outgoing data and verifies incoming.
pub struct BindingEngine {
    /// This node's composite hash
    node_hash: Option<CompositeHash>,
    /// Monotonic outgoing sequence counter
    out_seq:   u64,
    /// Total packets bound (outgoing)
    bound:     u64,
    /// Total packets verified (incoming)
    verified:  u64,
    /// Total binding violations detected
    violations: u64,
}

impl BindingEngine {
    pub const fn new() -> Self {
        Self {
            node_hash:  None,
            out_seq:    0,
            bound:      0,
            verified:   0,
            violations: 0,
        }
    }

    /// Register this node's composite hash.
    /// Must be called before any data can leave the node.
    pub fn register_node_hash(&mut self, hash: CompositeHash) -> Result<(), BindingError> {
        if !hash.is_valid() {
            return Err(BindingError::InvalidHash);
        }
        self.node_hash = Some(hash);
        Ok(())
    }

    /// Stamp an outgoing data packet with the node's composite hash.
    pub fn stamp_outgoing(
        &mut self,
        tier: DataTier,
    ) -> Result<BoundPacketHeader, BindingError> {
        let hash = self.node_hash.ok_or(BindingError::NodeHashNotRegistered)?;
        self.out_seq += 1;
        self.bound += 1;
        Ok(BoundPacketHeader::new(tier, self.out_seq, &hash))
    }

    /// Verify an incoming bound packet header.
    /// Returns Ok if the binding is structurally valid.
    /// Full hash verification against EdisonDB in ASL-M12.
    pub fn verify_incoming(
        &mut self,
        header: &BoundPacketHeader,
    ) -> Result<(), BindingError> {
        if !header.is_valid_magic() {
            self.violations += 1;
            return Err(BindingError::InvalidMagic);
        }
        if !header.has_valid_hash() {
            self.violations += 1;
            return Err(BindingError::InvalidHash);
        }
        if header.seq == 0 {
            self.violations += 1;
            return Err(BindingError::ZeroSequence);
        }
        self.verified += 1;
        Ok(())
    }

    pub fn bound_count(&self) -> u64    { self.bound }
    pub fn verified_count(&self) -> u64 { self.verified }
    pub fn violation_count(&self) -> u64 { self.violations }
    pub fn is_registered(&self) -> bool { self.node_hash.is_some() }
    pub fn current_seq(&self) -> u64    { self.out_seq }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingError {
    NodeHashNotRegistered,
    InvalidHash,
    InvalidMagic,
    ZeroSequence,
}
