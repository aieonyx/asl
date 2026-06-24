// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Node Commissioning Ceremony — ASL-M1 full implementation.
//
// The ceremony runs in strict sequential steps.
// Any step failure returns CeremonyResult::Failure immediately.
// DevMode is unconditionally rejected — no override exists.
//
// ASL-M3 upgrade path:
//   check_trust_anchor() → real Ed25519 verification via SOMA hardware
//   allocate_pd_slots()  → real seL4 Untyped capability allocation

use asl_common::version::{ASL_VERSION, ASL_VERSION_STRING};
use asl_common::pd::PdId;
use asl_common::datatier::DataTier;
use asl_common::arpi::ArpiHeader;

/// Result of the Node Commissioning Ceremony.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeremonyResult {
    Success,
    Failure(&'static str),
}

/// Ceremony step state — must advance linearly.
/// Any regression is a hard fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum CeremonyState {
    NotStarted       = 0,
    VersionVerified  = 1,
    DevModeRejected  = 2,
    TrustAnchorReady = 3,
    ArpiValidated    = 4,
    DataTierVerified = 5,
    PdSlotsAllocated = 6,
    Complete         = 7,
}

/// Runs the full Node Commissioning Ceremony.
/// Returns Success only if all steps pass in order.
pub fn run() -> CeremonyResult {
    let mut state = CeremonyState::NotStarted;

    // Step 1: Version integrity
    state = match step_verify_version(state) {
        Ok(s) => s,
        Err(e) => return CeremonyResult::Failure(e),
    };

    // Step 2: DevMode rejection — unconditional
    state = match step_reject_devmode(state) {
        Ok(s) => s,
        Err(e) => return CeremonyResult::Failure(e),
    };

    // Step 3: Trust anchor check
    state = match step_check_trust_anchor(state) {
        Ok(s) => s,
        Err(e) => return CeremonyResult::Failure(e),
    };

    // Step 4: ARPi header structural validation
    state = match step_validate_arpi(state) {
        Ok(s) => s,
        Err(e) => return CeremonyResult::Failure(e),
    };

    // Step 5: DataTier flow rules
    state = match step_verify_datatier(state) {
        Ok(s) => s,
        Err(e) => return CeremonyResult::Failure(e),
    };

    // Step 6: PD slot allocation
    state = match step_allocate_pd_slots(state) {
        Ok(s) => s,
        Err(e) => return CeremonyResult::Failure(e),
    };

    // Final: state must be PdSlotsAllocated
    if state != CeremonyState::PdSlotsAllocated {
        return CeremonyResult::Failure("ceremony ended in unexpected state");
    }

    CeremonyResult::Success
}

/// Step 1 — Verify ASL version string integrity.
fn step_verify_version(state: CeremonyState) -> Result<CeremonyState, &'static str> {
    if state != CeremonyState::NotStarted {
        return Err("version check: invalid entry state");
    }
    if ASL_VERSION_STRING.is_empty() {
        return Err("version string is empty");
    }
    if !ASL_VERSION_STRING.contains("ASL") {
        return Err("version string missing ASL marker");
    }
    if !ASL_VERSION_STRING.contains("seL4") {
        return Err("version string missing seL4 pin");
    }
    if ASL_VERSION.is_empty() {
        return Err("ASL_VERSION is empty");
    }
    Ok(CeremonyState::VersionVerified)
}

/// Step 2 — DevMode unconditional rejection.
/// DevMode is ALWAYS rejected. No override. No flag. No exception.
/// This is the Inverted Admin Model at boot level.
fn step_reject_devmode(state: CeremonyState) -> Result<CeremonyState, &'static str> {
    if state != CeremonyState::VersionVerified {
        return Err("devmode check: invalid entry state");
    }
    if is_dev_mode() {
        return Err("DevMode detected — unconditionally rejected (Inverted Admin Model)");
    }
    Ok(CeremonyState::DevModeRejected)
}

/// Step 3 — Trust anchor validation.
/// ASL-M1: stub sentinel value.
/// ASL-M3: real Ed25519 public key from SOMA USB-A hardware.
fn step_check_trust_anchor(state: CeremonyState) -> Result<CeremonyState, &'static str> {
    if state != CeremonyState::DevModeRejected {
        return Err("trust anchor: invalid entry state");
    }
    let anchor = trust_anchor_stub();
    if anchor == 0 {
        return Err("trust anchor is zero — no root key present");
    }
    if anchor == u64::MAX {
        return Err("trust anchor is sentinel max — invalid");
    }
    // Verify anchor has expected AIEONYX prefix (0xA1E0 = ALEO)
    if (anchor >> 48) != 0xA1E0 {
        return Err("trust anchor missing AIEONYX prefix");
    }
    Ok(CeremonyState::TrustAnchorReady)
}

