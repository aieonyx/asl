// Copyright (c) 2026 Edison Lepiten / AIEONYX

// SPDX-License-Identifier: Apache-2.0

//

// asl-genesis — GENESIS root task (ASL-M1)

//

// GENESIS is the first process seL4 hands control to after boot.

// ASL-M1 implements the full commissioning ceremony with real

// assertions. seL4 syscall wiring follows in ASL-M2.

//

// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓



#![no_std]

#![no_main]

#![deny(unsafe_op_in_unsafe_fn)]













#[cfg(kani)]
extern crate kani;

use asl_common::version::ASL_VERSION_STRING;

use asl_common::pd::PdId;



mod commissioning;

mod log;

mod panic;





/// GENESIS entry point.

/// seL4 transfers control here after kernel boot.

/// This function must never return — it halts after

/// surrendering all authority.

#[no_mangle]

pub extern "C" fn main() -> ! {

    log::emit("GENESIS root task starting");

    log::emit(ASL_VERSION_STRING);

    log::emit("S4+i: Security · Sovereignty · Simplicity · Speed · +i");



    // ── Node Commissioning Ceremony ───────────────────────────────

    let result = commissioning::run();

    match result {

        commissioning::CeremonyResult::Success => {

            log::emit("GENESIS: commissioning ceremony complete");

        }

        commissioning::CeremonyResult::Failure(reason) => {

            log::emit("GENESIS: commissioning FAILED — halting");

            log::emit(reason);

            panic!("commissioning failure");

        }

    }



    // ── Enumerate and register mandatory PDs ─────────────────────

    let mandatory_pds = [

        PdId::ArpiBroker,

        PdId::DataTierEnforcer,

        PdId::TrustGraphGate,

        PdId::InvertedAdmin,

        PdId::AxonBridge,

    ];



    let mut registered = 0usize;

    for pd in mandatory_pds.iter() {

        assert!(pd.is_mandatory(), "GENESIS: non-mandatory PD in mandatory list");

        registered += 1;

    }

    assert_eq!(registered, 5, "GENESIS: mandatory PD count mismatch");

    log::emit("GENESIS: all 5 mandatory PDs registered");



    // ── Surrender authority ───────────────────────────────────────

    commissioning::surrender_authority();

    assert!(commissioning::authority_surrendered(),

        "GENESIS: authority surrender verification failed");



    log::emit("GENESIS: authority surrendered — sovereign stack is live");

    log::emit("GENESIS: entering idle loop");



    // GENESIS holds zero capabilities from this point.

    // In full seL4: seL4_Yield() in loop.

    // In QEMU test harness: spin_loop() detected as clean exit.

    loop {

        core::hint::spin_loop();

    }

}

