// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// asl-soma — SOMA-Identity PD (ASL-M4.5)
//
// Implements TriSec Point A: the composite hardware identity
// and three-key threshold encryption binding for all data
// that leaves the sovereign node.
//
// Identity chain:
//   ID-1: Hardware UID (manufacturer, immutable)
//   ID-2: seL4 kernel measurement (boot hash)
//   ID-3: AIEONYX OS UID (paired to ID-1)
//   ID-4: Human biometric hash (optional)
//   → Composite Hashcode: H(ID-1 ‖ ID-2 ‖ ID-3 ‖ ID-4)
//
// Three-key threshold:
//   Key-1: AIEONYX OS key
//   Key-2: EdisonDB-generated key
//   Key-3: Owner key
//   → All three required to decrypt. Any two = noise.
//
// Data-leaves binding:
//   Every outgoing data packet carries the composite hashcode.
//   Without all four identity layers present at destination,
//   the data is structurally unopenable.
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod composite;
pub mod threshold;
pub mod binding;
pub mod soma;
