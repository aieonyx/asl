// Copyright (c) 2026 Edison Lepiten / AIEONYX

// SPDX-License-Identifier: Apache-2.0

//

// asl-common — shared sovereignty primitives for all ASL PDs.



#![no_std]

#![deny(unsafe_op_in_unsafe_fn)]












#[cfg(kani)]
extern crate kani;

pub mod arpi;

pub mod datatier;

pub mod pd;

pub mod version;

