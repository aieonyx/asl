#![cfg(kani)]

// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Kani proofs — ARPi provenance header invariants

use asl_common::arpi::ArpiHeader;
use asl_common::pd::PdId;

/// PROOF: ArpiHeader is always exactly 78 bytes.
/// This is a compile-time assertion, but Kani provides
/// a runtime-verified proof over all possible struct layouts.
#[cfg(kani)]
#[kani::proof]
fn proof_arpi_header_size() {
    assert_eq!(core::mem::size_of::<ArpiHeader>(), 78);
    assert_eq!(ArpiHeader::SIZE, 78);
}

/// PROOF: ArpiHeader magic is always 0xA291.
#[cfg(kani)]
#[kani::proof]
fn proof_arpi_magic_constant() {
    assert_eq!(ArpiHeader::MAGIC, 0xA291u16);
}

/// PROOF: A header with correct magic always validates.
/// Symbolic over all possible src/dst/tier/seq combinations.
#[cfg(kani)]
#[kani::proof]
fn proof_arpi_valid_magic_validates() {
    let src: u8  = kani::any();
    let dst: u8  = kani::any();
    let tier: u8 = kani::any();
    let seq: u64 = kani::any();
    let sig = [0u8; 64];
    let h = ArpiHeader::new(src, dst, tier, seq, sig);
    assert!(h.is_valid_magic());
}

/// PROOF: Mandatory PD IDs are always < 0x10.
#[cfg(kani)]
#[kani::proof]
fn proof_mandatory_pd_ids_bounded() {
    let mandatory = [
        PdId::Genesis as u8,
        PdId::ArpiBroker as u8,
        PdId::DataTierEnforcer as u8,
        PdId::TrustGraphGate as u8,
        PdId::InvertedAdmin as u8,
        PdId::AxonBridge as u8,
    ];
    for &id in mandatory.iter() {
        assert!(id < 0x10u8);
    }
}

/// PROOF: Optional PD IDs are always >= 0x10.
#[cfg(kani)]
#[kani::proof]
fn proof_optional_pd_ids_bounded() {
    let optional = [
        PdId::GpuCap as u8,
        PdId::PowerMgmt as u8,
        PdId::NetworkRouting as u8,
        PdId::FirewallCap as u8,
        PdId::TouchSensor as u8,
    ];
    for &id in optional.iter() {
        assert!(id >= 0x10u8);
    }
}

// ── Non-kani tests (run with cargo test) ────────────────────────────

#[test]
fn test_arpi_size_proof_holds() {
    assert_eq!(core::mem::size_of::<ArpiHeader>(), 78);
}

#[test]
fn test_arpi_magic_proof_holds() {
    assert_eq!(ArpiHeader::MAGIC, 0xA291u16);
}

#[test]
fn test_arpi_valid_header_proof() {
    let h = ArpiHeader::new(0x01, 0x02, 0x00, 42, [0u8; 64]);
    assert!(h.is_valid_magic());
}

#[test]
fn test_mandatory_pd_ids_proof() {
    assert!((PdId::Genesis as u8) < 0x10);
    assert!((PdId::ArpiBroker as u8) < 0x10);
    assert!((PdId::AxonBridge as u8) < 0x10);
}
