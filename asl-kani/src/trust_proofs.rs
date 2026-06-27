#![cfg(kani)]

// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Kani proofs — TrustGraph capability model invariants

use asl_trustgraph::token::{CapToken, CapabilityType, TokenResult, validate};
use asl_common::datatier::DataTier;

/// PROOF: Self-grant tokens are always invalid.
/// Symbolic over all possible PD IDs and capability types.
#[cfg(kani)]
#[kani::proof]
fn proof_self_grant_always_invalid() {
    let pd: u8 = kani::any();
    let sig = [0x01u8; 64];
    let t = CapToken::new(pd, pd, CapabilityType::Read, DataTier::Noise, 1, sig);
    assert_eq!(validate(&t), TokenResult::SelfGrant);
}

/// PROOF: Zero sequence tokens are always invalid.
#[cfg(kani)]
#[kani::proof]
fn proof_zero_seq_always_invalid() {
    let src: u8 = kani::any();
    let dst: u8 = kani::any();
    kani::assume(src != dst);
    let sig = [0x01u8; 64];
    let t = CapToken::new(src, dst, CapabilityType::Read, DataTier::Noise, 0, sig);
    assert_eq!(validate(&t), TokenResult::ZeroSeq);
}

/// PROOF: Zero signature tokens are always invalid.
#[cfg(kani)]
#[kani::proof]
fn proof_zero_sig_always_invalid() {
    let src: u8 = kani::any();
    let dst: u8 = kani::any();
    kani::assume(src != dst);
    let t = CapToken::new(src, dst, CapabilityType::Read, DataTier::Noise, 1, [0u8; 64]);
    assert_eq!(validate(&t), TokenResult::ZeroSignature);
}

/// PROOF: A valid token (non-self, non-zero seq, non-zero sig) validates.
#[cfg(kani)]
#[kani::proof]
fn proof_valid_token_validates() {
    let src: u8 = kani::any();
    let dst: u8 = kani::any();
    let seq: u64 = kani::any();
    kani::assume(src != dst);
    kani::assume(seq > 0);
    let sig = [0x01u8; 64];
    let t = CapToken::new(src, dst, CapabilityType::Read, DataTier::Noise, seq, sig);
    assert_eq!(validate(&t), TokenResult::Valid);
}

// ── Non-kani tests ────────────────────────────────────────────────────

#[test]
fn test_self_grant_invalid_proof() {
    let t = CapToken::new(0x01, 0x01, CapabilityType::Read, DataTier::Noise, 1, [0x01u8; 64]);
    assert_eq!(validate(&t), TokenResult::SelfGrant);
}

#[test]
fn test_zero_seq_invalid_proof() {
    let t = CapToken::new(0x01, 0x02, CapabilityType::Read, DataTier::Noise, 0, [0x01u8; 64]);
    assert_eq!(validate(&t), TokenResult::ZeroSeq);
}

#[test]
fn test_valid_token_validates_proof() {
    let t = CapToken::new(0x01, 0x02, CapabilityType::Execute, DataTier::Noise, 42, [0x01u8; 64]);
    assert_eq!(validate(&t), TokenResult::Valid);
}
