// Copyright (c) 2026 Edison Lepiten / AIEONYX

// SPDX-License-Identifier: Apache-2.0

//

// asl-axon-bridge — AXON-Bridge PD (ASL-M5)

//

// The boundary between AXON-compiled userspace and the

// ASL-seL4 mKernel. This PD is the last mandatory sovereign PD.

//

// Responsibilities:

//   1. ABI version contract — compiler emits token, bridge validates

//   2. Capability translation — AXON cap-flow types → seL4 cap objects

//   3. @constant_time enforcement — timing contracts at PD boundary

//   4. AXON-STUB-001 FFI pattern — resolves AXON FFI stubs at link time

//   5. Profile enforcement — seL4-strict profile validated on load

//

// AXON parser constraints (confirmed from axon_sel4 rewrite):

//   - if-condition RHS must be literal when structs in scope

//   - No return inside if blocks — use early return pattern

//   - Variable equality via diff() helper compared to zero

//   - All types i64 at FFI boundary

//

// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓



#![no_std]

#![deny(unsafe_op_in_unsafe_fn)]



#[cfg(kani)]
extern crate kani;

pub mod abi;

pub mod capability;

pub mod constant_time;

pub mod bridge;

pub mod stub;

