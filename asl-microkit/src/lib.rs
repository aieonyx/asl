// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// asl-microkit lib — all sovereign PD logic.
// ASL v1.5 hybrid: Rust staticlib + C Microkit shims.

#![no_std]

mod dbg;
mod panic;

// Track A — mandatory sovereign PDs
pub mod genesis;
pub mod arpi;

// Track B — driver PDs
pub mod input;
pub mod storage;
pub mod usb;
pub mod network;

// Track B M9 — AXON-Bridge runtime
pub mod axon_bridge_init;
