// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ASL-M3 test suite — TrustGraph-Gate PD
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_trustgraph::graph::{GraphResult, TrustGraph};
use asl_trustgraph::token::{CapToken, CapabilityType, TokenResult, validate};
use asl_trustgraph::trust_score::{
    TrustRegistry, MANDATORY_TRUST, BASELINE_TRUST, MAX_TRUST,
};
use asl_common::datatier::DataTier;
use asl_common::pd::PdId;

const SIG: [u8; 64] = { let mut s = [0u8; 64]; s[0] = 0x01; s };

fn make_token(src: PdId, dst: PdId, cap: CapabilityType, seq: u64) -> CapToken {
    CapToken::new(src as u8, dst as u8, cap, DataTier::Noise, seq, SIG)
}

fn commissioned_graph() -> TrustGraph {
    let mut g = TrustGraph::new();
    g.commission().expect("commission must succeed");
    g
}

// ── Token validation tests (8) ───────────────────────────────────────

#[test]
fn test_token_valid() {
    let t = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Execute, 1);
    assert_eq!(validate(&t), TokenResult::Valid);
}

#[test]
fn test_token_self_grant_rejected() {
    let t = CapToken::new(0x01, 0x01, CapabilityType::Read, DataTier::Noise, 1, SIG);
    assert_eq!(validate(&t), TokenResult::SelfGrant);
}

#[test]
fn test_token_zero_seq_rejected() {
    let t = CapToken::new(0x01, 0x02, CapabilityType::Read, DataTier::Noise, 0, SIG);
    assert_eq!(validate(&t), TokenResult::ZeroSeq);
}

#[test]
fn test_token_zero_sig_rejected() {
    let t = CapToken::new(0x01, 0x02, CapabilityType::Read, DataTier::Noise, 1, [0u8; 64]);
    assert_eq!(validate(&t), TokenResult::ZeroSignature);
}

#[test]
fn test_token_structurally_valid() {
    let t = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Execute, 1);
    assert!(t.is_structurally_valid());
}

#[test]
fn test_token_grants_tier_upgrade() {
    let t = CapToken::new(0x01, 0x02, CapabilityType::TierUpgrade, DataTier::Personal, 1, SIG);
    assert!(t.grants_tier_upgrade());
}

#[test]
fn test_token_non_tier_upgrade_does_not_grant_upgrade() {
    let t = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Execute, 1);
    assert!(!t.grants_tier_upgrade());
}

#[test]
fn test_token_capability_types_distinct() {
    assert_ne!(CapabilityType::Read as u8, CapabilityType::Write as u8);
    assert_ne!(CapabilityType::Execute as u8, CapabilityType::Delegate as u8);
    assert_ne!(CapabilityType::TierUpgrade as u8, CapabilityType::AdminAction as u8);
}

// ── Trust graph tests (12) ────────────────────────────────────────────

#[test]
fn test_graph_commission_succeeds() {
    let mut g = TrustGraph::new();
    assert!(g.commission().is_ok());
}

#[test]
fn test_graph_commission_registers_six_pds() {
    let g = commissioned_graph();
    assert_eq!(g.registered_pd_count(), 6);
}

#[test]
fn test_graph_commission_creates_edges() {
    let g = commissioned_graph();
    assert!(g.edge_count() > 0);
}

#[test]
fn test_graph_validate_token_granted() {
    let mut g = commissioned_graph();
    let t = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Execute, 1);
    assert_eq!(g.validate_token(&t), GraphResult::Granted);
}

#[test]
fn test_graph_validate_token_no_edge() {
    let mut g = commissioned_graph();
    // No edge for this capability type
    let t = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Write, 1);
    assert_eq!(g.validate_token(&t), GraphResult::NoEdge);
}

#[test]
fn test_graph_validate_replay_rejected() {
    let mut g = commissioned_graph();
    let t1 = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Execute, 1);
    g.validate_token(&t1);
    let t2 = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Execute, 1);
    assert_eq!(g.validate_token(&t2), GraphResult::ReplayDetected);
}

