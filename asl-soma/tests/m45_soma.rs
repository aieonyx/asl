// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ASL-M4.5 test suite — SOMA-Identity PD (TriSec Point A)
// Target: 50+ tests, 0 failures
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_soma::composite::{CompositeHash, IdentityError, IdentityLayers, COMPOSITE_HASH_SIZE};
use asl_soma::threshold::{KeySlotId, ThresholdRegistry, ThresholdResult};
use asl_soma::binding::{BindingEngine, BindingError, BoundPacketHeader};
use asl_soma::soma::{Soma, SomaError};
use asl_common::datatier::DataTier;

const HW_FP:    u64 = 0xA1E0_0000_0001_0001;
const KERN_FP:  u64 = 0xA1E0_0000_0002_0001;
const OS_FP:    u64 = 0xA1E0_0000_0003_0001;
const BIO_FP:   u64 = 0xA1E0_0000_0004_0001;
const KEY_OS:   u64 = 0xA1E0_0000_00AA_0001;
const KEY_EDB:  u64 = 0xA1E0_0000_00BB_0001;
const KEY_OWN:  u64 = 0xA1E0_0000_00CC_0001;

fn layers() -> IdentityLayers {
    IdentityLayers::new(HW_FP, KERN_FP, OS_FP, BIO_FP)
}

fn commissioned_soma() -> Soma {
    let mut s = Soma::new();
    s.commission(layers(), KEY_OS, KEY_EDB, KEY_OWN).unwrap();
    s
}

// ── IdentityLayers tests (10) ─────────────────────────────────────────

#[test]
fn test_layers_complete_all_four() {
    assert!(layers().is_complete());
}

#[test]
fn test_layers_incomplete_zero_hw() {
    let l = IdentityLayers::new(0, KERN_FP, OS_FP, BIO_FP);
    assert!(!l.is_complete());
}

#[test]
fn test_layers_incomplete_zero_kernel() {
    let l = IdentityLayers::new(HW_FP, 0, OS_FP, BIO_FP);
    assert!(!l.is_complete());
}

#[test]
fn test_layers_incomplete_zero_os() {
    let l = IdentityLayers::new(HW_FP, KERN_FP, 0, BIO_FP);
    assert!(!l.is_complete());
}

#[test]
fn test_layers_biometric_optional() {
    // Zero biometric is allowed — not enrolled
    let l = IdentityLayers::new(HW_FP, KERN_FP, OS_FP, 0);
    assert!(l.is_complete()); // still complete without biometric
    assert!(!l.has_biometric());
}

#[test]
fn test_layers_has_biometric() {
    assert!(layers().has_biometric());
}

#[test]
fn test_layers_stub_is_complete() {
    assert!(IdentityLayers::stub().is_complete());
}

#[test]
fn test_layers_stub_has_biometric() {
    assert!(IdentityLayers::stub().has_biometric());
}

#[test]
fn test_layers_os_paired_to_hw() {
    assert!(layers().os_paired_to_hw());
}

#[test]
fn test_layers_unpaired_os_fails() {
    // OS UID without AIEONYX prefix — pairing fails
    let l = IdentityLayers::new(HW_FP, KERN_FP, 0x1234_5678_0003_0001, BIO_FP);
    assert!(!l.os_paired_to_hw());
}

// ── CompositeHash tests (10) ──────────────────────────────────────────

#[test]
fn test_hash_derive_succeeds() {
    assert!(CompositeHash::derive(&layers()).is_ok());
}

#[test]
fn test_hash_derive_incomplete_fails() {
    let l = IdentityLayers::new(0, KERN_FP, OS_FP, BIO_FP);
    assert_eq!(CompositeHash::derive(&l), Err(IdentityError::IncompleteIdentity));
}

#[test]
fn test_hash_is_valid_nonzero() {
    let h = CompositeHash::derive(&layers()).unwrap();
    assert!(h.is_valid());
}

#[test]
fn test_hash_size_is_32_bytes() {
    assert_eq!(COMPOSITE_HASH_SIZE, 32);
}

#[test]
fn test_hash_bytes_length() {
    let h = CompositeHash::derive(&layers()).unwrap();
    assert_eq!(h.as_bytes().len(), 32);
}

#[test]
fn test_hash_fingerprint_nonzero() {
    let h = CompositeHash::derive(&layers()).unwrap();
    assert_ne!(h.fingerprint(), 0);
}

#[test]
fn test_hash_deterministic_same_layers() {
    let h1 = CompositeHash::derive(&layers()).unwrap();
    let h2 = CompositeHash::derive(&layers()).unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn test_hash_different_layers_different_hash() {
    let l1 = layers();
    let l2 = IdentityLayers::new(HW_FP, KERN_FP, OS_FP, 0xDEAD_BEEF_0000_0001);
    let h1 = CompositeHash::derive(&l1).unwrap();
    let h2 = CompositeHash::derive(&l2).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn test_hash_hw_change_changes_hash() {
    let l1 = layers();
    let l2 = IdentityLayers::new(0xA1E0_0000_0001_0002, KERN_FP, OS_FP, BIO_FP);
    let h1 = CompositeHash::derive(&l1).unwrap();
    let h2 = CompositeHash::derive(&l2).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn test_hash_stub_produces_valid_hash() {
    let h = CompositeHash::derive(&IdentityLayers::stub()).unwrap();
    assert!(h.is_valid());
}

// ── ThresholdRegistry tests (12) ─────────────────────────────────────

#[test]
fn test_threshold_enroll_all_three() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey, KEY_OS).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, KEY_EDB).unwrap();
    r.enroll(KeySlotId::OwnerKey, KEY_OWN).unwrap();
    assert!(r.is_complete());
}

