#![cfg(kani)]

// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Kani proofs — AXON-Bridge ABI contract invariants

use asl_axon_bridge::abi::{validate_token, AbiResult, ABI_TOKEN_V1, ABI_TOKEN_PREFIX};

/// PROOF: ABI_TOKEN_V1 always validates successfully.
#[cfg(kani)]
#[kani::proof]
fn proof_v1_token_always_valid() {
    assert_eq!(validate_token(ABI_TOKEN_V1), AbiResult::Valid);
}

/// PROOF: Zero token always returns MissingToken.
#[cfg(kani)]
#[kani::proof]
fn proof_zero_token_missing() {
    assert_eq!(validate_token(0), AbiResult::MissingToken);
}

/// PROOF: Any token with wrong prefix is InvalidPrefix.
/// Symbolic over all tokens without the ABI prefix.
#[cfg(kani)]
#[kani::proof]
fn proof_wrong_prefix_invalid() {
    let token: u64 = kani::any();
    let prefix = (token >> 48) as u16;
    kani::assume(token != 0);
    kani::assume(prefix != ABI_TOKEN_PREFIX);
    assert_eq!(validate_token(token), AbiResult::InvalidPrefix);
}

/// PROOF: ABI prefix constant is 0xAB10.
#[cfg(kani)]
#[kani::proof]
fn proof_abi_prefix_constant() {
    assert_eq!(ABI_TOKEN_PREFIX, 0xAB10u16);
}

// ── Non-kani tests ────────────────────────────────────────────────────

#[test]
fn test_v1_token_valid_proof() {
    assert_eq!(validate_token(ABI_TOKEN_V1), AbiResult::Valid);
}

#[test]
fn test_zero_token_missing_proof() {
    assert_eq!(validate_token(0), AbiResult::MissingToken);
}

#[test]
fn test_wrong_prefix_invalid_proof() {
    assert_eq!(validate_token(0xDEAD_0100_0000_0001), AbiResult::InvalidPrefix);
}
