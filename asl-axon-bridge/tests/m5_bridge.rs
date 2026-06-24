// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ASL-M5 test suite — AXON-Bridge PD
// Target: 50+ tests, 0 failures
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_axon_bridge::abi::{
    validate_token, AbiResult, ABI_TOKEN_V1, ABI_TOKEN_PREFIX,
    AXON_BRIDGE_ABI_VERSION, token_version,
};
use asl_axon_bridge::capability::{CapRegistry, CapError, AxonCapName};
use asl_axon_bridge::constant_time::{
    validate_ct_token, make_ct_token, CtResult, CtRegistry, CT_TOKEN_PREFIX,
};
use asl_axon_bridge::stub::{StubRegistry, StubError, StubId};
use asl_axon_bridge::bridge::{AxonBridge, LoadResult};
use asl_common::pd::PdId;

// ── ABI token tests (12) ──────────────────────────────────────────────

#[test]
fn test_abi_token_v1_valid() {
    assert_eq!(validate_token(ABI_TOKEN_V1), AbiResult::Valid);
}

#[test]
fn test_abi_token_zero_missing() {
    assert_eq!(validate_token(0), AbiResult::MissingToken);
}

#[test]
fn test_abi_token_wrong_prefix() {
    let bad = 0xDEAD_0100_0000_0001u64;
    assert_eq!(validate_token(bad), AbiResult::InvalidPrefix);
}

#[test]
fn test_abi_token_wrong_marker() {
    let bad = 0xAB10_0100_0000_0002u64;
    assert_eq!(validate_token(bad), AbiResult::InvalidMarker);
}

#[test]
fn test_abi_token_version_mismatch_major() {
    // Major version 2 — mismatch with v1
    let bad = 0xAB10_0200_0000_0001u64;
    assert!(matches!(validate_token(bad), AbiResult::VersionMismatch { .. }));
}

#[test]
fn test_abi_token_prefix_constant() {
    assert_eq!(ABI_TOKEN_PREFIX, 0xAB10u16);
}

#[test]
fn test_abi_token_v1_version_extraction() {
    let (major, minor, patch) = token_version(ABI_TOKEN_V1);
    assert_eq!(major, 1);
    assert_eq!(minor, 0);
    assert_eq!(patch, 0);
}

#[test]
fn test_abi_version_constant() {
    // v1.0.0 = 0x0001_0000
    assert_eq!(AXON_BRIDGE_ABI_VERSION, 0x0001_0000);
}

#[test]
fn test_abi_minor_version_difference_still_valid() {
    // Minor version difference (v1.1.0) should still be valid
    let v1_1 = 0xAB10_0101_0000_0001u64;
    assert_eq!(validate_token(v1_1), AbiResult::Valid);
}

#[test]
fn test_abi_patch_version_difference_still_valid() {
    // Patch difference (v1.0.5) should still be valid
    let v1_0_5 = 0xAB10_0100_0005_0001u64;
    assert_eq!(validate_token(v1_0_5), AbiResult::Valid);
}

#[test]
fn test_abi_required_profile_constant() {
    assert_eq!(asl_axon_bridge::abi::REQUIRED_PROFILE, "seL4-strict");
}

#[test]
fn test_abi_required_target_constant() {
    assert_eq!(asl_axon_bridge::abi::REQUIRED_TARGET, "aarch64-sel4");
}

// ── Capability registry tests (10) ───────────────────────────────────

#[test]
fn test_cap_register_succeeds() {
    let mut r = CapRegistry::new();
    assert!(r.register(0x01, AxonCapName::Endpoint, 0x100).is_ok());
}

#[test]
fn test_cap_zero_cptr_rejected() {
    let mut r = CapRegistry::new();
    assert_eq!(r.register(0x01, AxonCapName::Endpoint, 0), Err(CapError::ZeroCptr));
}

#[test]
fn test_cap_duplicate_rejected() {
    let mut r = CapRegistry::new();
    r.register(0x01, AxonCapName::Endpoint, 0x100).unwrap();
    assert_eq!(r.register(0x01, AxonCapName::Endpoint, 0x101), Err(CapError::AlreadyMapped));
}

#[test]
fn test_cap_translate_succeeds() {
    let mut r = CapRegistry::new();
    r.register(0x01, AxonCapName::Endpoint, 0x100).unwrap();
    assert_eq!(r.translate(0x01, AxonCapName::Endpoint), Ok(0x100));
}

#[test]
fn test_cap_translate_missing() {
    let r = CapRegistry::new();
    assert_eq!(r.translate(0x01, AxonCapName::Endpoint), Err(CapError::NoMapping));
}