#[test]
fn test_threshold_zero_fingerprint_rejected() {
    let mut r = ThresholdRegistry::new();
    assert!(r.enroll(KeySlotId::OsKey, 0).is_err());
}

#[test]
fn test_threshold_enrolled_count_zero() {
    let r = ThresholdRegistry::new();
    assert_eq!(r.enrolled_count(), 0);
}

#[test]
fn test_threshold_enrolled_count_three() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey, KEY_OS).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, KEY_EDB).unwrap();
    r.enroll(KeySlotId::OwnerKey, KEY_OWN).unwrap();
    assert_eq!(r.enrolled_count(), 3);
}

#[test]
fn test_threshold_all_three_keys_met() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey, KEY_OS).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, KEY_EDB).unwrap();
    r.enroll(KeySlotId::OwnerKey, KEY_OWN).unwrap();
    let result = r.check_threshold(&[KEY_OS, KEY_EDB, KEY_OWN]);
    assert_eq!(result, ThresholdResult::ThresholdMet);
}

#[test]
fn test_threshold_two_keys_partial_match() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey, KEY_OS).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, KEY_EDB).unwrap();
    r.enroll(KeySlotId::OwnerKey, KEY_OWN).unwrap();
    // Only two keys — threshold NOT met — data is noise
    let result = r.check_threshold(&[KEY_OS, KEY_EDB]);
    assert_eq!(result, ThresholdResult::PartialMatch(2));
}

#[test]
fn test_threshold_one_key_partial_match() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey, KEY_OS).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, KEY_EDB).unwrap();
    r.enroll(KeySlotId::OwnerKey, KEY_OWN).unwrap();
    let result = r.check_threshold(&[KEY_OS]);
    assert_eq!(result, ThresholdResult::PartialMatch(1));
}

#[test]
fn test_threshold_no_keys_no_match() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey, KEY_OS).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, KEY_EDB).unwrap();
    r.enroll(KeySlotId::OwnerKey, KEY_OWN).unwrap();
    assert_eq!(r.check_threshold(&[]), ThresholdResult::NoMatch);
}

#[test]
fn test_threshold_not_enrolled_returns_not_enrolled() {
    let r = ThresholdRegistry::new();
    assert_eq!(r.check_threshold(&[KEY_OS, KEY_EDB, KEY_OWN]), ThresholdResult::NotEnrolled);
}

#[test]
fn test_threshold_revoke_reduces_count() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey, KEY_OS).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, KEY_EDB).unwrap();
    r.enroll(KeySlotId::OwnerKey, KEY_OWN).unwrap();
    r.revoke(KeySlotId::OwnerKey);
    assert!(!r.slot_enrolled(KeySlotId::OwnerKey));
    assert_eq!(r.enrolled_count(), 2);
}

#[test]
fn test_threshold_revoked_key_breaks_threshold() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey, KEY_OS).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, KEY_EDB).unwrap();
    r.enroll(KeySlotId::OwnerKey, KEY_OWN).unwrap();
    r.revoke(KeySlotId::OwnerKey);
    // Even with all three fingerprints presented, threshold fails
    // because registry is incomplete
    assert_eq!(
        r.check_threshold(&[KEY_OS, KEY_EDB, KEY_OWN]),
        ThresholdResult::NotEnrolled
    );
}

#[test]
fn test_threshold_wrong_keys_no_match() {
    let mut r = ThresholdRegistry::new();
    r.enroll(KeySlotId::OsKey, KEY_OS).unwrap();
    r.enroll(KeySlotId::EdisonDbKey, KEY_EDB).unwrap();
    r.enroll(KeySlotId::OwnerKey, KEY_OWN).unwrap();
    assert_eq!(
        r.check_threshold(&[0xDEAD, 0xBEEF, 0xCAFE]),
        ThresholdResult::NoMatch
    );
}

// ── BindingEngine tests (10) ──────────────────────────────────────────

#[test]
fn test_binding_register_node_hash() {
    let mut e = BindingEngine::new();
    let h = CompositeHash::derive(&layers()).unwrap();
    assert!(e.register_node_hash(h).is_ok());
    assert!(e.is_registered());
}

#[test]
fn test_binding_stamp_without_registration_fails() {
    let mut e = BindingEngine::new();
    assert_eq!(e.stamp_outgoing(DataTier::Noise), Err(BindingError::NodeHashNotRegistered));
}

#[test]
fn test_binding_stamp_succeeds_after_registration() {
    let mut e = BindingEngine::new();
    e.register_node_hash(CompositeHash::derive(&layers()).unwrap()).unwrap();
    assert!(e.stamp_outgoing(DataTier::Noise).is_ok());
}

