// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// m21_arpi_ipc.rs — Integration tests for ARPi Live IPC PD (M21)
// Target: 30+ tests, 0 failures
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_arpi_ipc::{
    ArpiBinder, ArpiHeader, ArpiError, BindOutcome,
    Ed25519PublicKey, Ed25519Signature, IdentityProof,
    MutualAuthSession, CapToken, ScopeRegistry, SchemaType,
    validate_schema, verify_identity, verify_mutual_auth,
    validate_scope, anomaly_gate, AnomalyResult,
    verify_sovereign_proof,
    ARPI_MAGIC, ARPI_HEADER_SIZE, ARPI_PD_ID,
    AXON_PROOF, ANOMALY_THRESHOLD, ED25519_KEY_SIZE,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn key(seed: u8) -> Ed25519PublicKey {
    let mut k = [0u8; ED25519_KEY_SIZE];
    k[0] = seed; k[1] = 0x01;
    Ed25519PublicKey(k)
}

fn proof(seed: u8) -> IdentityProof {
    let mut s = [0u8; 64];
    s[0] = seed; s[1] = 0x01;
    IdentityProof {
        pubkey: key(seed),
        sig:    Ed25519Signature(s),
        nonce:  [seed, 1, 2, 3, 4, 5, 6, 7],
    }
}

fn session(a: u8, b: u8) -> MutualAuthSession {
    MutualAuthSession { local: proof(a), remote: proof(b) }
}

fn token(seq: u32, schema: SchemaType) -> CapToken {
    CapToken::new(seq, ARPI_PD_ID, schema, 0x0001)
}

// ── Full bind lifecycle ───────────────────────────────────────────────────────

#[test]
fn test_sovereign_bind_lifecycle() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x01, 0x02);
    let tok = token(1, SchemaType::ArpiCtrl);

    let header = binder.bind(SchemaType::ArpiCtrl as u8, &sess, tok, 0).unwrap();

    // Header is valid
    assert!(header.validate().is_ok());
    assert_eq!(header.schema, SchemaType::ArpiCtrl);
    assert_eq!(&header.magic, &ARPI_MAGIC);

    // Bind logged as success
    assert_eq!(binder.log.count(), 1);
    assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Success);
}

#[test]
fn test_awp_message_bind() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x10, 0x20);
    let tok = token(1, SchemaType::AwpMsg);

    let header = binder.bind(SchemaType::AwpMsg as u8, &sess, tok, 10).unwrap();
    assert_eq!(header.schema, SchemaType::AwpMsg);
}

#[test]
fn test_critical_data_bind() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x03, 0x04);
    let tok = token(1, SchemaType::DataCrit);

    let header = binder.bind(SchemaType::DataCrit as u8, &sess, tok, 0).unwrap();
    assert_eq!(header.schema, SchemaType::DataCrit);
}

#[test]
fn test_multiple_sequential_binds() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x01, 0x02);

    for i in 1u32..=5 {
        let tok = token(i, SchemaType::ArpiCtrl);
        let h = binder.bind(SchemaType::ArpiCtrl as u8, &sess, tok, 0).unwrap();
        assert_eq!(h.seq, i as u64);
    }
    assert_eq!(binder.seq(), 5);
    assert_eq!(binder.log.count(), 5);
}

// ── All 5 failure modes ───────────────────────────────────────────────────────

#[test]
fn test_layer1_schema_failure_logged() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x01, 0x02);
    let result = binder.bind(0xFF, &sess, token(1, SchemaType::ArpiCtrl), 0);
    assert_eq!(result, Err(ArpiError::SchemaViolation));
    assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Failed(1));
}

#[test]
fn test_layer2_identity_failure_logged() {
    let mut binder = ArpiBinder::new();
    let mut sess = session(0x01, 0x02);
    sess.local.pubkey = Ed25519PublicKey([0u8; 32]); // invalid identity
    let result = binder.bind(0x01, &sess, token(1, SchemaType::ArpiCtrl), 0);
    assert_eq!(result, Err(ArpiError::IdentityRejected));
    assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Failed(2));
}

#[test]
fn test_layer3_self_bind_failure_logged() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x05, 0x05); // same key
    let result = binder.bind(0x01, &sess, token(1, SchemaType::ArpiCtrl), 0);
    assert_eq!(result, Err(ArpiError::MutualAuthFailed));
    assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Failed(3));
}

#[test]
fn test_layer4_scope_failure_logged() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x01, 0x02);
    // Token has DataCrit schema but we're requesting ArpiCtrl
    let tok = token(1, SchemaType::DataCrit);
    let result = binder.bind(SchemaType::ArpiCtrl as u8, &sess, tok, 0);
    assert_eq!(result, Err(ArpiError::ScopeViolation));
    assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Failed(4));
}

#[test]
fn test_layer5_anomaly_failure_logged() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x01, 0x02);
    let result = binder.bind(0x01, &sess, token(1, SchemaType::ArpiCtrl), 90);
    assert_eq!(result, Err(ArpiError::AnomalyEscalated));
    assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Failed(5));
}

