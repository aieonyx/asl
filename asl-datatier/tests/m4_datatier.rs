// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ASL-M4 test suite — DataTier-Enforcer PD
// Target: 50+ tests, 0 failures
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_datatier::enforcer::DataTierEnforcer;
use asl_datatier::flow::{DataFlow, FlowResult};
use asl_datatier::grant::{GrantRegistry, TierGrant, GrantError};
use asl_datatier::audit::{AuditLog, AuditEvent};
use asl_datatier::erasure::{ErasureQueue, ErasureError};
use asl_common::datatier::DataTier;

const VAULT_PD: u8 = 0x10; // dedicated vault PD
const SIG: [u8; 64] = { let mut s = [0u8; 64]; s[0] = 0x01; s };

fn make_flow(src: u8, dst: u8, src_tier: DataTier, dst_tier: DataTier) -> DataFlow {
    DataFlow::new(src, dst, src_tier, dst_tier, 64)
}

fn make_grant(pd: u8, src: DataTier, dst: DataTier, seq: u64) -> TierGrant {
    TierGrant::new(pd, src, dst, seq, SIG)
}

fn enforcer() -> DataTierEnforcer {
    DataTierEnforcer::new(VAULT_PD)
}

// ── DataFlow tests (10) ───────────────────────────────────────────────

#[test]
fn test_flow_same_tier_not_upgrade() {
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Noise);
    assert!(!f.is_tier_upgrade());
}

#[test]
fn test_flow_upgrade_noise_to_personal() {
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Personal);
    assert!(f.is_tier_upgrade());
}

#[test]
fn test_flow_upgrade_noise_to_critical() {
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Critical);
    assert!(f.is_tier_upgrade());
}

#[test]
fn test_flow_downgrade_is_not_upgrade() {
    let f = make_flow(0x01, 0x02, DataTier::Critical, DataTier::Noise);
    assert!(!f.is_tier_upgrade());
    assert!(f.is_downgrade());
}

#[test]
fn test_flow_internal_same_pd() {
    let f = make_flow(0x01, 0x01, DataTier::Critical, DataTier::Noise);
    assert!(f.is_internal());
}

#[test]
fn test_flow_cross_pd_not_internal() {
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Noise);
    assert!(!f.is_internal());
}

#[test]
fn test_flow_involves_critical_src() {
    let f = make_flow(0x01, 0x02, DataTier::Critical, DataTier::Noise);
    assert!(f.involves_critical());
}

#[test]
fn test_flow_involves_critical_dst() {
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Critical);
    assert!(f.involves_critical());
}

#[test]
fn test_flow_noise_to_noise_no_critical() {
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Noise);
    assert!(!f.involves_critical());
}

#[test]
fn test_flow_size_preserved() {
    let f = DataFlow::new(0x01, 0x02, DataTier::Noise, DataTier::Noise, 1024);
    assert_eq!(f.size, 1024);
}

// ── Grant registry tests (10) ─────────────────────────────────────────

#[test]
fn test_grant_register_succeeds() {
    let mut r = GrantRegistry::new();
    let g = make_grant(0x01, DataTier::Noise, DataTier::Personal, 1);
    assert!(r.register(g).is_ok());
}

#[test]
fn test_grant_invalid_same_tier_rejected() {
    let mut r = GrantRegistry::new();
    // Same tier is not an upgrade — grant is invalid
    let mut g = make_grant(0x01, DataTier::Noise, DataTier::Noise, 1);
    g.active = true;
    assert_eq!(r.register(g), Err(GrantError::InvalidGrant));
}

#[test]
fn test_grant_zero_seq_rejected() {
    let mut r = GrantRegistry::new();
    let g = make_grant(0x01, DataTier::Noise, DataTier::Personal, 0);
    assert_eq!(r.register(g), Err(GrantError::InvalidGrant));
}

#[test]
fn test_grant_zero_sig_rejected() {
    let mut r = GrantRegistry::new();
    let g = TierGrant::new(0x01, DataTier::Noise, DataTier::Personal, 1, [0u8; 64]);
    assert_eq!(r.register(g), Err(GrantError::InvalidGrant));
}

#[test]
fn test_grant_replay_rejected() {
    let mut r = GrantRegistry::new();
    r.register(make_grant(0x01, DataTier::Noise, DataTier::Personal, 5)).unwrap();
    let replay = make_grant(0x01, DataTier::Noise, DataTier::Personal, 3);
    assert_eq!(r.register(replay), Err(GrantError::ReplayDetected));
}

#[test]
fn test_grant_lookup_finds_matching() {
    let mut r = GrantRegistry::new();
    r.register(make_grant(0x01, DataTier::Noise, DataTier::Personal, 1)).unwrap();
    assert!(r.lookup(0x01, DataTier::Noise, DataTier::Personal).is_some());
}