#[test]
fn test_cap_revoke_pd() {
    let mut r = CapRegistry::new();
    r.register(0x01, AxonCapName::Endpoint, 0x100).unwrap();
    assert_eq!(r.revoke_pd(0x01), 1);
    assert_eq!(r.translate(0x01, AxonCapName::Endpoint), Err(CapError::NoMapping));
}

#[test]
fn test_cap_register_mandatory_pds() {
    let mut r = CapRegistry::new();
    assert!(r.register_mandatory_pds().is_ok());
    assert!(r.mapping_count() >= 5);
}

#[test]
fn test_cap_mandatory_pds_translatable() {
    let mut r = CapRegistry::new();
    r.register_mandatory_pds().unwrap();
    assert!(r.translate(PdId::Genesis as u8, AxonCapName::Endpoint).is_ok());
    assert!(r.translate(PdId::ArpiBroker as u8, AxonCapName::Endpoint).is_ok());
}

#[test]
fn test_cap_different_cap_types_coexist() {
    let mut r = CapRegistry::new();
    r.register(0x01, AxonCapName::Endpoint, 0x100).unwrap();
    r.register(0x01, AxonCapName::SharedFrame, 0x200).unwrap();
    assert_eq!(r.translate(0x01, AxonCapName::Endpoint), Ok(0x100));
    assert_eq!(r.translate(0x01, AxonCapName::SharedFrame), Ok(0x200));
}

#[test]
fn test_cap_revoke_other_pd_unaffected() {
    let mut r = CapRegistry::new();
    r.register(0x01, AxonCapName::Endpoint, 0x100).unwrap();
    r.register(0x02, AxonCapName::Endpoint, 0x101).unwrap();
    r.revoke_pd(0x01);
    assert!(r.translate(0x02, AxonCapName::Endpoint).is_ok());
}

// ── @constant_time tests (10) ─────────────────────────────────────────

#[test]
fn test_ct_token_valid() {
    let token = make_ct_token(0x1234, 1);
    assert_eq!(validate_ct_token(token), CtResult::Valid);
}

#[test]
fn test_ct_token_zero_missing() {
    assert_eq!(validate_ct_token(0), CtResult::MissingContract);
}

#[test]
fn test_ct_token_wrong_prefix() {
    let bad = 0xDEAD_FFFF_1234_0001u64;
    assert_eq!(validate_ct_token(bad), CtResult::InvalidPrefix);
}

#[test]
fn test_ct_token_zero_seq() {
    let bad = ((CT_TOKEN_PREFIX as u64) << 32) | 0x1234_0000u64;
    assert_eq!(validate_ct_token(bad), CtResult::ZeroSequence);
}

#[test]
fn test_ct_make_token_zero_seq_returns_zero() {
    assert_eq!(make_ct_token(0x1234, 0), 0u64);
}

#[test]
fn test_ct_prefix_constant() {
    assert_eq!(CT_TOKEN_PREFIX, 0xC0C0_FFFFu32);
}

#[test]
fn test_ct_registry_register() {
    let mut r = CtRegistry::new();
    let token = make_ct_token(0x1234, 1);
    assert!(r.register(0x1234, token).is_ok());
}

#[test]
fn test_ct_registry_verify_registered() {
    let mut r = CtRegistry::new();
    let token = make_ct_token(0x1234, 1);
    r.register(0x1234, token).unwrap();
    assert_eq!(r.verify(0x1234), CtResult::Valid);
}

#[test]
fn test_ct_registry_verify_unregistered() {
    let r = CtRegistry::new();
    assert_eq!(r.verify(0x9999), CtResult::MissingContract);
}

#[test]
fn test_ct_registry_count() {
    let mut r = CtRegistry::new();
    r.register(0x0001, make_ct_token(0x0001, 1)).unwrap();
    r.register(0x0002, make_ct_token(0x0002, 1)).unwrap();
    assert_eq!(r.registered_count(), 2);
}

// ── Stub registry tests (10) ──────────────────────────────────────────

#[test]
fn test_stub_register_succeeds() {
    let mut r = StubRegistry::new();
    assert!(r.register(StubId::Sel4SysSend).is_ok());
}

#[test]
fn test_stub_register_duplicate_fails() {
    let mut r = StubRegistry::new();
    r.register(StubId::Sel4SysSend).unwrap();
    assert_eq!(r.register(StubId::Sel4SysSend), Err(StubError::AlreadyRegistered));
}

#[test]
fn test_stub_register_ipc_stubs() {
    let mut r = StubRegistry::new();
    assert!(r.register_ipc_stubs().is_ok());
    assert_eq!(r.registered_count(), 5);
}

