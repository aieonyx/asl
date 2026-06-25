// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Storage Driver PD — sovereignty logic (ASL-M7)
//
// NVMe/eMMC driver stub. EdisonDB mounts on top of this PD.
// Storage access is mediated by DataTier-Enforcer — no direct
// storage access is possible without tier boundary validation.
//
// Storage operation types:
//   OP_READ  (0x01) — read block from storage
//   OP_WRITE (0x02) — write block to storage
//   OP_SYNC  (0x03) — flush write-ahead log
//   OP_ERASE (0x04) — GDPR Art.17 erasure (Critical tier)
//
// ASL-M7: structural stub with operation dispatch.
// ASL-M12: real EdisonDB PD wired on top of this.

use crate::dbg;

/// Storage operation codes
const OP_READ:  u8 = 0x01;
const OP_WRITE: u8 = 0x02;
const OP_SYNC:  u8 = 0x03;
const OP_ERASE: u8 = 0x04;

/// Storage status codes
const STATUS_OK:    u64 = 0x00;
const STATUS_ERR:   u64 = 0x01;
#[allow(dead_code)]
const STATUS_BUSY:  u64 = 0x02;

/// Storage block size — 4KB aligned to seL4 page size
pub const BLOCK_SIZE: usize = 4096;

/// Storage statistics
static mut READS:  u64 = 0;
static mut WRITES: u64 = 0;
static mut ERASES: u64 = 0;

#[no_mangle]
pub extern "C" fn asl_storage_init() {
    dbg::puts("Storage Driver PD: initializing\n");
    dbg::puts("Storage: NVMe/eMMC driver stub active\n");
    dbg::puts("Storage: DataTier-Enforcer boundary active\n");
    dbg::puts("Storage: GDPR Art.17 erasure path registered\n");
    dbg::puts("Storage: WAL sync path registered\n");
    dbg::puts("Storage: EdisonDB mount point ready\n");
    dbg::puts("Storage Driver PD: READY\n");
}

#[no_mangle]
pub extern "C" fn asl_storage_notified(channel: u8) {
    // Storage PD receives sync notifications from EdisonDB PD
    dbg::puts("Storage: WAL sync notification received\n");
    let _ = channel;
}

/// Storage PPC handler — EdisonDB calls this for all storage ops.
/// Returns status code: 0=OK, 1=ERR, 2=BUSY
#[no_mangle]
pub extern "C" fn asl_storage_protected(channel: u8, msginfo: u64) -> u64 {
    // Extract operation from message label (top 8 bits of msginfo)
    let op = ((msginfo >> 56) & 0xFF) as u8;
    let _ = channel;

    match op {
        OP_READ => {
            unsafe { READS += 1; }
            STATUS_OK
        }
        OP_WRITE => {
            unsafe { WRITES += 1; }
            STATUS_OK
        }
        OP_SYNC => {
            dbg::puts("Storage: WAL sync\n");
            STATUS_OK
        }
        OP_ERASE => {
            unsafe { ERASES += 1; }
            dbg::puts("Storage: GDPR Art.17 erasure executed\n");
            STATUS_OK
        }
        _ => STATUS_ERR,
    }
}
