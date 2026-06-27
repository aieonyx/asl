#![cfg(kani)]

// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Kani proofs — DataTier flow rule invariants

use asl_common::datatier::DataTier;

/// PROOF: Grant is required iff dst tier > src tier.
/// Symbolic over all possible tier combinations.
#[cfg(kani)]
#[kani::proof]
fn proof_grant_required_iff_upgrade() {
    let src_val: u8 = kani::any();
    let dst_val: u8 = kani::any();
    kani::assume(src_val <= 2);
    kani::assume(dst_val <= 2);
    let src = DataTier::from_u8(src_val);
    let dst = DataTier::from_u8(dst_val);
    let requires = DataTier::requires_grant(src, dst);
    let is_upgrade = (dst as u8) > (src as u8);
    assert_eq!(requires, is_upgrade);
}

/// PROOF: Same-tier flows never require a grant.
#[cfg(kani)]
#[kani::proof]
fn proof_same_tier_no_grant() {
    for tier in [DataTier::Noise, DataTier::Personal, DataTier::Critical] {
        assert!(!DataTier::requires_grant(tier, tier));
    }
}

/// PROOF: DataTier ordering is total and transitive.
#[cfg(kani)]
#[kani::proof]
fn proof_datatier_total_order() {
    assert!(DataTier::Noise    < DataTier::Personal);
    assert!(DataTier::Personal < DataTier::Critical);
    assert!(DataTier::Noise    < DataTier::Critical);
}

/// PROOF: from_u8 is total — never panics on any u8 input.
#[cfg(kani)]
#[kani::proof]
fn proof_from_u8_total() {
    let val: u8 = kani::any();
    let _ = DataTier::from_u8(val); // must not panic
}

/// PROOF: Unknown tier defaults to Critical (most restrictive).
#[cfg(kani)]
#[kani::proof]
fn proof_unknown_defaults_critical() {
    let val: u8 = kani::any();
    kani::assume(val > 2);
    let tier = DataTier::from_u8(val);
    assert_eq!(tier, DataTier::Critical);
}

// ── Non-kani tests ────────────────────────────────────────────────────

#[test]
fn test_grant_required_iff_upgrade_proof() {
    assert!(DataTier::requires_grant(DataTier::Noise, DataTier::Personal));
    assert!(DataTier::requires_grant(DataTier::Noise, DataTier::Critical));
    assert!(!DataTier::requires_grant(DataTier::Critical, DataTier::Noise));
    assert!(!DataTier::requires_grant(DataTier::Personal, DataTier::Noise));
}

#[test]
fn test_same_tier_no_grant_proof() {
    assert!(!DataTier::requires_grant(DataTier::Noise, DataTier::Noise));
    assert!(!DataTier::requires_grant(DataTier::Personal, DataTier::Personal));
    assert!(!DataTier::requires_grant(DataTier::Critical, DataTier::Critical));
}

#[test]
fn test_datatier_ordering_proof() {
    assert!(DataTier::Noise < DataTier::Personal);
    assert!(DataTier::Personal < DataTier::Critical);
}

#[test]
fn test_from_u8_unknown_defaults_critical_proof() {
    assert_eq!(DataTier::from_u8(0xFF), DataTier::Critical);
    assert_eq!(DataTier::from_u8(0x10), DataTier::Critical);
}
