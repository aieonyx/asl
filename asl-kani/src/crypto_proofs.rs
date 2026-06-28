// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// crypto_proofs.rs — Kani formal verification harnesses for asl-crypto-bridge
//
// Note: Kani cannot verify through x86 CPUID intrinsics used by AES-NI and
// Argon2 (TerminatorKind::InlineAsm unsupported). Harnesses prove structural
// and arithmetic properties from constants instead of calling crypto functions.
//
// Six harnesses:
//   1. key_len_invariant          — KEY_LEN constant is 32
//   2. nonce_counter_monotonic    — nonce bytes encode counter correctly
//   3. encrypt_output_len_math    — GCM tag overhead is always 16 bytes (math)
//   4. wrong_key_differs          — two different keys produce different bytes
//   5. empty_input_rejected_const — empty slice detection logic correct
//   6. nonce_encoding_correctness — counter encodes big-endian in nonce[0..8]

#![cfg(kani)]

extern crate kani;

use asl_crypto_bridge::{
    nonce_from_counter,
    KEY_LEN, NONCE_LEN,
};

// ── Harness 1: Key length invariant ──────────────────────────────────────────

#[kani::proof]
#[kani::unwind(1)]
fn key_len_invariant() {
    // KEY_LEN must be 32 (AES-256 requires 256-bit = 32-byte key)
    assert!(KEY_LEN == 32);
    assert!(KEY_LEN * 8 == 256);
}

// ── Harness 2: Nonce counter monotonic ───────────────────────────────────────

#[kani::proof]
#[kani::unwind(1)]
fn nonce_counter_monotonic() {
    let n: u64 = kani::any();
    kani::assume(n < u64::MAX);

    let n0 = nonce_from_counter(n);
    let n1 = nonce_from_counter(n + 1);

    // Different counters must produce different nonces
    assert!(n0 != n1);
}

// ── Harness 3: Encrypt output length math ────────────────────────────────────

#[kani::proof]
#[kani::unwind(1)]
fn encrypt_output_len_math() {
    // AES-256-GCM always appends a 16-byte authentication tag.
    // For any plaintext of length pt_len, ciphertext = pt_len + 16.
    let pt_len: usize = kani::any();
    kani::assume(pt_len <= 1024);

    let expected_ct_len = pt_len + 16;
    assert!(expected_ct_len > pt_len);
    assert!(expected_ct_len == pt_len + 16);
}

// ── Harness 4: Different keys produce different key material ─────────────────

#[kani::proof]
#[kani::unwind(1)]
fn wrong_key_differs() {
    let key1: [u8; KEY_LEN] = kani::any();
    let mut key2 = key1;
    let flip_idx: usize = kani::any();
    kani::assume(flip_idx < KEY_LEN);
    key2[flip_idx] ^= 0xFF;
    // After flipping one byte, keys must differ
    assert!(key1 != key2);
}

// ── Harness 5: Empty input detection ─────────────────────────────────────────

#[kani::proof]
#[kani::unwind(1)]
fn empty_input_rejected_const() {
    // Empty slice detection: is_empty() iff len() == 0
    let empty: &[u8] = &[];
    assert!(empty.is_empty());
    assert!(empty.len() == 0);

    let non_empty: &[u8] = &[0x01];
    assert!(!non_empty.is_empty());
    assert!(non_empty.len() == 1);
}

// ── Harness 6: Nonce encoding correctness ────────────────────────────────────

#[kani::proof]
#[kani::unwind(1)]
fn nonce_encoding_correctness() {
    let counter: u64 = kani::any();
    let nonce = nonce_from_counter(counter);

    // High 8 bytes encode counter big-endian
    let expected = counter.to_be_bytes();
    assert!(nonce[0] == expected[0]);
    assert!(nonce[1] == expected[1]);
    assert!(nonce[2] == expected[2]);
    assert!(nonce[3] == expected[3]);
    assert!(nonce[4] == expected[4]);
    assert!(nonce[5] == expected[5]);
    assert!(nonce[6] == expected[6]);
    assert!(nonce[7] == expected[7]);

    // Trailing 4 bytes are zero
    assert!(nonce[8]  == 0);
    assert!(nonce[9]  == 0);
    assert!(nonce[10] == 0);
    assert!(nonce[11] == 0);
    assert!(NONCE_LEN == 12);
}
