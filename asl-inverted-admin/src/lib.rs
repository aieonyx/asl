// Copyright (c) 2026 Edison Lepiten / AIEONYX

// SPDX-License-Identifier: Apache-2.0

//

// asl-inverted-admin — Inverted Admin Model PD (ASL-M3)

//

// Core principle: NO ambient authority exists in the sovereign stack.

// Every admin action requires:

//   1. An explicit capability token from TrustGraph-Gate

//   2. A second authorization from a distinct key holder

//   3. A monotonic action counter (prevents replay)

//

// DevMode is UNCONDITIONALLY rejected at every call site.

// No flag, no override, no escape hatch exists.

//

// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓



#![no_std]

#![deny(unsafe_op_in_unsafe_fn)]













#[cfg(kani)]
extern crate kani;

pub mod admin;

pub mod devmode;

pub mod dual_key;

pub mod action;

