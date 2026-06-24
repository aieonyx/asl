// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Three-key threshold encryption model.
//
// All three keys required to decrypt. Any two = encrypted noise.
// Key-1: AIEONYX OS key (held by OS PD)
// Key-2: EdisonDB key (held by EdisonDB PD)
// Key-3: Owner key (held by human owner, never stored on device)
//
// ASL-M4.5: structural model, key slot enforcement, threshold logic.
// ASL-M7: real Ed25519 + AES-256-GCM key derivation wired here.

/// Key slot identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeySlotId {
    /// Key-1: AIEONYX OS key
    OsKey      = 0x01,
    /// Key-2: EdisonDB-generated key
    EdisonDbKey = 0x02,
    /// Key-3: Owner key (human-held, never stored)
    OwnerKey   = 0x03,
}

/// Threshold key slot — holds a key fingerprint (not the key itself).
#[derive(Debug, Clone, Copy)]
pub struct ThresholdKeySlot {
    pub slot_id:     KeySlotId,
    /// First 8 bytes of the key's Ed25519 public key fingerprint.
    /// Full key is never in memory — only the fingerprint.
    pub fingerprint: u64,
    pub enrolled:    bool,
}

impl ThresholdKeySlot {
    pub fn new(slot_id: KeySlotId, fingerprint: u64) -> Result<Self, ThresholdError> {
        if fingerprint == 0 {
            return Err(ThresholdError::ZeroFingerprint);
        }
        Ok(Self { slot_id, fingerprint, enrolled: true })
    }

    pub const fn empty(slot_id: KeySlotId) -> Self {
        Self { slot_id, fingerprint: 0, enrolled: false }
    }
}

/// The three-key threshold registry.
pub struct ThresholdRegistry {
    os_key:       ThresholdKeySlot,
    edisondb_key: ThresholdKeySlot,
    owner_key:    ThresholdKeySlot,
}

impl ThresholdRegistry {
    pub const fn new() -> Self {
        Self {
            os_key:       ThresholdKeySlot::empty(KeySlotId::OsKey),
            edisondb_key: ThresholdKeySlot::empty(KeySlotId::EdisonDbKey),
            owner_key:    ThresholdKeySlot::empty(KeySlotId::OwnerKey),
        }
    }

    /// Enroll a key slot with its fingerprint.
    pub fn enroll(&mut self, slot: KeySlotId, fingerprint: u64) -> Result<(), ThresholdError> {
        let key_slot = ThresholdKeySlot::new(slot, fingerprint)?;
        match slot {
            KeySlotId::OsKey       => self.os_key = key_slot,
            KeySlotId::EdisonDbKey => self.edisondb_key = key_slot,
            KeySlotId::OwnerKey    => self.owner_key = key_slot,
        }
        Ok(())
    }

    /// Returns true if all three keys are enrolled.
    pub fn is_complete(&self) -> bool {
        self.os_key.enrolled
            && self.edisondb_key.enrolled
            && self.owner_key.enrolled
    }

    /// Returns number of enrolled keys (0-3).
    pub fn enrolled_count(&self) -> usize {
        [&self.os_key, &self.edisondb_key, &self.owner_key]
            .iter()
            .filter(|k| k.enrolled)
            .count()
    }

    /// Checks if a set of presented key fingerprints meets threshold.
    /// Threshold = all three keys must be present.
    /// Any two = threshold NOT met = data remains encrypted.
    pub fn check_threshold(
        &self,
        presented: &[u64],
    ) -> ThresholdResult {
        if !self.is_complete() {
            return ThresholdResult::NotEnrolled;
        }
        let has_os  = presented.contains(&self.os_key.fingerprint);
        let has_edb = presented.contains(&self.edisondb_key.fingerprint);
        let has_own = presented.contains(&self.owner_key.fingerprint);

        let count = [has_os, has_edb, has_own].iter().filter(|&&b| b).count();

        match count {
            3 => ThresholdResult::ThresholdMet,
            2 => ThresholdResult::PartialMatch(2), // encrypted noise
            1 => ThresholdResult::PartialMatch(1),
            _ => ThresholdResult::NoMatch,
        }
    }

    /// Revoke a key slot (key rotation or owner change).
    pub fn revoke(&mut self, slot: KeySlotId) {
        match slot {
            KeySlotId::OsKey       => self.os_key.enrolled = false,
            KeySlotId::EdisonDbKey => self.edisondb_key.enrolled = false,
            KeySlotId::OwnerKey    => self.owner_key.enrolled = false,
        }
    }

    pub fn slot_enrolled(&self, slot: KeySlotId) -> bool {
        match slot {
            KeySlotId::OsKey       => self.os_key.enrolled,
            KeySlotId::EdisonDbKey => self.edisondb_key.enrolled,
            KeySlotId::OwnerKey    => self.owner_key.enrolled,
        }
    }
}

/// Result of a threshold check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdResult {
    /// All three keys present — data may be decrypted.
    ThresholdMet,
    /// Only N keys present — data remains encrypted noise.
    PartialMatch(usize),
    /// No keys matched.
    NoMatch,
    /// Registry not fully enrolled yet.
    NotEnrolled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdError {
    ZeroFingerprint,
}
