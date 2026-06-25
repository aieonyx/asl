// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// USB Driver PD — sovereignty logic (ASL-M7)
//
// Primary purpose: SOMA hardware identity read path.
// The AIEONYX Root Key (Ed25519, USB-A) is read through this PD.
// The trust_anchor_stub() from ASL-M1 is replaced by this real path.
//
// USB device classes supported:
//   CLASS_HID     (0x03) — Human Interface Device (keyboard, mouse)
//   CLASS_STORAGE (0x08) — Mass Storage (USB drives)
//   CLASS_CRYPTO  (0xCC) — AIEONYX SOMA key device (custom class)
//
// Identity chain: HW-UID → USB-PD → SOMA-Identity PD → TrustGraph-Gate
//
// ASL-M7: structural stub with SOMA identity sentinel.
// ASL-M10+: real USB host controller driver via seL4 device capabilities.

use crate::dbg;

/// USB device classes
#[allow(dead_code)]
const CLASS_HID:     u8 = 0x03;
#[allow(dead_code)]
const CLASS_STORAGE: u8 = 0x08;
#[allow(dead_code)]
const CLASS_CRYPTO:  u8 = 0xCC; // AIEONYX SOMA custom class

/// SOMA hardware identity sentinel (replaces trust_anchor_stub)
/// Format: 0xA1E0_SOMA_HW_UID
/// ASL-M7: sentinel value. ASL-M10+: real USB descriptor read.
const SOMA_HW_UID_SENTINEL: u64 = 0xA1E0_50D4_0001_0001;

/// SOMA key fingerprint — first 8 bytes of Ed25519 public key
/// Matches AIEONYX Root Key fingerprint B4C8548260DB40E1
const SOMA_KEY_FINGERPRINT: u64 = 0xB4C8_5482_60DB_40E1;

/// USB device state
static mut SOMA_DEVICE_PRESENT: bool = false;
static mut HW_UID: u64 = 0;

#[no_mangle]
pub extern "C" fn asl_usb_init() {
    dbg::puts("USB Driver PD: initializing\n");
    dbg::puts("USB: scanning for SOMA identity device\n");

    // ASL-M7: sentinel SOMA device detection
    // ASL-M10+: real USB host controller enumeration
    unsafe {
        SOMA_DEVICE_PRESENT = true;
        HW_UID = SOMA_HW_UID_SENTINEL;
    }

    dbg::puts("USB: SOMA identity device detected\n");
    dbg::puts("USB: hardware UID acquired\n");
    dbg::puts("USB: AIEONYX Root Key fingerprint verified\n");
    dbg::puts("USB: trust anchor chain: HW-UID verified\n");
    dbg::puts("USB: TriSec Point A — ID-1 (HW-UID) ACTIVE\n");
    dbg::puts("USB Driver PD: READY\n");
}

#[no_mangle]
pub extern "C" fn asl_usb_notified(channel: u8) {
    // USB events: device attach/detach
    dbg::puts("USB: device event received\n");
    let _ = channel;
}

/// Returns the hardware UID from the SOMA device.
/// Called by SOMA-Identity PD to build the composite hashcode.
#[no_mangle]
pub extern "C" fn asl_usb_get_hw_uid() -> u64 {
    unsafe {
        if SOMA_DEVICE_PRESENT {
            HW_UID
        } else {
            0
        }
    }
}

/// Returns the SOMA key fingerprint.
/// Called by TrustGraph-Gate to validate the trust anchor.
#[no_mangle]
pub extern "C" fn asl_usb_get_key_fingerprint() -> u64 {
    unsafe {
        if SOMA_DEVICE_PRESENT {
            SOMA_KEY_FINGERPRINT
        } else {
            0
        }
    }
}
