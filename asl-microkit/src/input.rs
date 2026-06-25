// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Input Driver PD — sovereignty logic (ASL-M7)
//
// Handles keyboard/mouse/touchpad events as Microkit notifications.
// Each input device is a separate channel — isolated from other PDs.
// A crash in the input PD cannot affect any other sovereign PD.
//
// ASL-M7: structural stub with event counting.
// ASL-M10+: real interrupt handler wired via seL4 IRQ capabilities.

use crate::dbg;

/// Input channel assignments
const CH_KEYBOARD:  u8 = 1;
const CH_MOUSE:     u8 = 2;
const CH_TOUCHPAD:  u8 = 3;

/// Input event counter — monotonic, per device
static mut KEYBOARD_EVENTS: u64 = 0;
static mut MOUSE_EVENTS:    u64 = 0;
static mut TOUCHPAD_EVENTS: u64 = 0;

#[no_mangle]
pub extern "C" fn asl_input_init() {
    dbg::puts("Input Driver PD: initializing\n");
    dbg::puts("Input: keyboard channel registered\n");
    dbg::puts("Input: mouse channel registered\n");
    dbg::puts("Input: touchpad channel registered\n");
    dbg::puts("Input: S4+i isolation enforced — crash cannot escape PD\n");
    dbg::puts("Input Driver PD: READY\n");
}

#[no_mangle]
pub extern "C" fn asl_input_notified(channel: u8) {
    match channel {
        CH_KEYBOARD => {
            unsafe { KEYBOARD_EVENTS += 1; }
            dbg::puts("Input: keyboard event\n");
        }
        CH_MOUSE => {
            unsafe { MOUSE_EVENTS += 1; }
            dbg::puts("Input: mouse event\n");
        }
        CH_TOUCHPAD => {
            unsafe { TOUCHPAD_EVENTS += 1; }
            dbg::puts("Input: touchpad event\n");
        }
        _ => {
            dbg::puts("Input: unknown channel — ignored\n");
        }
    }
}
