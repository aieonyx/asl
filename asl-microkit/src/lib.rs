// Copyright (c) 2026 Edison Lepiten / AIEONYX

// SPDX-License-Identifier: Apache-2.0

//

// asl-microkit lib — all sovereign PD logic.

// ASL v1.5 hybrid: Rust staticlib + C Microkit shims.



#![no_std]












#[cfg(kani)]
extern crate kani;

mod dbg;

mod panic;



pub mod genesis;

pub mod arpi;

pub mod input;

pub mod storage;

pub mod usb;

pub mod network;

pub mod axon_bridge_init;

pub mod mcs;

pub mod wcet;

pub mod haniel;

pub mod edisondb;

pub mod onyxia;

