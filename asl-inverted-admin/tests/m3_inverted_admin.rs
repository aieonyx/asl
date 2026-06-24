// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ASL-M3 test suite — Inverted-Admin PD
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_inverted_admin::admin::{AdminResult, InvertedAdmin};
use asl_inverted_admin::action::{ActionCounter, AdminAction, AdminActionClass, CounterError};
use asl_inverted_admin::devmode::{self, DevModeResult};
use asl_inverted_admin::dual_key::{AuthToken, DualKeyError, DualKeyRegistry};
use asl_common::pd::PdId;

const FP_A: u64 = 0xA1E0_0000_0001_0001;
const FP_B: u64 = 0xA1E0_0000_0001_0002;
const SIG: [u8; 64] = {
    let mut s = [0u8; 64];
    s[0] = 0x01;
    s
};

fn make_action(class: AdminActionClass, counter: u64) -> AdminAction {
    AdminAction::new(class, counter, 0x02, PdId::Genesis as u8)
}

fn make_token(slot: u8, counter: u64) -> AuthToken {
    AuthToken::new(slot, counter, SIG)
}

fn ready_admin() -> InvertedAdmin {
    let mut a = InvertedAdmin::new();
    a.register_key_slot(0x01, FP_A).unwrap();
    a.register_key_slot(0x02, FP_B).unwrap();
    a.register_requestor(PdId::Genesis as u8).unwrap();
    a
}

// ── DevMode tests (5) ────────────────────────────────────────────────

#[test]
fn test_devmode_check_returns_not_active() {
    assert_eq!(devmode::check(), DevModeResult::NotActive);
}

#[test]
fn test_devmode_is_active_false() {
    assert!(!devmode::is_active());
}

#[test]
fn test_devmode_assert_not_devmode_does_not_panic() {
    devmode::assert_not_devmode(); // must not panic
}

#[test]
fn test_devmode_result_variants_distinct() {
    assert_ne!(DevModeResult::NotActive, DevModeResult::Rejected);
}

#[test]
fn test_devmode_not_active_is_safe_default() {
    // In production: ambiguity → Rejected. Stub → NotActive.
    // This test documents the stub contract.
    let result = devmode::check();
    assert_eq!(result, DevModeResult::NotActive);
}

// ── Dual-key registry tests (12) ─────────────────────────────────────

#[test]
fn test_dualkey_register_slot_succeeds() {
    let mut r = DualKeyRegistry::new();
    assert!(r.register_slot(0x01, FP_A).is_ok());
}

#[test]
fn test_dualkey_register_duplicate_fails() {
    let mut r = DualKeyRegistry::new();
    r.register_slot(0x01, FP_A).unwrap();
    assert_eq!(r.register_slot(0x01, FP_B), Err(DualKeyError::SlotAlreadyRegistered));
}

#[test]
fn test_dualkey_zero_slot_id_rejected() {
    let mut r = DualKeyRegistry::new();
    assert_eq!(r.register_slot(0x00, FP_A), Err(DualKeyError::InvalidSlotId));
}

#[test]
fn test_dualkey_zero_fingerprint_rejected() {
    let mut r = DualKeyRegistry::new();
    assert_eq!(r.register_slot(0x01, 0), Err(DualKeyError::ZeroFingerprint));
}

#[test]
fn test_dualkey_authorize_succeeds() {
    let mut r = DualKeyRegistry::new();
    r.register_slot(0x01, FP_A).unwrap();
    r.register_slot(0x02, FP_B).unwrap();
    let ta = make_token(0x01, 1);
    let tb = make_token(0x02, 1);
    assert!(r.authorize(&ta, &tb, 1).is_ok());
}

#[test]
fn test_dualkey_same_slot_rejected() {
    let mut r = DualKeyRegistry::new();
    r.register_slot(0x01, FP_A).unwrap();
    let ta = make_token(0x01, 1);
    let tb = make_token(0x01, 1);
    assert_eq!(r.authorize(&ta, &tb, 1), Err(DualKeyError::SameSlot));
}

