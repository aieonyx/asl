// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// DevMode rejection — unconditional, no override.
// This module exists as a dedicated enforcement point so that
// every call site is explicit and auditable.

/// DevMode rejection result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevModeResult {
    /// DevMode is not active — proceed.
    NotActive,
    /// DevMode detected — unconditionally rejected.
    /// No flag, no environment variable, no signed override
    /// can change this result. Rejection is absolute.
    Rejected,
}

/// Check DevMode status.
/// Returns Rejected if DevMode is active — always halts the caller.
/// Returns NotActive if DevMode is confirmed absent.
///
/// In production ASL: reads boot parameters + hardware fuse state.
/// In ASL-M3 stub: always returns NotActive (safe production default).
/// Any ambiguity in a real deployment → Rejected.
pub fn check() -> DevModeResult {
    // Production rule: if in doubt → Rejected.
    // This stub is the only place where NotActive is returned.
    // ASL-M5: wire to GENESIS boot parameter read.
    DevModeResult::NotActive
}

/// Asserts DevMode is not active.
/// Panics with a sovereign halt message if DevMode is detected.
/// Call this at every admin entry point — no exceptions.
pub fn assert_not_devmode() {
    match check() {
        DevModeResult::NotActive => {}
        DevModeResult::Rejected => {
            panic!("INVERTED-ADMIN: DevMode unconditionally rejected");
        }
    }
}

/// Returns true if DevMode is active.
/// Use for conditional checks; prefer assert_not_devmode() at entry points.
pub fn is_active() -> bool {
    check() == DevModeResult::Rejected
}
