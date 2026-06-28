// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ASL-M14: Kani formal verification harnesses
//
// These harnesses prove key sovereignty invariants using
// bounded model checking. They complement the 313 unit tests
// with mathematical guarantees over symbolic inputs.
//
// Harness categories:
//   1. ARPi header invariants (size, magic, anti-replay)
//   2. DataTier flow rules (grant requirements)
//   3. SOMA identity properties (hash non-zero, threshold)
//   4. AXON-Bridge ABI contract (token validity)
//   5. Inverted Admin Model (no ambient authority)
//   6. TrustGraph capability model (no self-grant)
//
// Run with: cargo kani --harness <name>
// All harnesses target x86_64 for model checking.
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

pub mod arpi_proofs;
pub mod datatier_proofs;
pub mod soma_proofs;
pub mod abi_proofs;
pub mod admin_proofs;
pub mod trust_proofs;
pub mod security_audit;
pub mod phoenix_proofs;
pub mod repl_proofs;
pub mod demo_proofs;
pub mod crypto_proofs;
pub mod haniel_proofs;