#[test]
fn test_dualkey_unknown_slot_rejected() {
    let mut r = DualKeyRegistry::new();
    r.register_slot(0x01, FP_A).unwrap();
    let ta = make_token(0x01, 1);
    let tb = make_token(0x99, 1);
    assert_eq!(r.authorize(&ta, &tb, 1), Err(DualKeyError::UnknownSlot));
}

#[test]
fn test_dualkey_counter_mismatch_rejected() {
    let mut r = DualKeyRegistry::new();
    r.register_slot(0x01, FP_A).unwrap();
    r.register_slot(0x02, FP_B).unwrap();
    let ta = make_token(0x01, 1);
    let tb = make_token(0x02, 2); // different counter
    assert_eq!(r.authorize(&ta, &tb, 1), Err(DualKeyError::CounterMismatch));
}

#[test]
fn test_dualkey_zero_sig_rejected() {
    let mut r = DualKeyRegistry::new();
    r.register_slot(0x01, FP_A).unwrap();
    r.register_slot(0x02, FP_B).unwrap();
    let ta = AuthToken::new(0x01, 1, [0u8; 64]); // zero sig
    let tb = make_token(0x02, 1);
    assert_eq!(r.authorize(&ta, &tb, 1), Err(DualKeyError::InvalidSignature));
}

#[test]
fn test_dualkey_deactivate_slot() {
    let mut r = DualKeyRegistry::new();
    r.register_slot(0x01, FP_A).unwrap();
    r.deactivate_slot(0x01).unwrap();
    assert!(!r.slot_active(0x01));
}

#[test]
fn test_dualkey_slot_count() {
    let mut r = DualKeyRegistry::new();
    r.register_slot(0x01, FP_A).unwrap();
    r.register_slot(0x02, FP_B).unwrap();
    assert_eq!(r.slot_count(), 2);
}

#[test]
fn test_dualkey_inactive_slot_rejected() {
    let mut r = DualKeyRegistry::new();
    r.register_slot(0x01, FP_A).unwrap();
    r.register_slot(0x02, FP_B).unwrap();
    r.deactivate_slot(0x01).unwrap();
    let ta = make_token(0x01, 1);
    let tb = make_token(0x02, 1);
    assert_eq!(r.authorize(&ta, &tb, 1), Err(DualKeyError::SlotInactive));
}

// ── Action counter tests (8) ──────────────────────────────────────────

#[test]
fn test_action_counter_register() {
    let mut c = ActionCounter::new();
    assert!(c.register(0x01).is_ok());
}

#[test]
fn test_action_counter_duplicate_fails() {
    let mut c = ActionCounter::new();
    c.register(0x01).unwrap();
    assert_eq!(c.register(0x01), Err(CounterError::AlreadyRegistered));
}

#[test]
fn test_action_counter_advance() {
    let mut c = ActionCounter::new();
    c.register(0x01).unwrap();
    assert!(c.validate_and_advance(0x01, 1).is_ok());
}

#[test]
fn test_action_counter_replay_rejected() {
    let mut c = ActionCounter::new();
    c.register(0x01).unwrap();
    c.validate_and_advance(0x01, 1).unwrap();
    assert!(matches!(
        c.validate_and_advance(0x01, 1),
        Err(CounterError::Replay { .. })
    ));
}

#[test]
fn test_action_counter_unknown_pd() {
    let mut c = ActionCounter::new();
    assert_eq!(c.validate_and_advance(0x99, 1), Err(CounterError::UnknownPd));
}

#[test]
fn test_action_all_classes_require_dual_key() {
    let classes = [
        AdminActionClass::PdLifecycle,
        AdminActionClass::CapabilityMgmt,
        AdminActionClass::TrustGraphEdit,
        AdminActionClass::KeyCeremony,
        AdminActionClass::TierBoundary,
        AdminActionClass::SovereignHalt,
    ];
    for class in classes {
        let action = AdminAction::new(class, 1, 0x02, 0x01);
        assert!(action.requires_dual_key(),
            "{:?} must require dual-key", class);
    }
}

#[test]
fn test_action_system_wide_target() {
    let action = AdminAction::new(AdminActionClass::SovereignHalt, 1, 0xFF, 0x01);
    assert!(action.is_system_wide());
}

