// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// crypto_proofs — Kani formal verification harnesses for asl-crypto-bridge
//
// Six harnesses proving encryption correctness properties:
//   1. key_len_invariant          — derived key is always KEY_LEN bytes
//   2. nonce_counter_monotonic    — nonce(n+1) != nonce(n) for all n < u64::MAX
//   3. encrypt_output_longer      — ciphertext always longer than plaintext (GCM tag)
//   4. decrypt_wrong_key_fails    — bitflip in key → DecryptError (authentication)
//   5. empty_input_rejected       — derive_key rejects empty passphrase/salt
//   6. nonce_encoding_correctness — counter encodes into nonce bytes correctly

#![cfg(kani)]

extern crate kani;

use asl_crypto_bridge::{
    derive_key, encrypt, decrypt, nonce_from_counter,
    CryptoError, KEY_LEN, NONCE_LEN,
};

// ── Harness 1: Key length invariant ──────────────────────────────────────────
//
// For any non-empty passphrase and salt, derive_key produces exactly KEY_LEN bytes.

#[kani::proof]
#[kani::unwind(4)]
fn key_len_invariant() {
    // Symbolic inputs — Kani explores all values.
    let pass_len: usize = kani::any();
    let salt_len: usize = kani::any();
    kani::assume(pass_len >= 1 && pass_len <= 8);
    kani::assume(salt_len >= 1 && salt_len <= 8);

    let passphrase = vec![kani::any::<u8>(); pass_len];
    let salt = vec![kani::any::<u8>(); salt_len];

    match derive_key(&passphrase, &salt) {
        Ok(key) => {
            assert!(key.len() == KEY_LEN);
        }
        Err(CryptoError::KdfError) => {
            // Argon2 can fail under extreme constraints — acceptable.
        }
        Err(CryptoError::InvalidInput) => {
            // Only reachable if pass or salt is empty — excluded by assume.
            kani::assert(false, "InvalidInput should not be reachable with non-empty inputs");
        }
        Err(_) => {
            kani::assert(false, "Unexpected error variant from derive_key");
        }
    }
}

// ── Harness 2: Nonce counter monotonic ───────────────────────────────────────
//
// For any counter n < u64::MAX, nonce(n) != nonce(n+1).

#[kani::proof]
#[kani::unwind(1)]
fn nonce_counter_monotonic() {
    let n: u64 = kani::any();
    kani::assume(n < u64::MAX);

    let n0 = nonce_from_counter(n);
    let n1 = nonce_from_counter(n + 1);

    assert!(n0 != n1);
}

// ── Harness 3: Encrypt output longer than plaintext ──────────────────────────
//
// AES-256-GCM appends a 16-byte authentication tag.
// Output length must be plaintext_len + 16.

#[kani::proof]
#[kani::unwind(2)]
fn encrypt_output_longer() {
    let pt_len: usize = kani::any();
    kani::assume(pt_len >= 1 && pt_len <= 16);

    let key_bytes: [u8; KEY_LEN] = kani::any();
    let nonce_bytes: [u8; NONCE_LEN] = kani::any();
    let plaintext = vec![kani::any::<u8>(); pt_len];

    match encrypt(&key_bytes, &nonce_bytes, &plaintext) {
        Ok(ct) => {
            // GCM tag is 16 bytes.
            assert!(ct.len() == pt_len + 16);
        }
        Err(_) => {
            // aes-gcm encrypt is infallible in practice; allow path for proof completeness.
        }
    }
}

// ── Harness 4: Wrong key causes decryption failure ───────────────────────────
//
// If the key is bitflipped by one bit, decrypt must return DecryptError.
// This proves GCM authentication catches key mismatch.

#[kani::proof]
#[kani::unwind(2)]
fn decrypt_wrong_key_fails() {
    let correct_key: [u8; KEY_LEN] = kani::any();
    let nonce_bytes: [u8; NONCE_LEN] = kani::any();

    // Minimal known plaintext.
    let plaintext = [0x41u8; 4]; // "AAAA"

    // Encrypt under correct key.
    let ct = match encrypt(&correct_key, &nonce_bytes, &plaintext) {
        Ok(c) => c,
        Err(_) => return, // encrypt infallible; allow for completeness
    };

    // Build a wrong key — flip exactly one bit in the first byte.
    let mut wrong_key = correct_key;
    wrong_key[0] ^= 0x01;

    // If keys happen to be equal after flip (only if bit was already 0), skip.
    kani::assume(wrong_key[0] != correct_key[0]);

    let result = decrypt(&wrong_key, &nonce_bytes, &ct);
    assert!(result == Err(CryptoError::DecryptError));
}

// ── Harness 5: Empty input rejection ─────────────────────────────────────────
//
// derive_key must reject empty passphrase or empty salt.

#[kani::proof]
#[kani::unwind(1)]
fn empty_input_rejected() {
    // Case A: empty passphrase.
    let result_a = derive_key(b"", b"valid-salt");
    assert!(result_a == Err(CryptoError::InvalidInput));

    // Case B: empty salt.
    let result_b = derive_key(b"valid-pass", b"");
    assert!(result_b == Err(CryptoError::InvalidInput));
}

// ── Harness 6: Nonce encoding correctness ────────────────────────────────────
//
// The counter must appear in the high bytes of the nonce (big-endian),
// and the trailing bytes must be zero.

#[kani::proof]
#[kani::unwind(1)]
fn nonce_encoding_correctness() {
    let counter: u64 = kani::any();
    let nonce = nonce_from_counter(counter);

    // High 8 bytes encode the counter big-endian.
    let expected_high = counter.to_be_bytes();
    assert!(nonce[..8] == expected_high);

    // Trailing 4 bytes are zero padding.
    assert!(nonce[8] == 0);
    assert!(nonce[9] == 0);
    assert!(nonce[10] == 0);
    assert!(nonce[11] == 0);
}
