// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// GENESIS boot logger — ASL-M1 stub.
// ASL-M2: replaced by ARPi-routed sovereign logging.
// Uses seL4_DebugPutChar in full deployment.

/// Emit a log line from GENESIS.
/// ASL-M1: no-op stub (output wired in ASL-M2 via seL4 debug channel).
#[inline]
pub fn emit(_msg: &str) {
    // Intentional no-op in ASL-M1.
    // seL4_DebugPutChar loop wired here in ASL-M2.
}
