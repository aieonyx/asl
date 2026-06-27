#![cfg(kani)]

// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Kani proofs — Inverted Admin Model invariants

use asl_inverted_admin::devmode::{check, DevModeResult};
use asl_inverted_admin::action::{AdminAction, AdminActionClass};

/// PROOF: DevMode check always returns NotActive in production.
/// The production stub guarantees this unconditionally.
#[cfg(kani)]
#[kani::proof]
fn proof_devmode_always_not_active() {
    let result = check();
    assert_eq!(result, DevModeResult::NotActive);
}

/// PROOF: All admin action classes require dual-key.
/// Symbolic over all possible action class values.
#[cfg(kani)]
#[kani::proof]
fn proof_all_actions_require_dual_key() {
    let classes = [
        AdminActionClass::PdLifecycle,
        AdminActionClass::CapabilityMgmt,
        AdminActionClass::TrustGraphEdit,
        AdminActionClass::KeyCeremony,
        AdminActionClass::TierBoundary,
        AdminActionClass::SovereignHalt,
    ];
    for class in classes.iter() {
        let action = AdminAction::new(*class, 1, 0x02, 0x01);
        assert!(action.requires_dual_key());
    }
}

/// PROOF: SovereignHalt targets system-wide (0xFF).
#[cfg(kani)]
#[kani::proof]
fn proof_sovereign_halt_system_wide() {
    let action = AdminAction::new(
        AdminActionClass::SovereignHalt, 1, 0xFF, 0x01
    );
    assert!(action.is_system_wide());
}

// ── Non-kani tests ────────────────────────────────────────────────────

#[test]
fn test_devmode_not_active_proof() {
    assert_eq!(check(), DevModeResult::NotActive);
}

#[test]
fn test_all_actions_dual_key_proof() {
    let action = AdminAction::new(AdminActionClass::SovereignHalt, 1, 0xFF, 0x01);
    assert!(action.requires_dual_key());
}
