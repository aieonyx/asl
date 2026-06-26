// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ============================================================
// ASL-M16 — AXON-REPL Protection Domain
// AIEONYX Sovereign Linux · Apache 2.0
// Role: AxonScript expression evaluator for Phoenix-Console
// Receives: ReplEvalRequest via IPC from Phoenix-Console
// Returns:  ReplEvalResult via IPC reply
// ============================================================

#![no_std]
#![no_main]

#[cfg(kani)]
extern crate kani;

use core::fmt::Write;

const SOVEREIGN_PROOF: u64 = 0x4153;
const ASL_VERSION: &str    = "v1.0";
const SEL4_VERSION: &str   = "15.0.0";
const PD_COUNT: usize      = 10;

const REPL_EVAL_LABEL:   u32 = 0x8000;
const REPL_RESULT_LABEL: u32 = 0x8001;
const REPL_EXPR_MAX: usize   = 128;
const REPL_RESULT_MAX: usize = 128;

// ── Value types the REPL can return ───────────────────────
#[derive(Clone, Copy)]
enum ReplVal {
    Int(i64),
    Str(&'static str),
    Sovereign,
    Err,
}

// ── Sovereign builtins ─────────────────────────────────────
fn eval_builtin(expr: &[u8]) -> Option<ReplVal> {
    match expr {
        b"sovereign()" => Some(ReplVal::Sovereign),
        b"pd_count()"  => Some(ReplVal::Int(PD_COUNT as i64)),
        b"version()"   => Some(ReplVal::Str("ASL v1.0 [seL4 15.0.0]")),
        b"help"        => Some(ReplVal::Str(
            "sovereign() pd_count() version() let x=<expr> help exit"
        )),
        b"exit"        => Some(ReplVal::Str("__EXIT__")),
        _              => None,
    }
}

// ── Minimal i64 arithmetic evaluator ──────────────────────
// Supports: integer literals, +, -, *, / (no parens in M16)
fn eval_arithmetic(expr: &[u8]) -> Option<i64> {
    // Trim leading/trailing whitespace
    let s = trim(expr);
    if s.is_empty() { return None; }

    // Try pure integer literal first
    if let Some(n) = parse_i64(s) { return Some(n); }

    // Try binary op: find last + or - (lowest precedence), then * /
    for op in [b'+', b'-', b'*', b'/'] {
        if let Some(pos) = rfind(s, op) {
            if pos == 0 { continue; } // unary minus not supported in M16
            let lhs = eval_arithmetic(&s[..pos])?;
            let rhs = eval_arithmetic(&s[pos+1..])?;
            return match op {
                b'+' => Some(lhs.wrapping_add(rhs)),
                b'-' => Some(lhs.wrapping_sub(rhs)),
                b'*' => Some(lhs.wrapping_mul(rhs)),
                b'/' => if rhs != 0 { Some(lhs / rhs) } else { None },
                _    => None,
            };
        }
    }
    None
}

// ── let binding (session scope — single slot in M16) ──────
static mut LET_NAME: [u8; 32]  = [0u8; 32];
static mut LET_NAME_LEN: usize = 0;
static mut LET_VAL: i64        = 0;

fn eval_let(expr: &[u8]) -> Option<ReplVal> {
    // Pattern: "let <name> = <expr>"
    let s = trim(expr);
    if !starts_with(s, b"let ") { return None; }
    let rest = &s[4..];
    let eq = find(rest, b'=')?;
    let name = trim(&rest[..eq]);
    let val_expr = trim(&rest[eq+1..]);
    let val = eval_arithmetic(val_expr)?;
    unsafe {
        let n = name.len().min(31);
        LET_NAME[..n].copy_from_slice(&name[..n]);
        LET_NAME_LEN = n;
        LET_VAL = val;
    }
    Some(ReplVal::Int(val))
}

fn lookup_name(name: &[u8]) -> Option<i64> {
    unsafe {
        if LET_NAME_LEN > 0 && name == &LET_NAME[..LET_NAME_LEN] {
            Some(LET_VAL)
        } else {
            None
        }
    }
}

// ── Top-level evaluator ────────────────────────────────────
fn evaluate(expr: &[u8]) -> ReplVal {
    let s = trim(expr);
    if s.is_empty() { return ReplVal::Str(""); }

    // Builtins
    if let Some(v) = eval_builtin(s) { return v; }

    // Let binding
    if starts_with(s, b"let ") {
        return eval_let(s).unwrap_or(ReplVal::Err);
    }

    // Name lookup
    if let Some(v) = lookup_name(s) { return ReplVal::Int(v); }

    // Arithmetic
    if let Some(n) = eval_arithmetic(s) { return ReplVal::Int(n); }

    ReplVal::Err
}

// ── Format result into output buffer ──────────────────────
fn format_result(val: ReplVal, buf: &mut [u8]) -> usize {
    match val {
        ReplVal::Sovereign => {
            let s = b"axon_main() -> 0x4153";
            let n = s.len().min(buf.len());
            buf[..n].copy_from_slice(&s[..n]);
            n
        }
        ReplVal::Int(n) => {
            // Simple i64 → decimal ASCII
            let mut tmp = [0u8; 22];
            let len = fmt_i64(n, &mut tmp);
            let n2 = len.min(buf.len());
            buf[..n2].copy_from_slice(&tmp[..n2]);
            n2
        }
        ReplVal::Str(s) => {
            let b = s.as_bytes();
            let n = b.len().min(buf.len());
            buf[..n].copy_from_slice(&b[..n]);
            n
        }
        ReplVal::Err => {
            let s = b"error: unknown expression";
            let n = s.len().min(buf.len());
            buf[..n].copy_from_slice(&s[..n]);
            n
        }
    }
}

// ── IPC message handler ────────────────────────────────────
#[repr(C)]
struct ReplRequest {
    tag:   u64,
    label: u32,
    seq:   u32,
    proof: u64,
    len:   u32,
    expr:  [u8; REPL_EXPR_MAX],
}

fn handle_request(req: &ReplRequest, w: &mut impl Write) -> bool {
    // Verify sovereign proof
    if req.proof != SOVEREIGN_PROOF || req.label != REPL_EVAL_LABEL {
        let _ = writeln!(w, "[REPL] FATAL: proof mismatch — dropping request");
        return false;
    }
    let len = (req.len as usize).min(REPL_EXPR_MAX);
    let expr = &req.expr[..len];
    let val = evaluate(expr);

    // Check for exit
    if let ReplVal::Str("__EXIT__") = val {
        let _ = writeln!(w, "[REPL] exit requested — Phoenix-Console halting");
        return false; // signal caller to stop
    }

    let mut result_buf = [0u8; REPL_RESULT_MAX];
    let result_len = format_result(val, &mut result_buf);
    let result_str = core::str::from_utf8(&result_buf[..result_len])
        .unwrap_or("?");
    let _ = writeln!(w, "{}", result_str);
    true
}

// ── PD entry point ─────────────────────────────────────────
#[no_mangle]
pub extern "C" fn axon_repl_main() -> u64 {
    struct SerialWriter;
    impl Write for SerialWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for b in s.bytes() {
                unsafe { core::ptr::write_volatile(0x0900_0000 as *mut u8, b); }
            }
            Ok(())
        }
    }
    let mut w = SerialWriter;
    let _ = writeln!(w, "[AXON-REPL] PD online — AxonScript evaluator ready");

    // Simulate processing a set of demo expressions for QEMU boot log
    let demo: &[&[u8]] = &[
        b"sovereign()",
        b"version()",
        b"pd_count()",
        b"let x = 21 + 21",
        b"x",
        b"1 + 2 * 3",
        b"help",
    ];

    for (i, expr) in demo.iter().enumerate() {
        let req = ReplRequest {
            tag:   0xA16_0001,
            label: REPL_EVAL_LABEL,
            seq:   i as u32,
            proof: SOVEREIGN_PROOF,
            len:   expr.len() as u32,
            expr:  {
                let mut buf = [0u8; REPL_EXPR_MAX];
                let n = expr.len().min(REPL_EXPR_MAX);
                buf[..n].copy_from_slice(&expr[..n]);
                buf
            },
        };
        let _ = write!(w, "phoenix@aieonyx:~$ ");
        // Echo expression
        let s = core::str::from_utf8(*expr).unwrap_or("?");
        let _ = writeln!(w, "{}", s);
        handle_request(&req, &mut w);
    }

    SOVEREIGN_PROOF
}

