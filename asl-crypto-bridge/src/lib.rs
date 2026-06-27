// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// asl-crypto-bridge — Sovereign Crypto Bridge
// Bridges AES-256-GCM encryption and Argon2 KDF into the DataTier-Enforcer PD.
// Resolves AUDIT-001: Critical tier data was stored plaintext.
//
// Design principles (S4+i):
//   Security   — AES-256-GCM with unique nonce per encrypt call
//   Sovereignty — no_std compatible; no OS entropy dependency at runtime
//   Simplicity  — three public functions: derive_key, encrypt, decrypt
//   Speed       — zero-copy where possible; zeroize on drop

#![no_std]
#![forbid(unsafe_code)]

#[cfg(kani)]
extern crate kani;

extern crate alloc;

use alloc::vec::Vec;
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::Zeroizing;

// ── Constants ────────────────────────────────────────────────────────────────

/// AES-256-GCM key length in bytes.
pub const KEY_LEN: usize = 32;

/// AES-256-GCM nonce length in bytes (96-bit).
pub const NONCE_LEN: usize = 12;

/// Argon2id parameters — conservative for embedded PD context.
/// m=64 KiB, t=3 iterations, p=1 lane.
pub const ARGON2_M_COST: u32 = 65536;
pub const ARGON2_T_COST: u32 = 3;
pub const ARGON2_P_COST: u32 = 1;

/// AAD tag used for Critical tier records.
pub const CRITICAL_AAD: &[u8] = b"ASL:DataTier:Critical:v1";

// ── Error type ───────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum CryptoError {
    /// Argon2 KDF failed.
    KdfError,
    /// AES-256-GCM encryption failed.
    EncryptError,
    /// AES-256-GCM decryption / authentication failed.
    DecryptError,
    /// Input length constraint violated.
    InvalidInput,
}

// ── Key derivation ────────────────────────────────────────────────────────────

/// Derive a 32-byte AES-256-GCM key from a passphrase and salt using Argon2id.
///
/// The returned key is wrapped in `Zeroizing` — it is zeroed on drop.
///
/// # Errors
/// Returns `CryptoError::KdfError` if Argon2 fails (malformed params or OOM).
pub fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    if passphrase.is_empty() || salt.is_empty() {
        return Err(CryptoError::InvalidInput);
    }

    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_LEN))
        .map_err(|_| CryptoError::KdfError)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key_bytes = Zeroizing::new([0u8; KEY_LEN]);
    argon2
        .hash_password_into(passphrase, salt, key_bytes.as_mut())
        .map_err(|_| CryptoError::KdfError)?;

    Ok(key_bytes)
}

// ── Encryption ────────────────────────────────────────────────────────────────

/// Encrypt `plaintext` with AES-256-GCM.
///
/// `key_bytes` must be exactly 32 bytes.
/// `nonce_bytes` must be exactly 12 bytes — callers must supply a unique nonce
/// per encryption call (e.g. a monotonic counter serialised to 12 bytes).
///
/// Returns ciphertext with the GCM authentication tag appended (ciphertext_len + 16 bytes).
///
/// # Errors
/// Returns `CryptoError::EncryptError` on failure.
pub fn encrypt(
    key_bytes: &[u8; KEY_LEN],
    nonce_bytes: &[u8; NONCE_LEN],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let payload = Payload {
        msg: plaintext,
        aad: CRITICAL_AAD,
    };

    cipher.encrypt(nonce, payload).map_err(|_| CryptoError::EncryptError)
}

// ── Decryption ────────────────────────────────────────────────────────────────

/// Decrypt and authenticate `ciphertext` with AES-256-GCM.
///
/// `ciphertext` must include the appended 16-byte GCM tag.
///
/// # Errors
/// Returns `CryptoError::DecryptError` if authentication fails or ciphertext is malformed.
pub fn decrypt(
    key_bytes: &[u8; KEY_LEN],
    nonce_bytes: &[u8; NONCE_LEN],
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    if ciphertext.len() < 16 {
        return Err(CryptoError::DecryptError);
    }

    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let payload = Payload {
        msg: ciphertext,
        aad: CRITICAL_AAD,
    };

    cipher.decrypt(nonce, payload).map_err(|_| CryptoError::DecryptError)
}