#[test]
fn test_graph_validate_seq_advance() {
    let mut g = commissioned_graph();
    let t1 = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Execute, 1);
    g.validate_token(&t1);
    let t2 = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Execute, 2);
    assert_eq!(g.validate_token(&t2), GraphResult::Granted);
}

#[test]
fn test_graph_invalid_token_rejected() {
    let mut g = commissioned_graph();
    let t = CapToken::new(0x01, 0x01, CapabilityType::Execute, DataTier::Noise, 1, SIG);
    assert!(matches!(g.validate_token(&t), GraphResult::InvalidToken(_)));
}

#[test]
fn test_graph_mandatory_pd_trust_score_max() {
    let g = commissioned_graph();
    assert_eq!(g.trust_score(PdId::Genesis as u8), Some(MANDATORY_TRUST));
    assert_eq!(g.trust_score(PdId::ArpiBroker as u8), Some(MANDATORY_TRUST));
}

#[test]
fn test_graph_revoke_edge() {
    let mut g = commissioned_graph();
    assert!(g.revoke_edge(
        PdId::Genesis as u8,
        PdId::ArpiBroker as u8,
        CapabilityType::Execute,
    ).is_ok());
    // After revocation token must be rejected
    let t = make_token(PdId::Genesis, PdId::ArpiBroker, CapabilityType::Execute, 1);
    assert_eq!(g.validate_token(&t), GraphResult::NoEdge);
}

#[test]
fn test_graph_add_custom_edge() {
    let mut g = commissioned_graph();
    g.add_edge(
        PdId::AxonBridge as u8,
        PdId::DataTierEnforcer as u8,
        CapabilityType::Read,
    ).unwrap();
    let edge_count = g.edge_count();
    assert!(edge_count > 5);
}

#[test]
fn test_graph_commission_double_fails() {
    let mut g = TrustGraph::new();
    g.commission().unwrap();
    assert!(g.commission().is_err());
}

// ── Trust score tests (8) ─────────────────────────────────────────────

#[test]
fn test_trust_mandatory_starts_at_max() {
    let mut r = TrustRegistry::new();
    r.register(0x01, true).unwrap();
    assert_eq!(r.score(0x01), Some(MANDATORY_TRUST));
}

#[test]
fn test_trust_optional_starts_at_baseline() {
    let mut r = TrustRegistry::new();
    r.register(0x01, false).unwrap();
    assert_eq!(r.score(0x01), Some(BASELINE_TRUST));
}

#[test]
fn test_trust_grant_increases_score() {
    let mut r = TrustRegistry::new();
    r.register(0x01, false).unwrap();
    let before = r.score(0x01).unwrap();
    r.record_grant(0x01).unwrap();
    assert!(r.score(0x01).unwrap() >= before);
}

#[test]
fn test_trust_revocation_decreases_score() {
    let mut r = TrustRegistry::new();
    r.register(0x01, false).unwrap();
    let before = r.score(0x01).unwrap();
    r.record_revocation(0x01).unwrap();
    assert!(r.score(0x01).unwrap() < before);
}

#[test]
fn test_trust_mandatory_score_does_not_exceed_max() {
    let mut r = TrustRegistry::new();
    r.register(0x01, true).unwrap();
    for _ in 0..100 {
        r.record_grant(0x01).unwrap();
    }
    assert_eq!(r.score(0x01).unwrap(), MAX_TRUST);
}

#[test]
fn test_trust_score_saturates_at_zero() {
    let mut r = TrustRegistry::new();
    r.register(0x01, false).unwrap();
    for _ in 0..100 {
        r.record_revocation(0x01).unwrap();
    }
    assert_eq!(r.score(0x01).unwrap(), 0);
}

#[test]
fn test_trust_is_trusted_mandatory() {
    let mut r = TrustRegistry::new();
    r.register(0x01, true).unwrap();
    assert!(r.is_trusted(0x01));
}

#[test]
fn test_trust_unknown_pd_not_trusted() {
    let r = TrustRegistry::new();
    assert!(!r.is_trusted(0x99));
}
