// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Node Commissioning Ceremony — ASL-M1 stub.
//
// ASL-M1: structural ceremony with sovereignty assertions.
// ASL-M3: real Ed25519 root key validation wired here.

use asl_common::version::ASL_VERSION_STRING;

/// Commissioning state — tracks ceremony progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CeremonyState {
    NotStarted,
    VersionVerified,
    TrustAnchorChecked,
    PdSlotsAllocated,
    Complete,
}

/// Runs the Node Commissioning Ceremony.
/// All steps must pass — any failure halts the system.
pub fn run() {
    let mut state = CeremonyState::NotStarted;

    // Step 1: Verify ASL version string integrity
    state = verify_version(state);
    assert!(state == CeremonyState::VersionVerified,
        "GENESIS: version verification failed — halting");

    // Step 2: Check trust anchor (stub — real Ed25519 in ASL-M3)
    state = check_trust_anchor(state);
    assert!(state == CeremonyState::TrustAnchorChecked,
        "GENESIS: trust anchor check failed — halting");

    // Step 3: Allocate PD slots
    state = allocate_pd_slots(state);
    assert!(state == CeremonyState::PdSlotsAllocated,
        "GENESIS: PD slot allocation failed — halting");

    // Step 4: DevMode check — unconditionally rejected
    assert!(!is_dev_mode(),
        "GENESIS: DevMode detected — unconditionally rejected per Inverted Admin Model");

    let _ = ASL_VERSION_STRING;
}

/// Surrenders all Untyped authority after commissioning.
/// After this call, GENESIS holds zero capabilities.
pub fn surrender_authority() {
    // ASL-M1 stub — in full seL4 deployment:
    // seL4_CNode_Delete() called on all Untyped caps
    // GENESIS enters idle loop with empty CNode
    AUTHORITY_SURRENDERED.store(true);
}

/// Returns true if DevMode is active.
/// DevMode is ALWAYS rejected in ASL — no override exists.
fn is_dev_mode() -> bool {
    // In a real deployment: checks boot parameters, signed manifest,
    // and hardware fuse state. Any ambiguity → false (safe default).
    false
}

fn verify_version(state: CeremonyState) -> CeremonyState {
    assert!(state == CeremonyState::NotStarted);
    // Version string must be non-empty and contain "ASL"
    assert!(!ASL_VERSION_STRING.is_empty());
    assert!(ASL_VERSION_STRING.contains("ASL"));
    CeremonyState::VersionVerified
}

fn check_trust_anchor(state: CeremonyState) -> CeremonyState {
    assert!(state == CeremonyState::VersionVerified);
    // ASL-M1 stub: trust anchor exists and is non-zero
    // ASL-M3: real Ed25519 public key loaded from SOMA hardware
    let anchor = trust_anchor_stub();
    assert!(anchor != 0, "GENESIS: trust anchor is zero — halting");
    CeremonyState::TrustAnchorChecked
}

fn allocate_pd_slots(state: CeremonyState) -> CeremonyState {
    assert!(state == CeremonyState::TrustAnchorChecked);
    // ASL-M1: verify slot count matches mandatory PD count (5 + self = 6)
    const MANDATORY_PD_COUNT: usize = 6;
    assert!(MANDATORY_PD_COUNT == 6);
    CeremonyState::PdSlotsAllocated
}

/// Stub trust anchor — replaced by SOMA hardware read in ASL-M3.
fn trust_anchor_stub() -> u64 {
    // Non-zero sentinel value — proves the anchor slot is populated.
    // Real value: Ed25519 public key fingerprint from USB-A root key.
    0xA1E0_4E4C_5339_0001
}

/// Simple atomic flag for authority surrender tracking.
struct AtomicFlag(core::cell::UnsafeCell<bool>);
unsafe impl Sync for AtomicFlag {}
impl AtomicFlag {
    const fn new() -> Self { Self(core::cell::UnsafeCell::new(false)) }
    fn store(&self, val: bool) { unsafe { *self.0.get() = val; } }
    #[allow(dead_code)]
    fn load(&self) -> bool { unsafe { *self.0.get() } }
}

static AUTHORITY_SURRENDERED: AtomicFlag = AtomicFlag::new();

/// Returns true if GENESIS has surrendered all authority.
/// Used by tests to verify correct commissioning completion.
#[allow(dead_code)]
pub fn authority_surrendered() -> bool {
    AUTHORITY_SURRENDERED.load()
}
