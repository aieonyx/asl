// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Build script — tells Cargo where to find libmicrokit.a

fn main() {
    let sdk = std::env::var("MICROKIT_SDK")
        .unwrap_or_else(|_| format!("{}//microkit-sdk-1.4.1",
            std::env::var("HOME").unwrap_or_default()));
    let lib_path = format!(
        "{}/board/qemu_virt_aarch64/debug/lib", sdk
    );
    println!("cargo:rustc-link-search=native={}", lib_path);
    println!("cargo:rustc-link-lib=static=microkit");
}