// ── No silent failure — every attempt logged ──────────────────────────────────

#[test]
fn test_every_attempt_is_logged() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x01, 0x02);

    // Success
    binder.bind(0x01, &sess, token(1, SchemaType::ArpiCtrl), 0).unwrap();
    // Failure
    let _ = binder.bind(0xFF, &sess, token(2, SchemaType::ArpiCtrl), 0);
    // Anomaly
    let _ = binder.bind(0x01, &sess, token(3, SchemaType::ArpiCtrl), 100);

    assert_eq!(binder.log.count(), 3);
}

// ── Provenance header ─────────────────────────────────────────────────────────

#[test]
fn test_header_size_is_78() {
    assert_eq!(ARPI_HEADER_SIZE, 78);
}

#[test]
fn test_header_magic_bytes() {
    assert_eq!(ARPI_MAGIC, [0x41, 0x52, 0x50, 0x69]); // "ARPi"
}

#[test]
fn test_header_reserved_bytes_zero() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x01, 0x02);
    let h = binder.bind(0x01, &sess, token(1, SchemaType::ArpiCtrl), 0).unwrap();
    let b = h.to_bytes();
    assert!(b[56..78].iter().all(|&x| x == 0));
}

#[test]
fn test_header_carries_seq() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x01, 0x02);
    let h = binder.bind(0x01, &sess, token(1, SchemaType::ArpiCtrl), 0).unwrap();
    assert_eq!(h.seq, 1);
    let h2 = binder.bind(0x01, &sess, token(2, SchemaType::ArpiCtrl), 0).unwrap();
    assert_eq!(h2.seq, 2);
}

// ── Scope registry ────────────────────────────────────────────────────────────

#[test]
fn test_scope_multi_schema() {
    let mut reg = ScopeRegistry::new();
    reg.grant(CapToken::new(1, 0x01, SchemaType::ArpiCtrl, 0)).unwrap();
    reg.grant(CapToken::new(2, 0x01, SchemaType::AwpMsg,   0)).unwrap();
    reg.grant(CapToken::new(3, 0x01, SchemaType::DataPers, 0)).unwrap();

    assert!(reg.has_scope(SchemaType::ArpiCtrl));
    assert!(reg.has_scope(SchemaType::AwpMsg));
    assert!(reg.has_scope(SchemaType::DataPers));
    assert!(!reg.has_scope(SchemaType::DataCrit));
    assert_eq!(reg.count(), 3);
    assert_eq!(reg.last_seq(), 3);
}

#[test]
fn test_scope_monotonic_strictly_enforced() {
    let mut reg = ScopeRegistry::new();
    reg.grant(CapToken::new(10, 0x01, SchemaType::ArpiCtrl, 0)).unwrap();
    assert!(reg.grant(CapToken::new(10, 0x01, SchemaType::AwpMsg, 0)).is_err()); // equal = replay
    assert!(reg.grant(CapToken::new(5,  0x01, SchemaType::AwpMsg, 0)).is_err()); // less = old
    assert!(reg.grant(CapToken::new(11, 0x01, SchemaType::AwpMsg, 0)).is_ok());  // greater = ok
}

// ── Anomaly gate ──────────────────────────────────────────────────────────────

#[test]
fn test_anomaly_boundary_values() {
    assert_eq!(anomaly_gate(0),                    AnomalyResult::Clear);
    assert_eq!(anomaly_gate(49),                   AnomalyResult::Clear);
    assert_eq!(anomaly_gate(50),                   AnomalyResult::Flagged);
    assert_eq!(anomaly_gate(ANOMALY_THRESHOLD - 1),AnomalyResult::Flagged);
    assert_eq!(anomaly_gate(ANOMALY_THRESHOLD),    AnomalyResult::Escalated);
    assert_eq!(anomaly_gate(100),                  AnomalyResult::Escalated);
}

#[test]
fn test_flagged_bind_still_succeeds() {
    let mut binder = ArpiBinder::new();
    let sess = session(0x01, 0x02);
    // Score 60 = Flagged but below Escalated threshold
    let h = binder.bind(0x01, &sess, token(1, SchemaType::ArpiCtrl), 60);
    assert!(h.is_ok());
}

// ── Constants ─────────────────────────────────────────────────────────────────

#[test]
fn test_sovereign_proof_invariant() {
    assert_eq!(AXON_PROOF, 0x4153);
    assert!(verify_sovereign_proof(AXON_PROOF));
    assert!(!verify_sovereign_proof(0x0000));
}

#[test]
fn test_arpi_pd_id_is_mandatory() {
    // Mandatory PDs have ID < 0x10 (established M1)
    assert!(ARPI_PD_ID < 0x10);
    assert_eq!(ARPI_PD_ID, 0x01);
}

#[test]
fn test_anomaly_threshold_value() {
    assert_eq!(ANOMALY_THRESHOLD, 75);
}
