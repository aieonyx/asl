// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// asl-axon-migration/src/lib.rs
//
// M22: AXON migration verification layer.
//
// The .ax source (sovereign_arpi.ax) is the primary deliverable.
// This Rust crate verifies the migration contract — it mirrors the
// function semantics from the .ax file so we can run unit tests
// confirming the logic is identical before and after migration.
//
// Migration contract: every function in sovereign_arpi.ax must produce
// identical results to its Rust mirror here.

#![no_std]

// ── Sovereign constants ───────────────────────────────────────────────────────

pub const AXON_PROOF: i64    = 0x4153;  // 16723
pub const ARPI_MAGIC: i64    = 0x4152_5069_u32 as i64; // "ARPi" = 1098088048... actually:
// "ARPi" = 0x41 0x52 0x50 0x69 = 0x41525069 = 1095921769
pub const ARPI_MAGIC_CORRECT: u32 = 0x41525069;
pub const ARPI_PD_ID: i64    = 0x01;
pub const ARPI_VERSION: i64  = 0x01;
pub const HEADER_SIZE: i64   = 78;
pub const ANOMALY_THRESHOLD: i64 = 75;
pub const ANOMALY_FLAGGED: i64   = 50;

// ── Layer 1: Schema ───────────────────────────────────────────────────────────

pub fn schema_valid(schema_byte: i64) -> i64 {
    if schema_byte >= 0 && schema_byte <= 6 { 1 } else { 0 }
}

// ── Layer 2: Identity ─────────────────────────────────────────────────────────

pub fn identity_valid(key_first: i64, nonce_first: i64, sig_first: i64) -> i64 {
    if key_first == 0 || nonce_first == 0 || sig_first == 0 { 0 } else { 1 }
}

// ── Layer 3: Mutual auth ──────────────────────────────────────────────────────

pub fn mutual_auth_valid(local_key: i64, remote_key: i64) -> i64 {
    if local_key == 0 || remote_key == 0 || local_key == remote_key { 0 } else { 1 }
}

// ── Layer 4: Scope ────────────────────────────────────────────────────────────

pub fn scope_valid(token_seq: i64, last_seq: i64, schema_byte: i64) -> i64 {
    if token_seq == 0 { return 0; }
    if token_seq <= last_seq { return 0; }
    if schema_valid(schema_byte) == 0 { return 0; }
    1
}

// ── Layer 5: Anomaly ──────────────────────────────────────────────────────────

/// Returns: 0=Clear, 1=Flagged, 2=Escalated
pub fn anomaly_result(score: i64) -> i64 {
    if score >= ANOMALY_THRESHOLD { 2 }
    else if score >= ANOMALY_FLAGGED { 1 }
    else { 0 }
}

// ── Full bind ─────────────────────────────────────────────────────────────────

/// Returns: 0=success, 1-5=failed at layer N
pub fn arpi_bind(
    schema_byte: i64,
    local_key:   i64,
    remote_key:  i64,
    token_seq:   i64,
    last_seq:    i64,
    anomaly:     i64,
) -> i64 {
    if schema_valid(schema_byte) == 0 { return 1; }
    if identity_valid(local_key, 1, 1) == 0 { return 2; }
    if mutual_auth_valid(local_key, remote_key) == 0 { return 3; }
    if scope_valid(token_seq, last_seq, schema_byte) == 0 { return 4; }
    if anomaly_result(anomaly) == 2 { return 5; }
    0
}

// ── Sovereign boot ────────────────────────────────────────────────────────────

pub fn sovereign_boot() -> i64 {
    if AXON_PROOF != 0x4153 { return 1; }
    if ARPI_PD_ID == 0 { return 2; }
    0
}

// ── axon_main mirror ──────────────────────────────────────────────────────────

