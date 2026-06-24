// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// @constant_time contract enforcement at the AXON-Bridge PD boundary.
//
// The AXON compiler's P55.7 codegen emits a timing contract token
// for functions annotated with @constant_time. The bridge verifies
// this token before allowing the function to execute in a PD context.
//
// Timing side-channels are eliminated at compile time (P55.7).
// The bridge provides runtime double-check that the contract token
// is present — belt-and-suspenders enforcement.
//
// Contract token format: 0xC0C0_FFFF_<fn_hash_16>_<seq_16>
//   C0C0 = constant_time prefix
//   FFFF = all-ones enforcement marker
//   fn_hash_16 = lower 16 bits of function name hash
//   seq_16 = monotonic sequence

/// Constant-time contract token prefix.
pub const CT_TOKEN_PREFIX: u32 = 0xC0C0_FFFF;

/// Constant-time enforcement result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtResult {
    /// Contract valid — function may execute.
    Valid,
    /// No contract token — function lacks @constant_time annotation.
    MissingContract,
    /// Token prefix invalid.
    InvalidPrefix,
    /// Token sequence is zero (stub/uninitialized).
    ZeroSequence,
}

/// Validates a @constant_time contract token.
pub fn validate_ct_token(token: u64) -> CtResult {
    if token == 0 {
        return CtResult::MissingContract;
    }
    let prefix = (token >> 32) as u32;
    if prefix != CT_TOKEN_PREFIX {
        return CtResult::InvalidPrefix;
    }
    let seq = (token & 0xFFFF) as u16;
    if seq == 0 {
        return CtResult::ZeroSequence;
    }
    CtResult::Valid
}

/// Generates a @constant_time contract token for a function.
/// fn_name_hash: lower 16 bits of FNV-1a hash of function name.
/// seq: monotonic sequence from the AXON compiler.
pub fn make_ct_token(fn_name_hash: u16, seq: u16) -> u64 {
    if seq == 0 { return 0; }
    ((CT_TOKEN_PREFIX as u64) << 32)
        | ((fn_name_hash as u64) << 16)
        | (seq as u64)
}

/// Registry of @constant_time contracts for loaded AXON functions.
pub struct CtRegistry {
    entries: [(u16, u64); 32], // (fn_hash, token)
    count:   usize,
}

impl CtRegistry {
    pub const fn new() -> Self {
        Self { entries: [(0, 0); 32], count: 0 }
    }

    /// Register a @constant_time contract for a function.
    pub fn register(&mut self, fn_hash: u16, token: u64) -> Result<(), CtError> {
        if self.count >= 32 {
            return Err(CtError::RegistryFull);
        }
        match validate_ct_token(token) {
            CtResult::Valid => {}
            CtResult::MissingContract => return Err(CtError::MissingContract),
            CtResult::InvalidPrefix   => return Err(CtError::InvalidToken),
            CtResult::ZeroSequence    => return Err(CtError::InvalidToken),
        }
        self.entries[self.count] = (fn_hash, token);
        self.count += 1;
        Ok(())
    }

    /// Verify a function has a valid @constant_time contract.
    pub fn verify(&self, fn_hash: u16) -> CtResult {
        match self.entries[..self.count].iter().find(|(h, _)| *h == fn_hash) {
            None => CtResult::MissingContract,
            Some((_, token)) => validate_ct_token(*token),
        }
    }

    pub fn registered_count(&self) -> usize { self.count }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtError {
    RegistryFull,
    MissingContract,
    InvalidToken,
}
