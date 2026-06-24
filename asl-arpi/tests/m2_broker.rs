// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ASL-M2 test suite — ARPi-Broker PD
// Target: 50+ tests, 0 failures
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_arpi::broker::{ArpiBroker, BrokerResult};
use asl_arpi::route::{RouteTable, RouteError, FASTPATH_PAYLOAD_BYTES, FASTPATH_TOTAL_BYTES};
use asl_arpi::sequence::{SequenceTable, SeqError};
use asl_arpi::tier_gate::{self, TierGateResult, GENESIS_TIER};
use asl_common::arpi::ArpiHeader;
use asl_common::datatier::DataTier;
use asl_common::pd::PdId;

fn make_header(src: PdId, dst: PdId, tier: DataTier, seq: u64) -> ArpiHeader {
    ArpiHeader::new(src as u8, dst as u8, tier as u8, seq, [0u8; 64])
}

fn commissioned_broker() -> ArpiBroker {
    let mut b = ArpiBroker::new();
    b.commission().expect("commissioning must succeed");
    b
}

// ── Broker commissioning tests (6) ───────────────────────────────────

#[test]
fn test_commission_succeeds() {
    let mut b = ArpiBroker::new();
    assert!(b.commission().is_ok());
}

#[test]
fn test_commission_registers_six_pds() {
    let b = commissioned_broker();
    assert_eq!(b.sequence_count(), 6);
}

#[test]
fn test_commission_registers_routes() {
    let b = commissioned_broker();
    assert!(b.route_count() > 0);
}

#[test]
fn test_commission_idempotent_fails_second_call() {
    let mut b = ArpiBroker::new();
    assert!(b.commission().is_ok());
    // Second commission must fail — PDs already registered
    assert!(b.commission().is_err());
}

#[test]
fn test_initial_dispatched_count_zero() {
    let b = commissioned_broker();
    assert_eq!(b.dispatched_count(), 0);
}

#[test]
fn test_initial_rejected_count_zero() {
    let b = commissioned_broker();
    assert_eq!(b.rejected_count(), 0);
}

// ── Broker dispatch — fastpath (5) ───────────────────────────────────

#[test]
fn test_dispatch_genesis_to_broker_fastpath() {
    let mut b = commissioned_broker();
    let h = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 1);
    assert_eq!(b.dispatch(&h, 0, None), BrokerResult::FastPath);
}

#[test]
fn test_dispatch_fastpath_small_payload() {
    let mut b = commissioned_broker();
    let h = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 1);
    assert_eq!(b.dispatch(&h, FASTPATH_PAYLOAD_BYTES, None), BrokerResult::FastPath);
}

#[test]
fn test_dispatch_slowpath_large_payload() {
    let mut b = commissioned_broker();
    let h = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 1);
    assert_eq!(b.dispatch(&h, FASTPATH_PAYLOAD_BYTES + 1, None), BrokerResult::SlowPath);
}

#[test]
fn test_dispatch_increments_dispatched_count() {
    let mut b = commissioned_broker();
    let h = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 1);
    b.dispatch(&h, 0, None);
    assert_eq!(b.dispatched_count(), 1);
}

#[test]
fn test_dispatch_multiple_increments_count() {
    let mut b = commissioned_broker();
    for seq in 1..=5 {
        let h = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, seq);
        b.dispatch(&h, 0, None);
    }
    assert_eq!(b.dispatched_count(), 5);
}

// ── Broker dispatch — rejection (8) ──────────────────────────────────

#[test]
fn test_dispatch_invalid_magic_rejected() {
    let mut b = commissioned_broker();
    let mut h = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 1);
    // Corrupt the magic
    let raw = &mut h as *mut ArpiHeader as *mut u8;
    unsafe { *raw = 0xFF; *(raw.add(1)) = 0xFF; }
    assert_eq!(b.dispatch(&h, 0, None), BrokerResult::InvalidMagic);
}

#[test]
fn test_dispatch_no_route_rejected() {
    let mut b = commissioned_broker();
    // DataTier-Enforcer → AxonBridge has no registered route
    let h = make_header(PdId::DataTierEnforcer, PdId::AxonBridge, DataTier::Noise, 1);
    assert_eq!(b.dispatch(&h, 0, None), BrokerResult::NoRoute);
}