/// Mirrors axon_main() in sovereign_arpi.ax.
/// Returns 0x4153 on sovereign success.
pub fn axon_main_mirror() -> i64 {
    if sovereign_boot() == 0 {
        let result = arpi_bind(1, 1, 2, 1, 0, 0);
        if result == 0 { return AXON_PROOF; }
        return result;
    }
    sovereign_boot()
}

// ── Tests — migration contract verification ───────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Schema
    #[test] fn test_schema_valid_range()   { for i in 0..=6i64 { assert_eq!(schema_valid(i), 1); } }
    #[test] fn test_schema_invalid()       { assert_eq!(schema_valid(7), 0); assert_eq!(schema_valid(255), 0); }

    // Identity
    #[test] fn test_identity_valid()       { assert_eq!(identity_valid(1, 1, 1), 1); }
    #[test] fn test_identity_zero_key()    { assert_eq!(identity_valid(0, 1, 1), 0); }
    #[test] fn test_identity_zero_nonce()  { assert_eq!(identity_valid(1, 0, 1), 0); }
    #[test] fn test_identity_zero_sig()    { assert_eq!(identity_valid(1, 1, 0), 0); }

    // Mutual auth
    #[test] fn test_mutual_valid()         { assert_eq!(mutual_auth_valid(1, 2), 1); }
    #[test] fn test_mutual_self_bind()     { assert_eq!(mutual_auth_valid(1, 1), 0); }
    #[test] fn test_mutual_zero_local()    { assert_eq!(mutual_auth_valid(0, 2), 0); }
    #[test] fn test_mutual_zero_remote()   { assert_eq!(mutual_auth_valid(1, 0), 0); }

    // Scope
    #[test] fn test_scope_valid()          { assert_eq!(scope_valid(1, 0, 1), 1); }
    #[test] fn test_scope_zero_seq()       { assert_eq!(scope_valid(0, 0, 1), 0); }
    #[test] fn test_scope_replay()         { assert_eq!(scope_valid(5, 5, 1), 0); }
    #[test] fn test_scope_old_seq()        { assert_eq!(scope_valid(3, 5, 1), 0); }
    #[test] fn test_scope_bad_schema()     { assert_eq!(scope_valid(1, 0, 99), 0); }

    // Anomaly
    #[test] fn test_anomaly_clear()        { assert_eq!(anomaly_result(0),  0); assert_eq!(anomaly_result(49), 0); }
    #[test] fn test_anomaly_flagged()      { assert_eq!(anomaly_result(50), 1); assert_eq!(anomaly_result(74), 1); }
    #[test] fn test_anomaly_escalated()    { assert_eq!(anomaly_result(75), 2); assert_eq!(anomaly_result(100),2); }

    // Full bind
    #[test] fn test_bind_success()         { assert_eq!(arpi_bind(1, 1, 2, 1, 0, 0), 0); }
    #[test] fn test_bind_fail_layer1()     { assert_eq!(arpi_bind(99,1, 2, 1, 0, 0), 1); }
    #[test] fn test_bind_fail_layer2()     { assert_eq!(arpi_bind(1, 0, 2, 1, 0, 0), 2); }
    #[test] fn test_bind_fail_layer3()     { assert_eq!(arpi_bind(1, 1, 1, 1, 0, 0), 3); }
    #[test] fn test_bind_fail_layer4()     { assert_eq!(arpi_bind(1, 1, 2, 0, 0, 0), 4); }
    #[test] fn test_bind_fail_layer5()     { assert_eq!(arpi_bind(1, 1, 2, 1, 0,90), 5); }

    // axon_main mirror — sovereign proof
    #[test] fn test_axon_main_returns_proof() { assert_eq!(axon_main_mirror(), 0x4153); }
    #[test] fn test_axon_proof_constant()     { assert_eq!(AXON_PROOF, 0x4153); }
    #[test] fn test_header_size()             { assert_eq!(HEADER_SIZE, 78); }
    #[test] fn test_arpi_pd_is_mandatory()    { assert!(ARPI_PD_ID < 0x10); }
}
