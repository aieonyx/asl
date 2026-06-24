// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// asl-genesis — GENESIS root task
//
// GENESIS is the first process seL4 hands control to after boot.
// Responsibilities (ASL-M1):
//   1. Announce boot with version string
//   2. Run Node Commissioning Ceremony (stub — Ed25519 in ASL-M3)
//   3. Allocate sovereign PD slots
//   4. Surrender authority after commissioning
//
// Post Doctrine gate: all five checks pass before merge.

#![no_std]
#![no_main]

use asl_common::version::ASL_VERSION_STRING;
use asl_common::pd::PdId;

mod commissioning;
mod panic;

/// GENESIS entry point.
/// seL4 calls this after handing over the root CNode and Untyped memory.
#[no_mangle]
pub extern "C" fn main() -> ! {
    // ── Phase 1: Announce ────────────────────────────────────────────
    genesis_log("GENESIS root task starting");
    genesis_log(ASL_VERSION_STRING);

    // ── Phase 2: Node Commissioning Ceremony ─────────────────────────
    commissioning::run();

    // ── Phase 3: Enumerate mandatory PDs ─────────────────────────────
    let mandatory = [
        PdId::ArpiBroker,
        PdId::DataTierEnforcer,
        PdId::TrustGraphGate,
        PdId::InvertedAdmin,
        PdId::AxonBridge,
    ];

    for pd in mandatory.iter() {
        genesis_log_pd("registering mandatory PD", *pd as u8);
    }

    // ── Phase 4: Surrender authority ─────────────────────────────────
    commissioning::surrender_authority();

    genesis_log("GENESIS commissioning complete — authority surrendered");
    genesis_log("Sovereign stack is live.");

    // GENESIS halts after surrender. It holds no further capabilities.
    loop {
        // Idle — all authority has been delegated.
        // In a full seL4 deployment this would call seL4_Yield().
        core::hint::spin_loop();
    }
}

/// Minimal logging for the GENESIS boot phase.
/// Writes to QEMU semihosting output in the M1 stub.
/// Replaced by ARPi-routed logging after ASL-M2.
fn genesis_log(msg: &str) {
    // M1 stub: in real seL4, this would use seL4_DebugPutChar
    // For QEMU test harness, we write to a known memory address
    // that the test runner monitors.
    let _ = msg; // silenced until seL4 IPC wired in ASL-M2
}

fn genesis_log_pd(msg: &str, pd_id: u8) {
    let _ = (msg, pd_id);
}
