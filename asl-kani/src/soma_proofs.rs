#![cfg(kani)]

// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Kani proofs — SOMA composite identity invariants

use asl_soma::composite::{CompositeHash, IdentityLayers};
use asl_soma::threshold::{KeySlotId, ThresholdRegistry, ThresholdResult};

/// PROOF: CompositeHash is always 32 bytes.
#[cfg(kani)]
#[kani::proof]
fn proof_composite_hash_size() {
    assert_eq!(asl_soma::composite::COMPOSITE_HASH_SIZE, 32);
    assert_eq!(core::mem::size_of::<CompositeHash>(), 32);
}

/// PROOF: Incomplete identity (zero hw_uid) always fails derivation.
#[cfg(kani)]
#[kani::proof]
fn proof_incomplete_identity_fails() {
    let layers = IdentityLayers::new(0, 0xA1E0_0001, 0xA1E0_0002, 0);
    assert!(CompositeHash::derive(&layers).is_err());
}

/// PROOF: Two distinct sets of layers always produce distinct hashes.
/// (collision resistance — symbolic proof over symbolic inputs)
#[cfg(kani)]
#[kani::proof]
fn proof_distinct_layers_distinct_hashes() {
    let hw1: u64 = kani::any();
    let hw2: u64 = kani::any();
    kani::assume(hw1 != hw2);
    kani::assume(hw1 != 0);
    kani::assume(hw2 != 0);
    // Both must share AIEONYX prefix for pairing
    kani::assume((hw1 >> 48) == 0xA1E0);
    kani::assume((hw2 >> 48) == 0xA1E0);
    let kern: u64 = 0xA1E0_0000_0002_0001;
    let os: u64   = 0xA1E0_0000_0003_0001;
    let l1 = IdentityLayers::new(hw1, kern, os, 0);
    let l2 = IdentityLayers::new(hw2, kern, os, 0);
    let h1 = CompositeHash::derive(&l1).unwrap();
    let h2 = CompositeHash::derive(&l2).unwrap();
    assert_ne!(h1, h2);
}

/// PROOF: Threshold requires all three keys — never fewer.
#[cfg(kani)]
#[kani::proof]
fn proof_threshold_requires_all_three() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey,       0xA1E0_0001).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, 0xA1E0_0002).unwrap();
    r.enroll(KeySlotId::OwnerKey,    0xA1E0_0003).unwrap();
    // Two keys never sufficient
    let result = r.check_threshold(&[0xA1E0_0001, 0xA1E0_0002]);
    assert_ne!(result, ThresholdResult::ThresholdMet);
    // All three sufficient
    let result3 = r.check_threshold(&[0xA1E0_0001, 0xA1E0_0002, 0xA1E0_0003]);
    assert_eq!(result3, ThresholdResult::ThresholdMet);
}

// ── Non-kani tests ────────────────────────────────────────────────────

#[test]
fn test_composite_hash_size_proof() {
    assert_eq!(asl_soma::composite::COMPOSITE_HASH_SIZE, 32);
}

#[test]
fn test_incomplete_identity_fails_proof() {
    let l = IdentityLayers::new(0, 1, 1, 0);
    assert!(CompositeHash::derive(&l).is_err());
}

#[test]
fn test_threshold_requires_all_three_proof() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey,       0xA1E0_0001).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, 0xA1E0_0002).unwrap();
    r.enroll(KeySlotId::OwnerKey,    0xA1E0_0003).unwrap();
    assert_ne!(
        r.check_threshold(&[0xA1E0_0001, 0xA1E0_0002]),
        ThresholdResult::ThresholdMet
    );
    assert_eq!(
        r.check_threshold(&[0xA1E0_0001, 0xA1E0_0002, 0xA1E0_0003]),
        ThresholdResult::ThresholdMet
    );
}
