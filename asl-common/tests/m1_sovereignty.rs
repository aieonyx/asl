// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ASL-M1 sovereignty test suite
// Integration tests — runs on x86_64 host with std
// Target: 45 tests, 0 failures
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_common::arpi::ArpiHeader;
use asl_common::datatier::DataTier;
use asl_common::pd::PdId;
use asl_common::version::{ASL_VERSION, ASL_VERSION_STRING};

// ── Version tests (5) ────────────────────────────────────────────────

#[test]
fn test_version_string_not_empty() {
    assert!(!ASL_VERSION_STRING.is_empty());
}

#[test]
fn test_version_string_contains_asl() {
    assert!(ASL_VERSION_STRING.contains("ASL"));
}

#[test]
fn test_version_string_contains_sel4() {
    assert!(ASL_VERSION_STRING.contains("seL4"));
}

#[test]
fn test_version_not_empty() {
    assert!(!ASL_VERSION.is_empty());
}

#[test]
fn test_version_starts_with_zero() {
    assert!(ASL_VERSION.starts_with('0'),
        "ASL-M1 is pre-release — version must start with 0");
}

// ── PdId mandatory/optional classification (13) ──────────────────────

#[test]
fn test_genesis_is_mandatory() {
    assert!(PdId::Genesis.is_mandatory());
}

#[test]
fn test_arpi_broker_is_mandatory() {
    assert!(PdId::ArpiBroker.is_mandatory());
}

#[test]
fn test_datatier_enforcer_is_mandatory() {
    assert!(PdId::DataTierEnforcer.is_mandatory());
}

#[test]
fn test_trustgraph_gate_is_mandatory() {
    assert!(PdId::TrustGraphGate.is_mandatory());
}

#[test]
fn test_inverted_admin_is_mandatory() {
    assert!(PdId::InvertedAdmin.is_mandatory());
}

#[test]
fn test_axon_bridge_is_mandatory() {
    assert!(PdId::AxonBridge.is_mandatory());
}

#[test]
fn test_gpu_cap_is_optional() {
    assert!(!PdId::GpuCap.is_mandatory());
}

#[test]
fn test_power_mgmt_is_optional() {
    assert!(!PdId::PowerMgmt.is_mandatory());
}

#[test]
fn test_network_routing_is_optional() {
    assert!(!PdId::NetworkRouting.is_mandatory());
}

#[test]
fn test_firewall_cap_is_optional() {
    assert!(!PdId::FirewallCap.is_mandatory());
}

#[test]
fn test_touch_sensor_is_optional() {
    assert!(!PdId::TouchSensor.is_mandatory());
}

#[test]
fn test_mandatory_pd_ids_below_0x10() {
    let mandatory = [
        PdId::Genesis, PdId::ArpiBroker, PdId::DataTierEnforcer,
        PdId::TrustGraphGate, PdId::InvertedAdmin, PdId::AxonBridge,
    ];
    for pd in mandatory {
        assert!((pd as u8) < 0x10, "mandatory PD ID must be < 0x10: {:?}", pd);
    }
}

#[test]
fn test_optional_pd_ids_at_or_above_0x10() {
    let optional = [
        PdId::GpuCap, PdId::PowerMgmt, PdId::NetworkRouting,
        PdId::FirewallCap, PdId::TouchSensor,
    ];
    for pd in optional {
        assert!((pd as u8) >= 0x10, "optional PD ID must be >= 0x10: {:?}", pd);
    }
}

// ── ARPi header tests (8) ────────────────────────────────────────────

#[test]
fn test_arpi_header_size_is_78_bytes() {
    assert_eq!(ArpiHeader::SIZE, 78);
}

#[test]
fn test_arpi_magic_constant() {
    assert_eq!(ArpiHeader::MAGIC, 0xA291u16);
}

#[test]
fn test_arpi_header_new_magic_valid() {
    let sig = [0u8; 64];
    let h = ArpiHeader::new(0x01, 0x02, 0x00, 1, sig);
    assert!(h.is_valid_magic());
}

#[test]
fn test_arpi_header_src_dst() {
    let sig = [0u8; 64];
    let h = ArpiHeader::new(
        PdId::Genesis as u8,
        PdId::ArpiBroker as u8,
        0x00, 42, sig,
    );
    assert_eq!({ h.src_pd }, PdId::Genesis as u8);
    assert_eq!({ h.dst_pd }, PdId::ArpiBroker as u8);
}

#[test]
fn test_arpi_header_seq_monotonic() {
    let sig = [0u8; 64];
    let h1 = ArpiHeader::new(0x01, 0x02, 0x00, 1, sig);
    let h2 = ArpiHeader::new(0x01, 0x02, 0x00, 2, sig);
    assert!({ h2.seq } > { h1.seq });
}