#[test]
fn test_stub_resolve_succeeds() {
    let mut r = StubRegistry::new();
    r.register(StubId::Sel4SysSend).unwrap();
    assert!(r.resolve(StubId::Sel4SysSend, 0xCAFE_0001).is_ok());
}

#[test]
fn test_stub_resolve_zero_addr_fails() {
    let mut r = StubRegistry::new();
    r.register(StubId::Sel4SysSend).unwrap();
    assert_eq!(r.resolve(StubId::Sel4SysSend, 0), Err(StubError::ZeroShimAddr));
}

#[test]
fn test_stub_not_registered_resolve_fails() {
    let mut r = StubRegistry::new();
    assert_eq!(r.resolve(StubId::Sel4SysSend, 0x1000), Err(StubError::NotFound));
}

#[test]
fn test_stub_ipc_not_ready_before_resolve() {
    let mut r = StubRegistry::new();
    r.register_ipc_stubs().unwrap();
    assert!(!r.ipc_ready());
}

#[test]
fn test_stub_ipc_ready_after_resolve() {
    let mut r = StubRegistry::new();
    r.register_ipc_stubs().unwrap();
    r.resolve(StubId::Sel4SysSend, 0x1001).unwrap();
    r.resolve(StubId::Sel4SysCall, 0x1002).unwrap();
    r.resolve(StubId::Sel4MrGet,   0x1003).unwrap();
    r.resolve(StubId::Sel4MrSet,   0x1004).unwrap();
    assert!(r.ipc_ready());
}

#[test]
fn test_stub_names_correct() {
    assert_eq!(StubId::Sel4SysSend.name(), "sel4_sys_send");
    assert_eq!(StubId::Sel4SysCall.name(), "sel4_sys_call");
    assert_eq!(StubId::Sel4MrGet.name(),   "sel4_mr_get");
    assert_eq!(StubId::Sel4MrSet.name(),   "sel4_mr_set");
}

#[test]
fn test_stub_resolved_count() {
    let mut r = StubRegistry::new();
    r.register_ipc_stubs().unwrap();
    r.resolve(StubId::Sel4SysSend, 0x1001).unwrap();
    assert_eq!(r.resolved_count(), 1);
}

// ── Bridge engine tests (10) ──────────────────────────────────────────

#[test]
fn test_bridge_commission_succeeds() {
    let mut b = AxonBridge::new();
    assert!(b.commission().is_ok());
}

#[test]
fn test_bridge_commission_registers_caps() {
    let mut b = AxonBridge::new();
    b.commission().unwrap();
    assert!(b.cap_count() >= 5);
}

#[test]
fn test_bridge_commission_registers_stubs() {
    let mut b = AxonBridge::new();
    b.commission().unwrap();
    assert_eq!(b.stub_count(), 5);
}

#[test]
fn test_bridge_load_valid_binary() {
    let mut b = AxonBridge::new();
    b.commission().unwrap();
    assert_eq!(b.load_binary(ABI_TOKEN_V1, 0x10, 0x8000_0000), LoadResult::Ready);
}

#[test]
fn test_bridge_load_invalid_abi_rejected() {
    let mut b = AxonBridge::new();
    b.commission().unwrap();
    assert_eq!(b.load_binary(0, 0x10, 0x8000_0000), LoadResult::AbiRejected);
}

#[test]
fn test_bridge_load_zero_entry_rejected() {
    let mut b = AxonBridge::new();
    b.commission().unwrap();
    assert_eq!(b.load_binary(ABI_TOKEN_V1, 0x10, 0), LoadResult::AbiRejected);
}

#[test]
fn test_bridge_loaded_count_increments() {
    let mut b = AxonBridge::new();
    b.commission().unwrap();
    b.load_binary(ABI_TOKEN_V1, 0x10, 0x8000_0000);
    assert_eq!(b.loaded_count(), 1);
}

#[test]
fn test_bridge_rejected_count_increments() {
    let mut b = AxonBridge::new();
    b.commission().unwrap();
    b.load_binary(0xDEAD, 0x10, 0x8000_0000);
    assert_eq!(b.rejected_count(), 1);
}

#[test]
fn test_bridge_pd_is_loaded_after_load() {
    let mut b = AxonBridge::new();
    b.commission().unwrap();
    b.load_binary(ABI_TOKEN_V1, 0x10, 0x8000_0000);
    assert!(b.pd_is_loaded(0x10));
}

#[test]
fn test_bridge_pd_not_loaded_before_load() {
    let b = AxonBridge::new();
    assert!(!b.pd_is_loaded(0x10));
}