#[test]
fn test_dispatch_replay_rejected() {
    let mut b = commissioned_broker();
    let h1 = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 1);
    b.dispatch(&h1, 0, None);
    // Same seq again — replay
    let h2 = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 1);
    assert_eq!(b.dispatch(&h2, 0, None), BrokerResult::ReplayDetected);
}

#[test]
fn test_dispatch_seq_zero_rejected() {
    let mut b = commissioned_broker();
    // seq=0 is <= last_seen=0, so replay
    let h = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 0);
    assert_eq!(b.dispatch(&h, 0, None), BrokerResult::ReplayDetected);
}

#[test]
fn test_dispatch_rejection_increments_rejected_count() {
    let mut b = commissioned_broker();
    let h = make_header(PdId::DataTierEnforcer, PdId::AxonBridge, DataTier::Noise, 1);
    b.dispatch(&h, 0, None);
    assert_eq!(b.rejected_count(), 1);
}

#[test]
fn test_dispatch_unknown_source_rejected() {
    let mut b = commissioned_broker();
    // GpuCap is not registered in sequence table
    let h = make_header(PdId::GpuCap, PdId::ArpiBroker, DataTier::Noise, 1);
    assert_eq!(b.dispatch(&h, 0, None), BrokerResult::NoRoute);
}

#[test]
fn test_dispatch_seq_must_be_strictly_increasing() {
    let mut b = commissioned_broker();
    let h1 = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 5);
    b.dispatch(&h1, 0, None);
    // seq=4 is less than last_seen=5
    let h2 = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 4);
    assert_eq!(b.dispatch(&h2, 0, None), BrokerResult::ReplayDetected);
}

#[test]
fn test_dispatch_seq_gap_allowed() {
    let mut b = commissioned_broker();
    let h1 = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 1);
    b.dispatch(&h1, 0, None);
    // seq=100 is fine — gaps are allowed, only replays rejected
    let h2 = make_header(PdId::Genesis, PdId::ArpiBroker, DataTier::Noise, 100);
    assert_eq!(b.dispatch(&h2, 0, None), BrokerResult::FastPath);
}

// ── Sequence table tests (10) ─────────────────────────────────────────

#[test]
fn test_seq_register_pd() {
    let mut t = SequenceTable::new();
    assert!(t.register(0x01).is_ok());
}

#[test]
fn test_seq_register_duplicate_fails() {
    let mut t = SequenceTable::new();
    t.register(0x01).unwrap();
    assert_eq!(t.register(0x01), Err(SeqError::AlreadyRegistered));
}

#[test]
fn test_seq_validate_first_message() {
    let mut t = SequenceTable::new();
    t.register(0x01).unwrap();
    assert!(t.validate_and_advance(0x01, 1).is_ok());
}

#[test]
fn test_seq_validate_replay_rejected() {
    let mut t = SequenceTable::new();
    t.register(0x01).unwrap();
    t.validate_and_advance(0x01, 1).unwrap();
    assert!(matches!(
        t.validate_and_advance(0x01, 1),
        Err(SeqError::ReplayDetected { .. })
    ));
}

#[test]
fn test_seq_validate_unknown_pd() {
    let mut t = SequenceTable::new();
    assert_eq!(t.validate_and_advance(0x99, 1), Err(SeqError::UnknownPd));
}

#[test]
fn test_seq_last_seq_after_advance() {
    let mut t = SequenceTable::new();
    t.register(0x01).unwrap();
    t.validate_and_advance(0x01, 42).unwrap();
    assert_eq!(t.last_seq(0x01), Some(42));
}

#[test]
fn test_seq_initial_last_seq_zero() {
    let mut t = SequenceTable::new();
    t.register(0x01).unwrap();
    assert_eq!(t.last_seq(0x01), Some(0));
}

#[test]
fn test_seq_multiple_pds_independent() {
    let mut t = SequenceTable::new();
    t.register(0x01).unwrap();
    t.register(0x02).unwrap();
    t.validate_and_advance(0x01, 10).unwrap();
    t.validate_and_advance(0x02, 1).unwrap();
    assert_eq!(t.last_seq(0x01), Some(10));
    assert_eq!(t.last_seq(0x02), Some(1));
}

#[test]
fn test_seq_count_after_registrations() {
    let mut t = SequenceTable::new();
    t.register(0x01).unwrap();
    t.register(0x02).unwrap();
    t.register(0x03).unwrap();
    assert_eq!(t.registered_count(), 3);
}

