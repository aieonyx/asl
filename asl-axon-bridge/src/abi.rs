// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AXON-Bridge ABI version contract.
//
// The AXON compiler emits an ABI version token in every
// seL4-target binary. The bridge validates this token before
// loading any AXON userspace binary into a PD.
//
// Version mismatch = binary rejected — no silent degradation.
//
// Token format: 0xAB10_XXYY_ZZZZ_0001
//   AB10 = AXON Bridge prefix
//   XX   = major version
//   YY   = minor version
//   ZZZZ = patch version
//   0001 = AIEONYX marker

/// Current ASL-M5 AXON-Bridge ABI version.
pub const AXON_BRIDGE_ABI_VERSION: u32 = 0x0001_0000; // 1.0.0

/// ABI token prefix — first 16 bits of any valid token.
pub const ABI_TOKEN_PREFIX: u16 = 0xAB10;

/// Full ABI token for ASL v0.1.0 AXON-Bridge.
pub const ABI_TOKEN_V1: u64 = 0xAB10_0100_0000_0001;

/// seL4 target profile required for all AXON userspace binaries.
pub const REQUIRED_PROFILE: &str = "seL4-strict";

/// AXON target triple required for all AXON userspace binaries.
pub const REQUIRED_TARGET: &str = "aarch64-sel4";

/// ABI validation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiResult {
    /// Token valid — binary may be loaded.
    Valid,
    /// Token prefix mismatch — not an AXON-Bridge binary.
    InvalidPrefix,
    /// Version mismatch — binary compiled for different bridge version.
    VersionMismatch { expected: u32, got: u32 },
    /// Token is zero — binary has no ABI token.
    MissingToken,
    /// Token marker invalid — not an AIEONYX binary.
    InvalidMarker,
}

/// Validates an AXON-Bridge ABI token.
pub fn validate_token(token: u64) -> AbiResult {
    if token == 0 {
        return AbiResult::MissingToken;
    }
    // Check prefix (top 16 bits)
    let prefix = (token >> 48) as u16;
    if prefix != ABI_TOKEN_PREFIX {
        return AbiResult::InvalidPrefix;
    }
    // Check AIEONYX marker (bottom 16 bits)
    let marker = (token & 0xFFFF) as u16;
    if marker != 0x0001 {
        return AbiResult::InvalidMarker;
    }
    // Extract version (bits 32-47 = major.minor, bits 16-31 = patch)
    let major = ((token >> 40) & 0xFF) as u8;
    let minor = ((token >> 32) & 0xFF) as u8;
    let patch = ((token >> 16) & 0xFFFF) as u16;
    let got_version = ((major as u32) << 16) | ((minor as u32) << 8) | (patch as u32);

    let expected_major = (AXON_BRIDGE_ABI_VERSION >> 16) as u8;
    // Minor and patch may differ — only major must match
    if major != expected_major {
        return AbiResult::VersionMismatch {
            expected: AXON_BRIDGE_ABI_VERSION,
            got:      got_version,
        };
    }
    AbiResult::Valid
}

/// Extracts the version from a token as (major, minor, patch).
pub fn token_version(token: u64) -> (u8, u8, u16) {
    let major = ((token >> 40) & 0xFF) as u8;
    let minor = ((token >> 32) & 0xFF) as u8;
    let patch = ((token >> 16) & 0xFFFF) as u16;
    (major, minor, patch)
}