#[test]
fn test_binding_stamp_increments_seq() {
    let mut e = BindingEngine::new();
    e.register_node_hash(CompositeHash::derive(&layers()).unwrap()).unwrap();
    e.stamp_outgoing(DataTier::Noise).unwrap();
    e.stamp_outgoing(DataTier::Personal).unwrap();
    assert_eq!(e.current_seq(), 2);
}

#[test]
fn test_binding_bound_count_increments() {
    let mut e = BindingEngine::new();
    e.register_node_hash(CompositeHash::derive(&layers()).unwrap()).unwrap();
    e.stamp_outgoing(DataTier::Noise).unwrap();
    e.stamp_outgoing(DataTier::Noise).unwrap();
    assert_eq!(e.bound_count(), 2);
}

#[test]
fn test_binding_verify_valid_header() {
    let mut e = BindingEngine::new();
    let h = CompositeHash::derive(&layers()).unwrap();
    e.register_node_hash(h).unwrap();
    let header = e.stamp_outgoing(DataTier::Noise).unwrap();
    assert!(e.verify_incoming(&header).is_ok());
}

#[test]
fn test_binding_verify_invalid_magic_fails() {
    let mut e = BindingEngine::new();
    e.register_node_hash(CompositeHash::derive(&layers()).unwrap()).unwrap();
    let mut h = e.stamp_outgoing(DataTier::Noise).unwrap();
    h.magic = 0xDEAD;
    assert_eq!(e.verify_incoming(&h), Err(BindingError::InvalidMagic));
}

#[test]
fn test_binding_violation_count_increments() {
    let mut e = BindingEngine::new();
    e.register_node_hash(CompositeHash::derive(&layers()).unwrap()).unwrap();
    let mut h = e.stamp_outgoing(DataTier::Noise).unwrap();
    h.magic = 0xDEAD;
    e.verify_incoming(&h).unwrap_err();
    assert_eq!(e.violation_count(), 1);
}

#[test]
fn test_binding_header_magic_constant() {
    assert_eq!(BoundPacketHeader::MAGIC, 0xA1E0u16);
}

#[test]
fn test_binding_header_size_48_bytes() {
    assert_eq!(BoundPacketHeader::SIZE, 48);
}

// ── SOMA engine tests (10) ────────────────────────────────────────────

#[test]
fn test_soma_commission_succeeds() {
    let mut s = Soma::new();
    assert!(s.commission(layers(), KEY_OS, KEY_EDB, KEY_OWN).is_ok());
}

#[test]
fn test_soma_is_commissioned_after_commission() {
    let s = commissioned_soma();
    assert!(s.is_commissioned());
}

#[test]
fn test_soma_threshold_complete_after_commission() {
    let s = commissioned_soma();
    assert!(s.threshold_complete());
}

#[test]
fn test_soma_hash_present_after_commission() {
    let s = commissioned_soma();
    assert!(s.composite_hash().is_some());
}

#[test]
fn test_soma_stamp_succeeds() {
    let mut s = commissioned_soma();
    assert!(s.stamp(DataTier::Noise).is_ok());
}

#[test]
fn test_soma_stamp_and_verify() {
    let mut s = commissioned_soma();
    let header = s.stamp(DataTier::Personal).unwrap();
    assert!(s.verify(&header).is_ok());
}

#[test]
fn test_soma_stamp_without_commission_fails() {
    let mut s = Soma::new();
    assert!(matches!(s.stamp(DataTier::Noise), Err(SomaError::NotCommissioned)));
}

#[test]
fn test_soma_threshold_met_all_keys() {
    let s = commissioned_soma();
    assert_eq!(
        s.check_threshold(&[KEY_OS, KEY_EDB, KEY_OWN]),
        ThresholdResult::ThresholdMet
    );
}

#[test]
fn test_soma_threshold_two_keys_insufficient() {
    let s = commissioned_soma();
    assert_eq!(
        s.check_threshold(&[KEY_OS, KEY_EDB]),
        ThresholdResult::PartialMatch(2)
    );
}

#[test]
fn test_soma_bound_count_after_stamps() {
    let mut s = commissioned_soma();
    s.stamp(DataTier::Noise).unwrap();
    s.stamp(DataTier::Personal).unwrap();
    s.stamp(DataTier::Critical).unwrap();
    assert_eq!(s.bound_count(), 3);
}

// ── BindingMode tests (3) ─────────────────────────────────────────────

#[test]
fn test_binding_mode_critical_minimum_is_full() {
    use asl_soma::binding::BindingMode;
    assert_eq!(
        BindingMode::minimum_for_tier(DataTier::Critical),
        BindingMode::Full
    );
}

#[test]
fn test_binding_mode_critical_cannot_be_open() {
    use asl_soma::binding::BindingMode;
    assert!(!BindingMode::Open.satisfies_tier(DataTier::Critical));
}

#[test]
fn test_binding_mode_noise_allows_provenance() {
    use asl_soma::binding::BindingMode;
    assert!(BindingMode::Provenance.satisfies_tier(DataTier::Noise));
}
