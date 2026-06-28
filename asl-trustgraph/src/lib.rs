// Copyright (c) 2026 Edison Lepiten / AIEONYX

// SPDX-License-Identifier: Apache-2.0

//

// asl-trustgraph — TrustGraph-Gate PD (ASL-M3)

//

// The trust graph is a directed capability graph:

//   - Nodes: Protection Domains (PDs)

//   - Edges: granted capabilities between PDs

//   - Tokens: signed proof of a granted capability edge

//

// TrustGraph-Gate validates capability tokens before any

// privileged operation proceeds. The ARPi-Broker tier gate

// calls into TrustGraph-Gate for cross-tier grant validation.

//

// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓



#![no_std]

#![deny(unsafe_op_in_unsafe_fn)]













#[cfg(kani)]
extern crate kani;

pub mod graph;

pub mod token;

pub mod trust_score;

