// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// asl-datatier — DataTier-Enforcer Protection Domain
//
// Enforces three-tier sovereign data policy inside seL4:
//   Critical  — AES-256-GCM encrypted at rest (AUDIT-001 resolved)
//   Personal  — stored with ARPi provenance header, cleartext
//   Noise     — ephemeral, no persistence guarantee
//
// The PD holds a single session key derived via Argon2id at boot from a
// passphrase supplied by SOMA-Identity over the seL4 IPC channel.
// A monotonic nonce counter prevents nonce reuse across the session.

#![no_std]
#![forbid(unsafe_code)]













#[cfg(kani)]
extern crate kani;

extern crate alloc;

use alloc::vec::Vec;
use asl_crypto_bridge::{
    decrypt, derive_key, encrypt, nonce_from_counter, CryptoError, KEY_LEN, NONCE_LEN,
};
use zeroize::Zeroizing;

// ── Tier classification ───────────────────────────────────────────────────────

/// Data sensitivity tier as enforced by the DataTier-Enforcer PD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DataTier {
    /// Encrypted at rest with AES-256-GCM. Identity seeds, credentials, keys.
    Critical = 0,
    /// Stored with ARPi provenance header. User preferences, session state.
    Personal = 1,
    /// Ephemeral. Telemetry, cache, transient buffers.
    Noise = 2,
}

// ── Enforcer errors ───────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum EnforcerError {
    /// Crypto bridge returned an error.
    Crypto(CryptoError),
    /// Session key has not been initialised — call `init_session` first.
    NotInitialised,
    /// Nonce counter overflowed — session must be rekeyed.
    NonceExhausted,
    /// Invalid input (empty record, etc.).
    InvalidInput,
}

impl From<CryptoError> for EnforcerError {
    fn from(e: CryptoError) -> Self {
        EnforcerError::Crypto(e)
    }
}

// ── DataTier-Enforcer PD state ────────────────────────────────────────────────

/// Runtime state of the DataTier-Enforcer PD.
///
/// One instance lives for the lifetime of the PD. Initialised once via
/// `init_session`; never cloned or serialised.
pub struct DataTierEnforcer {
    /// Derived session key. Zeroized on drop.
    session_key: Option<Zeroizing<[u8; KEY_LEN]>>,
    /// Monotonic nonce counter. Incremented on every Critical encrypt call.
    nonce_counter: u64,
}

impl DataTierEnforcer {
    /// Create an uninitialised enforcer. Call `init_session` before use.
    pub const fn new() -> Self {
        Self {
            session_key: None,
            nonce_counter: 0,
        }
    }

    /// Initialise the session key from `passphrase` + `salt` via Argon2id.
    ///
    /// Must be called once at PD boot after SOMA-Identity delivers credentials
    /// over the seL4 IPC channel. Safe to call again to rekey.
    pub fn init_session(&mut self, passphrase: &[u8], salt: &[u8]) -> Result<(), EnforcerError> {
        let key = derive_key(passphrase, salt)?;
        self.session_key = Some(key);
        self.nonce_counter = 0;
        Ok(())
    }

    /// Store a record according to its tier policy.
    ///
    /// - `Critical` → AES-256-GCM encrypt, return ciphertext blob.
    /// - `Personal`  → return plaintext (ARPi header prepended by caller).
    /// - `Noise`     → return plaintext as-is (ephemeral; no persistence).
    pub fn store(
        &mut self,
        tier: DataTier,
        record: &[u8],
    ) -> Result<Vec<u8>, EnforcerError> {
        if record.is_empty() {
            return Err(EnforcerError::InvalidInput);
        }
        match tier {
            DataTier::Critical => self.encrypt_critical(record),
            DataTier::Personal | DataTier::Noise => Ok(record.to_vec()),
        }
    }

    /// Retrieve a record, decrypting if it came from the Critical tier.
    pub fn retrieve(
        &mut self,
        tier: DataTier,
        blob: &[u8],
    ) -> Result<Vec<u8>, EnforcerError> {
        if blob.is_empty() {
            return Err(EnforcerError::InvalidInput);
        }
        match tier {
            DataTier::Critical => self.decrypt_critical(blob),
            DataTier::Personal | DataTier::Noise => Ok(blob.to_vec()),
        }
    }

    /// Returns `true` if the session key has been initialised.
    #[inline]
    pub fn is_initialised(&self) -> bool {
        self.session_key.is_some()
    }

    /// Returns the current nonce counter value (for audit/logging).
    #[inline]
    pub fn nonce_counter(&self) -> u64 {
        self.nonce_counter
    }