#[test]
fn test_grant_lookup_wrong_pd_misses() {
    let mut r = GrantRegistry::new();
    r.register(make_grant(0x01, DataTier::Noise, DataTier::Personal, 1)).unwrap();
    assert!(r.lookup(0x02, DataTier::Noise, DataTier::Personal).is_none());
}

#[test]
fn test_grant_revoke_all_for_pd() {
    let mut r = GrantRegistry::new();
    r.register(make_grant(0x01, DataTier::Noise, DataTier::Personal, 1)).unwrap();
    let revoked = r.revoke_all(0x01);
    assert_eq!(revoked, 1);
    assert!(r.lookup(0x01, DataTier::Noise, DataTier::Personal).is_none());
}

#[test]
fn test_grant_revoke_other_pd_unaffected() {
    let mut r = GrantRegistry::new();
    r.register(make_grant(0x01, DataTier::Noise, DataTier::Personal, 1)).unwrap();
    r.register(make_grant(0x02, DataTier::Noise, DataTier::Personal, 2)).unwrap();
    r.revoke_all(0x01);
    assert!(r.lookup(0x02, DataTier::Noise, DataTier::Personal).is_some());
}

#[test]
fn test_grant_active_count() {
    let mut r = GrantRegistry::new();
    r.register(make_grant(0x01, DataTier::Noise, DataTier::Personal, 1)).unwrap();
    r.register(make_grant(0x02, DataTier::Noise, DataTier::Critical, 2)).unwrap();
    assert_eq!(r.active_count(), 2);
}

// ── Enforcer flow tests (12) ──────────────────────────────────────────

#[test]
fn test_enforcer_same_tier_permitted() {
    let mut e = enforcer();
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Noise);
    assert_eq!(e.check_flow(&f), FlowResult::Permitted);
}

#[test]
fn test_enforcer_downgrade_permitted() {
    let mut e = enforcer();
    // Downgrade must originate from vault PD — Critical data
    // cannot leave a non-vault PD even at lower tier context.
    // Vault PD downgrading Critical→Noise is permitted.
    let f = make_flow(VAULT_PD, 0x02, DataTier::Critical, DataTier::Noise);
    assert_eq!(e.check_flow(&f), FlowResult::Permitted);
}

#[test]
fn test_enforcer_internal_flow_permitted() {
    let mut e = enforcer();
    let f = make_flow(0x01, 0x01, DataTier::Critical, DataTier::Critical);
    assert_eq!(e.check_flow(&f), FlowResult::Permitted);
}

#[test]
fn test_enforcer_upgrade_without_grant_blocked() {
    let mut e = enforcer();
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Personal);
    assert_eq!(e.check_flow(&f), FlowResult::RequiresGrant);
}

#[test]
fn test_enforcer_critical_leaving_non_vault_blocked() {
    let mut e = enforcer();
    // src_pd is NOT the vault PD
    let f = make_flow(0x01, 0x02, DataTier::Critical, DataTier::Critical);
    assert_eq!(e.check_flow(&f), FlowResult::CriticalVaultViolation);
}

#[test]
fn test_enforcer_critical_from_vault_downgrade_permitted() {
    let mut e = enforcer();
    // Vault PD can send Critical data at lower tier context
    let f = make_flow(VAULT_PD, 0x02, DataTier::Critical, DataTier::Noise);
    assert_eq!(e.check_flow(&f), FlowResult::Permitted);
}

#[test]
fn test_enforcer_upgrade_with_grant_permitted() {
    let mut e = enforcer();
    e.register_grant(make_grant(0x01, DataTier::Noise, DataTier::Personal, 1)).unwrap();
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Personal);
    assert_eq!(e.check_flow(&f), FlowResult::PermittedWithGrant);
}

#[test]
fn test_enforcer_audit_records_permitted() {
    let mut e = enforcer();
    let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Noise);
    e.check_flow(&f);
    assert!(e.audit_count() > 0);
}

#[test]
fn test_enforcer_audit_records_violation() {
    let mut e = enforcer();
    let f = make_flow(0x01, 0x02, DataTier::Critical, DataTier::Critical);
    e.check_flow(&f);
    assert!(e.audit_count() > 0);
}

#[test]
fn test_enforcer_vault_pd_identity() {
    let e = enforcer();
    assert_eq!(e.vault_pd(), VAULT_PD);
}

#[test]
fn test_enforcer_multiple_flows_audit_all() {
    let mut e = enforcer();
    for _ in 0..5 {
        let f = make_flow(0x01, 0x02, DataTier::Noise, DataTier::Noise);
        e.check_flow(&f);
    }
    assert_eq!(e.audit_count(), 5);
}

#[test]
fn test_enforcer_grant_registered_increments_count() {
    let mut e = enforcer();
    e.register_grant(make_grant(0x01, DataTier::Noise, DataTier::Personal, 1)).unwrap();
    assert_eq!(e.grant_count(), 1);
}

// ── Audit log tests (8) ───────────────────────────────────────────────