#[test]
fn test_action_specific_target_not_system_wide() {
    let action = AdminAction::new(AdminActionClass::PdLifecycle, 1, 0x02, 0x01);
    assert!(!action.is_system_wide());
}

// ── InvertedAdmin engine tests (10) ──────────────────────────────────

#[test]
fn test_admin_authorize_succeeds() {
    let mut a = ready_admin();
    let action = make_action(AdminActionClass::PdLifecycle, 1);
    let ta = make_token(0x01, 1);
    let tb = make_token(0x02, 1);
    assert_eq!(a.authorize(&action, &ta, &tb), AdminResult::Authorized);
}

#[test]
fn test_admin_authorize_increments_count() {
    let mut a = ready_admin();
    let action = make_action(AdminActionClass::PdLifecycle, 1);
    a.authorize(&action, &make_token(0x01, 1), &make_token(0x02, 1));
    assert_eq!(a.authorized_count(), 1);
}

#[test]
fn test_admin_replay_rejected() {
    let mut a = ready_admin();
    let action1 = make_action(AdminActionClass::PdLifecycle, 1);
    a.authorize(&action1, &make_token(0x01, 1), &make_token(0x02, 1));
    let action2 = make_action(AdminActionClass::PdLifecycle, 1);
    let result = a.authorize(&action2, &make_token(0x01, 1), &make_token(0x02, 1));
    assert!(matches!(result, AdminResult::CounterFailed(_)));
}

#[test]
fn test_admin_same_slot_rejected() {
    let mut a = ready_admin();
    let action = make_action(AdminActionClass::PdLifecycle, 1);
    let ta = make_token(0x01, 1);
    let tb = make_token(0x01, 1);
    assert!(matches!(a.authorize(&action, &ta, &tb), AdminResult::DualKeyFailed(_)));
}

#[test]
fn test_admin_unknown_requestor_rejected() {
    let mut a = InvertedAdmin::new();
    a.register_key_slot(0x01, FP_A).unwrap();
    a.register_key_slot(0x02, FP_B).unwrap();
    // No requestor registered
    let action = make_action(AdminActionClass::PdLifecycle, 1);
    let result = a.authorize(&action, &make_token(0x01, 1), &make_token(0x02, 1));
    assert!(matches!(result, AdminResult::CounterFailed(_)));
}

#[test]
fn test_admin_rejected_count_increments() {
    let mut a = ready_admin();
    let action = make_action(AdminActionClass::PdLifecycle, 1);
    let ta = make_token(0x01, 1);
    let tb = make_token(0x01, 1); // same slot — will reject
    a.authorize(&action, &ta, &tb);
    assert_eq!(a.rejected_count(), 1);
}

#[test]
fn test_admin_sequential_actions_succeed() {
    let mut a = ready_admin();
    for i in 1..=5u64 {
        let action = make_action(AdminActionClass::CapabilityMgmt, i);
        let result = a.authorize(&action, &make_token(0x01, i), &make_token(0x02, i));
        assert_eq!(result, AdminResult::Authorized, "action {} failed", i);
    }
    assert_eq!(a.authorized_count(), 5);
}

#[test]
fn test_admin_slot_count_after_registration() {
    let a = ready_admin();
    assert_eq!(a.slot_count(), 2);
}

#[test]
fn test_admin_zero_sig_dual_key_fails() {
    let mut a = ready_admin();
    let action = make_action(AdminActionClass::PdLifecycle, 1);
    let ta = AuthToken::new(0x01, 1, [0u8; 64]);
    let tb = make_token(0x02, 1);
    assert!(matches!(a.authorize(&action, &ta, &tb), AdminResult::DualKeyFailed(_)));
}

#[test]
fn test_admin_sovereign_halt_requires_dual_key() {
    let mut a = ready_admin();
    let action = make_action(AdminActionClass::SovereignHalt, 1);
    // Even halt requires dual-key — no escape hatch
    let result = a.authorize(&action, &make_token(0x01, 1), &make_token(0x02, 1));
    assert_eq!(result, AdminResult::Authorized);
}