    // ── Private ──────────────────────────────────────────────────────────────

    fn encrypt_critical(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, EnforcerError> {
        let key = self.session_key.as_ref().ok_or(EnforcerError::NotInitialised)?;
        let counter = self
            .nonce_counter
            .checked_add(1)
            .ok_or(EnforcerError::NonceExhausted)?;
        self.nonce_counter = counter;

        let nonce = nonce_from_counter(counter);
        // Prepend nonce to ciphertext so decrypt_critical can recover it.
        let mut ct = encrypt(key, &nonce, plaintext)?;
        let mut blob = Vec::with_capacity(NONCE_LEN + ct.len());
        blob.extend_from_slice(&nonce);
        blob.append(&mut ct);
        Ok(blob)
    }

    fn decrypt_critical(&self, blob: &[u8]) -> Result<Vec<u8>, EnforcerError> {
        if blob.len() < NONCE_LEN + 16 {
            return Err(EnforcerError::Crypto(CryptoError::DecryptError));
        }
        let key = self.session_key.as_ref().ok_or(EnforcerError::NotInitialised)?;
        let nonce: [u8; NONCE_LEN] = blob[..NONCE_LEN].try_into().unwrap();
        let ciphertext = &blob[NONCE_LEN..];
        Ok(decrypt(key, &nonce, ciphertext)?)
    }
}

impl Default for DataTierEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &[u8] = b"sovereign-test-passphrase";
    const SALT: &[u8] = b"aieonyx-datatier-salt";

    fn init() -> DataTierEnforcer {
        let mut e = DataTierEnforcer::new();
        e.init_session(PASS, SALT).unwrap();
        e
    }

    #[test]
    fn test_not_initialised_returns_error() {
        let mut e = DataTierEnforcer::new();
        let result = e.store(DataTier::Critical, b"secret");
        assert_eq!(result, Err(EnforcerError::NotInitialised));
    }

    #[test]
    fn test_critical_roundtrip() {
        let mut e = init();
        let record = b"identity-seed-0xCAFEBABE";
        let blob = e.store(DataTier::Critical, record).unwrap();
        let recovered = e.retrieve(DataTier::Critical, &blob).unwrap();
        assert_eq!(recovered, record);
    }

    #[test]
    fn test_critical_ciphertext_differs() {
        let mut e = init();
        let record = b"top-secret";
        let blob = e.store(DataTier::Critical, record).unwrap();
        assert_ne!(&blob[NONCE_LEN..], record.as_slice());
    }

    #[test]
    fn test_two_encrypts_different_nonce() {
        let mut e = init();
        let b1 = e.store(DataTier::Critical, b"record-one").unwrap();
        let b2 = e.store(DataTier::Critical, b"record-one").unwrap();
        // Nonces differ → ciphertexts differ even for identical plaintext.
        assert_ne!(b1, b2);
    }

    #[test]
    fn test_nonce_counter_increments() {
        let mut e = init();
        assert_eq!(e.nonce_counter(), 0);
        e.store(DataTier::Critical, b"a").unwrap();
        assert_eq!(e.nonce_counter(), 1);
        e.store(DataTier::Critical, b"b").unwrap();
        assert_eq!(e.nonce_counter(), 2);
    }

    #[test]
    fn test_personal_passthrough() {
        let mut e = init();
        let record = b"user-preference-dark-mode";
        let blob = e.store(DataTier::Personal, record).unwrap();
        assert_eq!(blob, record);
        let recovered = e.retrieve(DataTier::Personal, &blob).unwrap();
        assert_eq!(recovered, record);
    }

    #[test]
    fn test_noise_passthrough() {
        let mut e = init();
        let record = b"cache-entry-xyz";
        let blob = e.store(DataTier::Noise, record).unwrap();
        assert_eq!(blob, record);
    }

    #[test]
    fn test_empty_record_rejected() {
        let mut e = init();
        assert_eq!(e.store(DataTier::Critical, b""), Err(EnforcerError::InvalidInput));
        assert_eq!(e.store(DataTier::Personal, b""), Err(EnforcerError::InvalidInput));
    }

    #[test]
    fn test_tampered_blob_rejected() {
        let mut e = init();
        let mut blob = e.store(DataTier::Critical, b"sensitive").unwrap();
        blob[NONCE_LEN] ^= 0xFF; // flip first ciphertext byte
        let result = e.retrieve(DataTier::Critical, &blob);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_initialised() {
        let e_uninit = DataTierEnforcer::new();
        assert!(!e_uninit.is_initialised());
        let e_init = init();
        assert!(e_init.is_initialised());
    }
}