#[test]
fn test_audit_append_succeeds() {
    let mut log = AuditLog::new();
    assert!(log.append(
        AuditEvent::FlowPermitted,
        0x01, 0x02,
        DataTier::Noise, DataTier::Noise,
    ).is_ok());
}

#[test]
fn test_audit_count_increments() {
    let mut log = AuditLog::new();
    log.append(AuditEvent::FlowPermitted, 0x01, 0x02, DataTier::Noise, DataTier::Noise).unwrap();
    log.append(AuditEvent::FlowBlocked, 0x01, 0x02, DataTier::Noise, DataTier::Personal).unwrap();
    assert_eq!(log.count(), 2);
}

#[test]
fn test_audit_seq_monotonic() {
    let mut log = AuditLog::new();
    let s1 = log.append(AuditEvent::FlowPermitted, 0x01, 0x02, DataTier::Noise, DataTier::Noise).unwrap();
    let s2 = log.append(AuditEvent::FlowBlocked, 0x01, 0x02, DataTier::Noise, DataTier::Personal).unwrap();
    assert!(s2 > s1);
}

#[test]
fn test_audit_count_by_event() {
    let mut log = AuditLog::new();
    log.append(AuditEvent::FlowPermitted, 0x01, 0x02, DataTier::Noise, DataTier::Noise).unwrap();
    log.append(AuditEvent::FlowPermitted, 0x01, 0x02, DataTier::Noise, DataTier::Noise).unwrap();
    log.append(AuditEvent::TierViolation, 0x01, 0x02, DataTier::Critical, DataTier::Noise).unwrap();
    assert_eq!(log.count_by_event(AuditEvent::FlowPermitted), 2);
    assert_eq!(log.count_by_event(AuditEvent::TierViolation), 1);
}

#[test]
fn test_audit_last_entry() {
    let mut log = AuditLog::new();
    log.append(AuditEvent::FlowPermitted, 0x01, 0x02, DataTier::Noise, DataTier::Noise).unwrap();
    log.append(AuditEvent::TierViolation, 0x03, 0x04, DataTier::Critical, DataTier::Noise).unwrap();
    let last = log.last().unwrap();
    assert_eq!(last.event, AuditEvent::TierViolation);
}

#[test]
fn test_audit_empty_last_none() {
    let log = AuditLog::new();
    assert!(log.last().is_none());
}

#[test]
fn test_audit_initial_count_zero() {
    let log = AuditLog::new();
    assert_eq!(log.count(), 0);
}

#[test]
fn test_audit_initial_seq_zero() {
    let log = AuditLog::new();
    assert_eq!(log.current_seq(), 0);
}

// ── Erasure queue tests (10) ──────────────────────────────────────────

#[test]
fn test_erasure_submit_succeeds() {
    let mut q = ErasureQueue::new();
    assert!(q.submit(0x01, DataTier::Critical).is_ok());
}

#[test]
fn test_erasure_submit_returns_id() {
    let mut q = ErasureQueue::new();
    let id = q.submit(0x01, DataTier::Critical).unwrap();
    assert!(id > 0);
}

#[test]
fn test_erasure_ids_monotonic() {
    let mut q = ErasureQueue::new();
    let id1 = q.submit(0x01, DataTier::Critical).unwrap();
    let id2 = q.submit(0x01, DataTier::Personal).unwrap();
    assert!(id2 > id1);
}

#[test]
fn test_erasure_pending_count() {
    let mut q = ErasureQueue::new();
    q.submit(0x01, DataTier::Critical).unwrap();
    q.submit(0x01, DataTier::Personal).unwrap();
    assert_eq!(q.pending_count(), 2);
}

#[test]
fn test_erasure_authorize_then_complete() {
    let mut q = ErasureQueue::new();
    let id = q.submit(0x01, DataTier::Critical).unwrap();
    q.authorize(id).unwrap();
    assert!(q.complete(id).is_ok());
}

#[test]
fn test_erasure_complete_without_authorize_fails() {
    let mut q = ErasureQueue::new();
    let id = q.submit(0x01, DataTier::Critical).unwrap();
    assert_eq!(q.complete(id), Err(ErasureError::NotAuthorized));
}

#[test]
fn test_erasure_reject_request() {
    let mut q = ErasureQueue::new();
    let id = q.submit(0x01, DataTier::Critical).unwrap();
    assert!(q.reject(id).is_ok());
}

#[test]
fn test_erasure_unknown_id_fails() {
    let mut q = ErasureQueue::new();
    assert_eq!(q.authorize(999), Err(ErasureError::NotFound));
}

#[test]
fn test_erasure_critical_requires_dual_auth() {
    use asl_datatier::erasure::ErasureRequest;
    let r = ErasureRequest::new(0x01, DataTier::Critical, 1);
    assert!(r.requires_dual_auth());
}

#[test]
fn test_erasure_noise_no_dual_auth() {
    use asl_datatier::erasure::ErasureRequest;
    let r = ErasureRequest::new(0x01, DataTier::Noise, 1);
    assert!(!r.requires_dual_auth());
}
