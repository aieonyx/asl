// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// asl-datatier — DataTier-Enforcer PD (ASL-M4)
//
// Enforces EdisonDB Critical/Personal/Noise tier boundaries
// at the kernel level. No data flow crosses a tier boundary
// without a signed grant from TrustGraph-Gate.
//
// Tiers (ascending sensitivity):
//   Noise    — public operational data, no protection required
//   Personal — user-owned, consent required for access
//   Critical — encrypted vault, no plaintext outside vault PD
//
// Also implements GDPR Art.17 erasure hook — Critical tier
// erasure requests are logged and forwarded to EdisonDB.
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod enforcer;
pub mod flow;
pub mod grant;
pub mod erasure;
pub mod audit;
