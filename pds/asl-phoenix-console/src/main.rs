// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ============================================================
// ASL-M15 — Phoenix Console Protection Domain
// AIEONYX Sovereign Linux · Apache 2.0
// Role: Sovereign console output + AXON shell prompt (stub)
// ============================================================

#![no_std]
#![no_main]

use core::fmt::Write;

const SOVEREIGN_PROOF: u64 = 0x4153;
const CONSOLE_EP_LABEL: u32 = 0x6000;

// ── Console line buffer ────────────────────────────────────
struct ConsolePD {
    line_count: u32,
}

impl ConsolePD {
    const fn new() -> Self { Self { line_count: 0 } }

    fn emit(&mut self, w: &mut impl Write, line: &str) {
        self.line_count += 1;
        let _ = writeln!(w, "[CONSOLE {:04}] {}", self.line_count, line);
    }

    fn prompt(&mut self, w: &mut impl Write) {
        let _ = write!(w, "phoenix@aieonyx:~$ ");
    }
}

// ── IPC message from Phoenix-Init ─────────────────────────
#[repr(C)]
struct AslMsg {
    tag:     u64,
    label:   u32,
    payload: [u64; 4],
}

fn handle_message(msg: &AslMsg, console: &mut ConsolePD, w: &mut impl Write) {
    match msg.label {
        // 0x6000 = CONSOLE_UP signal from Phoenix-Init
        label if label == CONSOLE_EP_LABEL => {
            console.emit(w, "Console PD activated by Phoenix-Init");
            console.emit(w, &format!("Sovereign proof received: {:#x}", msg.payload[0]));
            assert_eq!(msg.payload[0], SOVEREIGN_PROOF,
                       "Sovereign proof mismatch — boot integrity violated");
            console.emit(w, "AXON shell (stub) — M16 will wire AxonScript REPL");
            console.prompt(w);
        }
        other => {
            console.emit(w, &format!("Unknown label {:#x} — dropping", other));
        }
    }
}

#[no_mangle]
pub extern "C" fn phoenix_console_main() -> u64 {
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
    let mut console = ConsolePD::new();

    // Simulate receiving the CONSOLE_UP IPC from Phoenix-Init
    let init_signal = AslMsg {
        tag: 0xA15_0001,
        label: CONSOLE_EP_LABEL,
        payload: [SOVEREIGN_PROOF, 0, 0, 0],
    };
    handle_message(&init_signal, &mut console, &mut w);

    SOVEREIGN_PROOF
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