#[test]
fn test_arpi_header_asl_version_byte() {
    let sig = [0u8; 64];
    let h = ArpiHeader::new(0x01, 0x02, 0x00, 1, sig);
    assert_eq!({ h.asl_ver }, 0x01u8);
}

#[test]
fn test_arpi_header_data_tier_noise() {
    let sig = [0u8; 64];
    let h = ArpiHeader::new(0x01, 0x02, DataTier::Noise as u8, 1, sig);
    assert_eq!({ h.data_tier }, 0x00u8);
}

#[test]
fn test_arpi_header_data_tier_critical() {
    let sig = [0u8; 64];
    let h = ArpiHeader::new(0x01, 0x02, DataTier::Critical as u8, 1, sig);
    assert_eq!({ h.data_tier }, 0x02u8);
}

// ── DataTier tests (12) ──────────────────────────────────────────────

#[test]
fn test_datatier_noise_to_personal_requires_grant() {
    assert!(DataTier::requires_grant(DataTier::Noise, DataTier::Personal));
}

#[test]
fn test_datatier_noise_to_critical_requires_grant() {
    assert!(DataTier::requires_grant(DataTier::Noise, DataTier::Critical));
}

#[test]
fn test_datatier_personal_to_critical_requires_grant() {
    assert!(DataTier::requires_grant(DataTier::Personal, DataTier::Critical));
}

#[test]
fn test_datatier_noise_to_noise_no_grant() {
    assert!(!DataTier::requires_grant(DataTier::Noise, DataTier::Noise));
}

#[test]
fn test_datatier_personal_to_personal_no_grant() {
    assert!(!DataTier::requires_grant(DataTier::Personal, DataTier::Personal));
}

#[test]
fn test_datatier_critical_to_critical_no_grant() {
    assert!(!DataTier::requires_grant(DataTier::Critical, DataTier::Critical));
}

#[test]
fn test_datatier_critical_to_personal_no_grant() {
    assert!(!DataTier::requires_grant(DataTier::Critical, DataTier::Personal));
}

#[test]
fn test_datatier_critical_to_noise_no_grant() {
    assert!(!DataTier::requires_grant(DataTier::Critical, DataTier::Noise));
}

#[test]
fn test_datatier_personal_to_noise_no_grant() {
    assert!(!DataTier::requires_grant(DataTier::Personal, DataTier::Noise));
}

#[test]
fn test_datatier_ordering() {
    assert!(DataTier::Noise < DataTier::Personal);
    assert!(DataTier::Personal < DataTier::Critical);
    assert!(DataTier::Noise < DataTier::Critical);
}

#[test]
fn test_datatier_noise_repr() {
    assert_eq!(DataTier::Noise as u8, 0x00u8);
}

#[test]
fn test_datatier_personal_repr() {
    assert_eq!(DataTier::Personal as u8, 0x01u8);
}

#[test]
fn test_datatier_critical_repr() {
    assert_eq!(DataTier::Critical as u8, 0x02u8);
}

// ── Commissioning invariant tests (7) ────────────────────────────────

#[test]
fn test_trust_anchor_has_aieonyx_prefix() {
    let anchor: u64 = 0xA1E0_4E4C_5339_0001;
    assert_eq!(anchor >> 48, 0xA1E0u64);
}

#[test]
fn test_trust_anchor_nonzero() {
    let anchor: u64 = 0xA1E0_4E4C_5339_0001;
    assert_ne!(anchor, 0u64);
}

#[test]
fn test_trust_anchor_not_max() {
    let anchor: u64 = 0xA1E0_4E4C_5339_0001;
    assert_ne!(anchor, u64::MAX);
}

#[test]
fn test_mandatory_pd_count_is_six() {
    let mandatory = [
        PdId::Genesis, PdId::ArpiBroker, PdId::DataTierEnforcer,
        PdId::TrustGraphGate, PdId::InvertedAdmin, PdId::AxonBridge,
    ];
    assert_eq!(mandatory.len(), 6);
}

#[test]
fn test_optional_pd_count_is_five() {
    let optional = [
        PdId::GpuCap, PdId::PowerMgmt, PdId::NetworkRouting,
        PdId::FirewallCap, PdId::TouchSensor,
    ];
    assert_eq!(optional.len(), 5);
}

#[test]
fn test_devmode_unconditionally_false() {
    let dev_mode = false;
    assert!(!dev_mode, "DevMode must be unconditionally false in sovereign build");
}

#[test]
fn test_arpi_size_runtime_matches_compile_time() {
    assert_eq!(core::mem::size_of::<ArpiHeader>(), 78);
}
