// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Dual-key authorization — every admin action requires
// two independent authorizations from distinct key holders.
// A single key, even the root key, cannot authorize alone.
//
// ASL-M3: structural enforcement with key slot model.
// ASL-M5: real Ed25519 signature verification wired here.

/// Maximum number of key slots in the dual-key registry.
pub const MAX_KEY_SLOTS: usize = 8;

/// A registered key slot for dual-key authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySlot {
    /// Slot ID (0x01–0x08)
    pub slot_id:    u8,
    /// Key fingerprint (first 8 bytes of Ed25519 public key)
    /// Full key stored in SOMA hardware — fingerprint only in memory
    pub fingerprint: u64,
    pub active:     bool,
}

impl KeySlot {
    pub const fn empty() -> Self {
        Self { slot_id: 0, fingerprint: 0, active: false }
    }
}

/// Authorization token — proves a key slot authorized an action.
#[derive(Debug, Clone, Copy)]
pub struct AuthToken {
    /// Which slot authorized
    pub slot_id:    u8,
    /// Action counter this token covers
    pub counter:    u64,
    /// Signature stub (64 bytes — real Ed25519 in ASL-M5)
    pub sig:        [u8; 64],
}

impl AuthToken {
    pub fn new(slot_id: u8, counter: u64, sig: [u8; 64]) -> Self {
        Self { slot_id, counter, sig }
    }

    /// Returns true if this token's signature is non-zero.
    /// ASL-M3 stub: any non-zero sig accepted.
    /// ASL-M5: real Ed25519 verification.
    pub fn is_valid_stub(&self) -> bool {
        self.sig.iter().any(|&b| b != 0)
    }
}

/// Dual-key registry and authorization engine.
pub struct DualKeyRegistry {
    slots: [KeySlot; MAX_KEY_SLOTS],
    count: usize,
}

impl DualKeyRegistry {
    pub const fn new() -> Self {
        Self {
            slots: [KeySlot::empty(); MAX_KEY_SLOTS],
            count: 0,
        }
    }

    /// Register a key slot.
    pub fn register_slot(
        &mut self,
        slot_id: u8,
        fingerprint: u64,
    ) -> Result<(), DualKeyError> {
        if self.count >= MAX_KEY_SLOTS {
            return Err(DualKeyError::RegistryFull);
        }
        if slot_id == 0 {
            return Err(DualKeyError::InvalidSlotId);
        }
        if self.find_slot(slot_id).is_some() {
            return Err(DualKeyError::SlotAlreadyRegistered);
        }
        if fingerprint == 0 {
            return Err(DualKeyError::ZeroFingerprint);
        }
        self.slots[self.count] = KeySlot {
            slot_id,
            fingerprint,
            active: true,
        };
        self.count += 1;
        Ok(())
    }

    /// Authorize an admin action with two distinct tokens.
    /// Both tokens must:
    ///   1. Reference distinct, registered, active slots
    ///   2. Have valid signatures (stub in ASL-M3)
    ///   3. Cover the same action counter
    pub fn authorize(
        &self,
        token_a: &AuthToken,
        token_b: &AuthToken,
        expected_counter: u64,
    ) -> Result<(), DualKeyError> {
        // Tokens must be from distinct slots
        if token_a.slot_id == token_b.slot_id {
            return Err(DualKeyError::SameSlot);
        }
        // Both slots must be registered and active
        let slot_a = self.find_slot(token_a.slot_id)
            .ok_or(DualKeyError::UnknownSlot)?;
        let slot_b = self.find_slot(token_b.slot_id)
            .ok_or(DualKeyError::UnknownSlot)?;
        if !slot_a.active || !slot_b.active {
            return Err(DualKeyError::SlotInactive);
        }
        // Both tokens must cover the same counter
        if token_a.counter != expected_counter {
            return Err(DualKeyError::CounterMismatch);
        }
        if token_b.counter != expected_counter {
            return Err(DualKeyError::CounterMismatch);
        }
        // Signature validation (stub)
        if !token_a.is_valid_stub() {
            return Err(DualKeyError::InvalidSignature);
        }
        if !token_b.is_valid_stub() {
            return Err(DualKeyError::InvalidSignature);
        }
        Ok(())
    }

    /// Number of registered slots.
    pub fn slot_count(&self) -> usize { self.count }

    /// Returns true if slot is registered and active.
    pub fn slot_active(&self, slot_id: u8) -> bool {
        self.find_slot(slot_id).map(|s| s.active).unwrap_or(false)
    }

    /// Deactivate a slot (key rotation or revocation).
    pub fn deactivate_slot(&mut self, slot_id: u8) -> Result<(), DualKeyError> {
        let count = self.count;
        self.slots[..count]
            .iter_mut()
            .find(|s| s.slot_id == slot_id)
            .map(|s| { s.active = false; })
            .ok_or(DualKeyError::UnknownSlot)
    }

    fn find_slot(&self, slot_id: u8) -> Option<&KeySlot> {
        self.slots[..self.count]
            .iter()
            .find(|s| s.slot_id == slot_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DualKeyError {
    RegistryFull,
    InvalidSlotId,
    SlotAlreadyRegistered,
    ZeroFingerprint,
    SameSlot,
    UnknownSlot,
    SlotInactive,
    CounterMismatch,
    InvalidSignature,
}