#[test]
fn test_seq_zero_seq_is_replay() {
    let mut t = SequenceTable::new();
    t.register(0x01).unwrap();
    // last_seq starts at 0; seq=0 is not strictly greater
    assert!(matches!(
        t.validate_and_advance(0x01, 0),
        Err(SeqError::ReplayDetected { .. })
    ));
}

// ── Route table tests (10) ────────────────────────────────────────────

#[test]
fn test_route_register_succeeds() {
    let mut t = RouteTable::new();
    assert!(t.register(0x01, 0x02, true).is_ok());
}

#[test]
fn test_route_register_duplicate_fails() {
    let mut t = RouteTable::new();
    t.register(0x01, 0x02, true).unwrap();
    assert_eq!(t.register(0x01, 0x02, false), Err(RouteError::AlreadyExists));
}

#[test]
fn test_route_self_route_rejected() {
    let mut t = RouteTable::new();
    assert_eq!(t.register(0x01, 0x01, true), Err(RouteError::SelfRoute));
}

#[test]
fn test_route_lookup_existing() {
    let mut t = RouteTable::new();
    t.register(0x01, 0x02, true).unwrap();
    assert!(t.lookup(0x01, 0x02).is_ok());
}

#[test]
fn test_route_lookup_missing() {
    let t = RouteTable::new();
    assert_eq!(t.lookup(0x01, 0x02), Err(RouteError::NoRoute));
}

#[test]
fn test_route_fastpath_flag() {
    let mut t = RouteTable::new();
    t.register(0x01, 0x02, true).unwrap();
    assert!(t.is_fastpath(0x01, 0x02));
}

#[test]
fn test_route_slowpath_flag() {
    let mut t = RouteTable::new();
    t.register(0x01, 0x02, false).unwrap();
    assert!(!t.is_fastpath(0x01, 0x02));
}

#[test]
fn test_route_deactivate() {
    let mut t = RouteTable::new();
    t.register(0x01, 0x02, true).unwrap();
    t.deactivate(0x01, 0x02).unwrap();
    assert_eq!(t.lookup(0x01, 0x02), Err(RouteError::NoRoute));
}

#[test]
fn test_route_count() {
    let mut t = RouteTable::new();
    t.register(0x01, 0x02, true).unwrap();
    t.register(0x02, 0x03, true).unwrap();
    assert_eq!(t.route_count(), 2);
}

#[test]
fn test_fastpath_payload_budget() {
    // ARPi header (78) + payload must fit in seL4 fastpath (~120 bytes)
    assert!(FASTPATH_TOTAL_BYTES <= 122);
    assert!(FASTPATH_PAYLOAD_BYTES > 0);
}

// ── Tier gate tests (8) ───────────────────────────────────────────────

#[test]
fn test_tier_gate_same_tier_allowed() {
    assert_eq!(
        tier_gate::check(DataTier::Noise, DataTier::Noise, None),
        TierGateResult::Allow
    );
}

#[test]
fn test_tier_gate_downgrade_allowed() {
    assert_eq!(
        tier_gate::check(DataTier::Critical, DataTier::Noise, None),
        TierGateResult::Allow
    );
}

#[test]
fn test_tier_gate_upgrade_requires_grant() {
    assert_eq!(
        tier_gate::check(DataTier::Noise, DataTier::Personal, None),
        TierGateResult::RequiresGrant
    );
}

#[test]
fn test_tier_gate_critical_upgrade_requires_grant() {
    assert_eq!(
        tier_gate::check(DataTier::Noise, DataTier::Critical, None),
        TierGateResult::RequiresGrant
    );
}

#[test]
fn test_tier_gate_empty_token_still_requires_grant() {
    assert_eq!(
        tier_gate::check(DataTier::Noise, DataTier::Critical, Some(&[])),
        TierGateResult::RequiresGrant
    );
}

#[test]
fn test_tier_gate_nonempty_token_accepted() {
    assert_eq!(
        tier_gate::check(DataTier::Noise, DataTier::Critical, Some(&[0x01])),
        TierGateResult::GrantAccepted
    );
}

#[test]
fn test_genesis_tier_is_noise() {
    assert_eq!(GENESIS_TIER, DataTier::Noise);
}

#[test]
fn test_datatier_from_u8_unknown_defaults_critical() {
    assert_eq!(DataTier::from_u8(0xFF), DataTier::Critical);
}
