// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// DataTier gate — enforces cross-tier grant requirement at the
// ARPi-Broker boundary. No message crosses a tier boundary
// without an explicit signed grant from TrustGraph-Gate.
//
// ASL-M2: grant validation is stubbed (always fails for cross-tier).
// ASL-M4: real TrustGraph-Gate grant tokens wired here.

use asl_common::datatier::DataTier;

/// Result of a tier gate check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierGateResult {
    /// Message may proceed — same tier or downgrade.
    Allow,
    /// Message requires a signed grant — upgrade detected.
    RequiresGrant,
    /// Grant was provided and validated (ASL-M4+).
    GrantAccepted,
}

/// Checks whether a message from src_tier to dst_tier may proceed.
/// If a grant token is provided, it is validated (stub in ASL-M2).
pub fn check(
    src_tier: DataTier,
    dst_tier: DataTier,
    grant_token: Option<&[u8]>,
) -> TierGateResult {
    if !DataTier::requires_grant(src_tier, dst_tier) {
        return TierGateResult::Allow;
    }
    // Cross-tier upgrade: grant required
    match grant_token {
        None => TierGateResult::RequiresGrant,
        Some(token) => {
            // ASL-M2 stub: any non-empty token accepted as placeholder
            // ASL-M4: real Ed25519 grant token verification here
            if token.is_empty() {
                TierGateResult::RequiresGrant
            } else {
                TierGateResult::GrantAccepted
            }
        }
    }
}

/// Canonical tier for GENESIS → ARPi-Broker bootstrap messages.
/// GENESIS operates at Noise tier — it carries no user data.
pub const GENESIS_TIER: DataTier = DataTier::Noise;

/// Canonical tier for DataTier-Enforcer → ARPi-Broker control messages.
pub const DATATIER_CONTROL_TIER: DataTier = DataTier::Noise;
