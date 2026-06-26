// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ============================================================
// ASL-M16 — REPL IPC Protocol
// asl-common/src/repl_ipc.rs
// AIEONYX Sovereign Linux · Apache 2.0
// Shared between Phoenix-Console and AXON-Bridge PDs
// ============================================================

/// IPC label: Phoenix-Console → AXON-Bridge (eval request)
pub const REPL_EVAL_LABEL:   u32 = 0x8000;
/// IPC label: AXON-Bridge → Phoenix-Console (eval result)
pub const REPL_RESULT_LABEL: u32 = 0x8001;
/// IPC label: REPL session control (exit / reset)
pub const REPL_CTRL_LABEL:   u32 = 0x8002;

/// Maximum expression length the REPL accepts (bytes)
pub const REPL_EXPR_MAX: usize = 128;
/// Maximum result string length returned (bytes)
pub const REPL_RESULT_MAX: usize = 128;

/// REPL eval request — sent from Phoenix-Console to AXON-Bridge
#[repr(C)]
pub struct ReplEvalRequest {
    pub tag:     u64,           // 0xA16_0001
    pub label:   u32,           // REPL_EVAL_LABEL
    pub seq:     u32,           // monotonic sequence number
    pub proof:   u64,           // sovereign proof 0x4153
    pub len:     u32,           // expression byte length
    pub expr:    [u8; REPL_EXPR_MAX], // expression bytes (UTF-8)
}

/// REPL eval result — returned from AXON-Bridge to Phoenix-Console
#[repr(C)]
pub struct ReplEvalResult {
    pub tag:     u64,           // 0xA16_0002
    pub label:   u32,           // REPL_RESULT_LABEL
    pub seq:     u32,           // mirrors request seq
    pub status:  ReplStatus,    // Ok / Err / Sovereign
    pub len:     u32,           // result byte length
    pub result:  [u8; REPL_RESULT_MAX], // result bytes (UTF-8)
}

/// REPL evaluation status
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplStatus {
    Ok       = 0x00, // evaluation succeeded
    Err      = 0x01, // syntax or evaluation error
    Sovereign = 0x02, // sovereign builtin result
    Ctrl     = 0x03, // control command (exit/help)
}

impl ReplEvalRequest {
    pub fn new(seq: u32, expr: &[u8]) -> Self {
        let mut req = Self {
            tag:   0xA16_0001,
            label: REPL_EVAL_LABEL,
            seq,
            proof: 0x4153,
            len:   expr.len().min(REPL_EXPR_MAX) as u32,
            expr:  [0u8; REPL_EXPR_MAX],
        };
        let n = expr.len().min(REPL_EXPR_MAX);
        req.expr[..n].copy_from_slice(&expr[..n]);
        req
    }

    /// Verify sovereign proof is intact before processing
    pub fn verify_proof(&self) -> bool {
        self.proof == 0x4153
            && self.label == REPL_EVAL_LABEL
            && self.len <= REPL_EXPR_MAX as u32
    }
}

impl ReplEvalResult {
    pub fn ok(seq: u32, val: &[u8]) -> Self {
        Self::build(seq, ReplStatus::Ok, val)
    }

    pub fn err(seq: u32, msg: &[u8]) -> Self {
        Self::build(seq, ReplStatus::Err, msg)
    }

    pub fn sovereign(seq: u32, val: &[u8]) -> Self {
        Self::build(seq, ReplStatus::Sovereign, val)
    }

    fn build(seq: u32, status: ReplStatus, data: &[u8]) -> Self {
        let mut res = Self {
            tag:    0xA16_0002,
            label:  REPL_RESULT_LABEL,
            seq,
            status,
            len:    data.len().min(REPL_RESULT_MAX) as u32,
            result: [0u8; REPL_RESULT_MAX],
        };
        let n = data.len().min(REPL_RESULT_MAX);
        res.result[..n].copy_from_slice(&data[..n]);
        res
    }
}