// ── Nonce utilities ───────────────────────────────────────────────────────────

/// Encode a monotonic u64 counter into a 12-byte nonce.
/// Bytes 0–7: counter (big-endian). Bytes 8–11: zero-padded.
/// Safe as long as the counter never wraps — PD responsibility.
#[inline]
pub fn nonce_from_counter(counter: u64) -> [u8; NONCE_LEN] {
    let mut n = [0u8; NONCE_LEN];
    n[..8].copy_from_slice(&counter.to_be_bytes());
    n
}

// ── Smoke tests (std only) ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PASS: &[u8] = b"sovereign-passphrase-test";
    const TEST_SALT: &[u8] = b"aieonyx-salt-m18";
    const TEST_DATA: &[u8] = b"Critical tier record: identity seed 0xDEAD";

    #[test]
    fn test_derive_key_deterministic() {
        let k1 = derive_key(TEST_PASS, TEST_SALT).unwrap();
        let k2 = derive_key(TEST_PASS, TEST_SALT).unwrap();
        assert_eq!(*k1, *k2);
    }

    #[test]
    fn test_derive_key_different_salt() {
        let k1 = derive_key(TEST_PASS, TEST_SALT).unwrap();
        let k2 = derive_key(TEST_PASS, b"different-salt").unwrap();
        assert_ne!(*k1, *k2);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_key(TEST_PASS, TEST_SALT).unwrap();
        let nonce = nonce_from_counter(1);
        let ct = encrypt(&*key, &nonce, TEST_DATA).unwrap();
        let pt = decrypt(&*key, &nonce, &ct).unwrap();
        assert_eq!(pt, TEST_DATA);
    }

    #[test]
    fn test_ciphertext_differs_from_plaintext() {
        let key = derive_key(TEST_PASS, TEST_SALT).unwrap();
        let nonce = nonce_from_counter(42);
        let ct = encrypt(&*key, &nonce, TEST_DATA).unwrap();
        assert_ne!(ct, TEST_DATA);
    }

    #[test]
    fn test_wrong_key_fails_decrypt() {
        let key = derive_key(TEST_PASS, TEST_SALT).unwrap();
        let wrong_key = derive_key(b"wrong", TEST_SALT).unwrap();
        let nonce = nonce_from_counter(1);
        let ct = encrypt(&*key, &nonce, TEST_DATA).unwrap();
        let result = decrypt(&*wrong_key, &nonce, &ct);
        assert_eq!(result, Err(CryptoError::DecryptError));
    }

    #[test]
    fn test_nonce_uniqueness() {
        let n0 = nonce_from_counter(0);
        let n1 = nonce_from_counter(1);
        assert_ne!(n0, n1);
    }

    #[test]
    fn test_empty_passphrase_rejected() {
        let result = derive_key(b"", TEST_SALT);
        assert_eq!(result, Err(CryptoError::InvalidInput));
    }

    #[test]
    fn test_empty_salt_rejected() {
        let result = derive_key(TEST_PASS, b"");
        assert_eq!(result, Err(CryptoError::InvalidInput));
    }

    #[test]
    fn test_tampered_ciphertext_rejected() {
        let key = derive_key(TEST_PASS, TEST_SALT).unwrap();
        let nonce = nonce_from_counter(7);
        let mut ct = encrypt(&*key, &nonce, TEST_DATA).unwrap();
        ct[0] ^= 0xFF; // flip first byte
        let result = decrypt(&*key, &nonce, &ct);
        assert_eq!(result, Err(CryptoError::DecryptError));
    }

    #[test]
    fn test_nonce_counter_encoding() {
        let n = nonce_from_counter(0x0102030405060708u64);
        assert_eq!(&n[..8], &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
        assert_eq!(&n[8..], &[0x00, 0x00, 0x00, 0x00]);
    }
}
