// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// asl-microkit lib — sovereignty logic for all Microkit PDs.
// ASL v1.5 hybrid: Rust staticlib + C Microkit shims.

#![no_std]

mod dbg;
mod panic;

// Track A — mandatory sovereign PDs
pub mod genesis;
pub mod arpi;

// Track B — driver PDs (ASL-M7)
pub mod input;
pub mod storage;
pub mod usb;
