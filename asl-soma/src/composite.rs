// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Composite Hashcode — four-layer sovereign identity.
// Combines hardware, kernel, OS, and biometric identity
// into a single 32-byte composite hash.
//
// ASL-M4.5: FNV-1a hash composition (structural).
// ASL-M7: real hardware UID read from SOMA USB device.
// ASL-M10+: real seL4 boot measurement wired here.

/// Size of the composite hashcode in bytes.
pub const COMPOSITE_HASH_SIZE: usize = 32;

/// AIEONYX composite identity prefix marker.
pub const AIEONYX_IDENTITY_MARKER: u64 = 0xA1E0_1D_00_0000_0001;

/// Four identity layers that compose the sovereign hashcode.
#[derive(Debug, Clone, Copy)]
pub struct IdentityLayers {
    /// ID-1: Hardware UID from manufacturer (CPU serial, TPM, etc.)
    /// ASL-M4.5: stub sentinel. ASL-M7: real SOMA hardware read.
    pub hw_uid:      u64,
    /// ID-2: seL4 kernel boot measurement hash (first 8 bytes)
    /// ASL-M4.5: stub. ASL-M10: real seL4 boot attestation.
    pub kernel_meas: u64,
    /// ID-3: AIEONYX OS UID — cryptographically paired to hw_uid
    pub os_uid:      u64,
    /// ID-4: Human biometric hash (optional — zero if not enrolled)
    pub biometric:   u64,
}

impl IdentityLayers {
    pub fn new(hw_uid: u64, kernel_meas: u64, os_uid: u64, biometric: u64) -> Self {
        Self { hw_uid, kernel_meas, os_uid, biometric }
    }

    /// Creates a stub identity for ASL-M4.5 testing.
    /// All four layers are populated with non-zero sentinels.
    pub fn stub() -> Self {
        Self {
            hw_uid:      0xA1E0_0000_0001_0001, // HW sentinel
            kernel_meas: 0xA1E0_0000_0002_0001, // Kernel sentinel
            os_uid:      0xA1E0_0000_0003_0001, // OS sentinel
            biometric:   0xA1E0_0000_0004_0001, // Biometric sentinel
        }
    }

    /// Returns true if all four identity layers are populated.
    pub fn is_complete(&self) -> bool {
        self.hw_uid != 0
            && self.kernel_meas != 0
            && self.os_uid != 0
        // biometric is optional — zero allowed
    }

    /// Returns true if biometric is enrolled.
    pub fn has_biometric(&self) -> bool {
        self.biometric != 0
    }

    /// Returns true if OS UID is plausibly paired to HW UID.
    /// Full cryptographic pairing verified in ASL-M7.
    pub fn os_paired_to_hw(&self) -> bool {
        // Stub: both must share the AIEONYX prefix
        (self.hw_uid >> 48) == 0xA1E0
            && (self.os_uid >> 48) == 0xA1E0
    }
}

/// The composite hashcode — 32 bytes derived from all four layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompositeHash([u8; COMPOSITE_HASH_SIZE]);

impl CompositeHash {
    /// Derives the composite hash from identity layers.
    /// Uses FNV-1a composition (structural — real SHA-256 in ASL-M7).
    pub fn derive(layers: &IdentityLayers) -> Result<Self, IdentityError> {
        if !layers.is_complete() {
            return Err(IdentityError::IncompleteIdentity);
        }

        // FNV-1a 64-bit basis
        const FNV_BASIS: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x00000100000001b3;

        // Hash each layer sequentially
        let mut h = FNV_BASIS;
        for &val in &[
            layers.hw_uid,
            layers.kernel_meas,
            layers.os_uid,
            layers.biometric,
            AIEONYX_IDENTITY_MARKER,
        ] {
            for byte in val.to_le_bytes() {
                h ^= byte as u64;
                h = h.wrapping_mul(FNV_PRIME);
            }
        }

        // Expand 64-bit hash to 32 bytes via second FNV pass
        let mut result = [0u8; COMPOSITE_HASH_SIZE];
        let h2 = h.wrapping_mul(FNV_PRIME) ^ 0xDEAD_BEEF_A1E0_0001;
        let h3 = h2.wrapping_mul(FNV_PRIME) ^ 0xA1E0_CAFE_BABE_0002;
        let h4 = h3.wrapping_mul(FNV_PRIME) ^ 0x5AFE_DEAD_C0DE_0003;
        result[0..8].copy_from_slice(&h.to_le_bytes());
        result[8..16].copy_from_slice(&h2.to_le_bytes());
        result[16..24].copy_from_slice(&h3.to_le_bytes());
        result[24..32].copy_from_slice(&h4.to_le_bytes());

        Ok(CompositeHash(result))
    }

    /// Returns the raw bytes of the composite hash.
    pub fn as_bytes(&self) -> &[u8; COMPOSITE_HASH_SIZE] { &self.0 }

    /// Returns true if the hash is non-zero (valid).
    pub fn is_valid(&self) -> bool {
        self.0.iter().any(|&b| b != 0)
    }

    /// Returns the first 8 bytes as a u64 fingerprint.
    pub fn fingerprint(&self) -> u64 {
        u64::from_le_bytes(self.0[0..8].try_into().unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityError {
    /// One or more mandatory identity layers is zero.
    IncompleteIdentity,
    /// Hardware UID and OS UID pairing failed.
    PairingFailed,
    /// Biometric required but not enrolled.
    BiometricRequired,
}