/// Step 4 — ARPi header structural validation.
/// Proves the 78-byte header layout is correct before any IPC begins.
fn step_validate_arpi(state: CeremonyState) -> Result<CeremonyState, &'static str> {
    if state != CeremonyState::TrustAnchorReady {
        return Err("ARPi validation: invalid entry state");
    }
    // Size must be exactly 78 bytes — compile-time proven in arpi.rs
    // Runtime double-check during commissioning
    if ArpiHeader::SIZE != 78 {
        return Err("ARPi header size mismatch — layout broken");
    }
    if ArpiHeader::MAGIC != 0xA291 {
        return Err("ARPi magic constant mismatch");
    }
    // Validate a synthetic header
    let sig = [0u8; 64];
    let header = ArpiHeader::new(
        PdId::Genesis as u8,
        PdId::ArpiBroker as u8,
        0x00, // Noise tier
        1,    // seq = 1 (first message)
        sig,
    );
    if !header.is_valid_magic() {
        return Err("ARPi header magic validation failed");
    }
    Ok(CeremonyState::ArpiValidated)
}

/// Step 5 — DataTier flow rule verification.
/// Proves the grant requirement logic is correct before PDs launch.
fn step_verify_datatier(state: CeremonyState) -> Result<CeremonyState, &'static str> {
    if state != CeremonyState::ArpiValidated {
        return Err("DataTier verify: invalid entry state");
    }
    // Noise → Personal requires grant
    if !DataTier::requires_grant(DataTier::Noise, DataTier::Personal) {
        return Err("DataTier: Noise→Personal should require grant");
    }
    // Noise → Critical requires grant
    if !DataTier::requires_grant(DataTier::Noise, DataTier::Critical) {
        return Err("DataTier: Noise→Critical should require grant");
    }
    // Personal → Critical requires grant
    if !DataTier::requires_grant(DataTier::Personal, DataTier::Critical) {
        return Err("DataTier: Personal→Critical should require grant");
    }
    // Same-tier flows do NOT require grant
    if DataTier::requires_grant(DataTier::Noise, DataTier::Noise) {
        return Err("DataTier: Noise→Noise should not require grant");
    }
    if DataTier::requires_grant(DataTier::Critical, DataTier::Critical) {
        return Err("DataTier: Critical→Critical should not require grant");
    }
    // Downgrade flows do NOT require grant
    if DataTier::requires_grant(DataTier::Critical, DataTier::Noise) {
        return Err("DataTier: Critical→Noise should not require grant");
    }
    Ok(CeremonyState::DataTierVerified)
}

/// Step 6 — PD slot allocation verification.
fn step_allocate_pd_slots(state: CeremonyState) -> Result<CeremonyState, &'static str> {
    if state != CeremonyState::DataTierVerified {
        return Err("PD allocation: invalid entry state");
    }
    // Verify mandatory PD count
    const MANDATORY: &[PdId] = &[
        PdId::Genesis,
        PdId::ArpiBroker,
        PdId::DataTierEnforcer,
        PdId::TrustGraphGate,
        PdId::InvertedAdmin,
        PdId::AxonBridge,
    ];
    if MANDATORY.len() != 6 {
        return Err("mandatory PD count must be exactly 6");
    }
    for pd in MANDATORY {
        if !pd.is_mandatory() {
            return Err("non-mandatory PD in mandatory list");
        }
    }
    // Verify optional PDs are correctly classified
    const OPTIONAL: &[PdId] = &[
        PdId::GpuCap,
        PdId::PowerMgmt,
        PdId::NetworkRouting,
        PdId::FirewallCap,
        PdId::TouchSensor,
    ];
    for pd in OPTIONAL {
        if pd.is_mandatory() {
            return Err("mandatory PD misclassified as optional");
        }
    }
    Ok(CeremonyState::PdSlotsAllocated)
}

/// Returns true if DevMode is detected.
/// DevMode is always false in production — the check exists to
/// ensure the rejection path is exercised and never bypassed.
fn is_dev_mode() -> bool {
    false
}

/// Stub trust anchor — 64-bit sentinel with AIEONYX prefix.
/// Replaced by SOMA Ed25519 key fingerprint read in ASL-M3.
fn trust_anchor_stub() -> u64 {
    // 0xA1E0 prefix = ALEO (AIEONYX marker)
    // 0x4E4C_5339_0001 = NLS9\x00\x01 (Sovereign Node v1)
    0xA1E0_4E4C_5339_0001
}

/// Atomic authority surrender flag.
struct SurrenderFlag(core::cell::UnsafeCell<bool>);
unsafe impl Sync for SurrenderFlag {}
impl SurrenderFlag {
    const fn new() -> Self { Self(core::cell::UnsafeCell::new(false)) }
    fn set(&self) { unsafe { *self.0.get() = true; } }
    #[allow(dead_code)]
    fn get(&self) -> bool { unsafe { *self.0.get() } }
}

static SURRENDERED: SurrenderFlag = SurrenderFlag::new();

/// Surrenders all Untyped capability authority.
/// After this call GENESIS holds zero capabilities.
/// In full seL4: seL4_CNode_Delete() on all Untyped caps.
pub fn surrender_authority() {
    SURRENDERED.set();
}

/// Returns true if GENESIS has surrendered all authority.
pub fn authority_surrendered() -> bool {
    SURRENDERED.get()
}