// ── Helper functions (no_std compatible) ──────────────────
fn trim(s: &[u8]) -> &[u8] {
    let start = s.iter().position(|&b| b != b' ' && b != b'\t').unwrap_or(s.len());
    let end   = s.iter().rposition(|&b| b != b' ' && b != b'\t').map(|i| i+1).unwrap_or(0);
    if start >= end { &[] } else { &s[start..end] }
}

fn parse_i64(s: &[u8]) -> Option<i64> {
    if s.is_empty() { return None; }
    let (neg, digits) = if s[0] == b'-' { (true, &s[1..]) } else { (false, s) };
    if digits.is_empty() { return None; }
    let mut n: i64 = 0;
    for &b in digits {
        if b < b'0' || b > b'9' { return None; }
        n = n.checked_mul(10)?.checked_add((b - b'0') as i64)?;
    }
    Some(if neg { -n } else { n })
}

fn rfind(s: &[u8], c: u8) -> Option<usize> {
    s.iter().rposition(|&b| b == c)
}

fn find(s: &[u8], c: u8) -> Option<usize> {
    s.iter().position(|&b| b == c)
}

fn starts_with(s: &[u8], prefix: &[u8]) -> bool {
    s.len() >= prefix.len() && &s[..prefix.len()] == prefix
}

fn fmt_i64(mut n: i64, buf: &mut [u8; 22]) -> usize {
    if n == 0 { buf[0] = b'0'; return 1; }
    let neg = n < 0;
    if neg { n = -n; }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    let mut out = 0;
    if neg { buf[out] = b'-'; out += 1; }
    for j in (0..i).rev() { buf[out] = tmp[j]; out += 1; }
    out
}

#[cfg(not(kani))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
