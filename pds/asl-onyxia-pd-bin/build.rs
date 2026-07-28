// Copyright (c) 2026 Edison Lepiten / AIEONYX
// build.rs — Microkit symbol retention (linker flags via .cargo/config.toml)
fn main() {
    // All linker flags are in .cargo/config.toml
    // This build.rs only handles rerun triggers
    println!("cargo:rerun-if-changed=/home/edisonbl/asl/microkit.ld");
    println!("cargo:rerun-if-changed=src/main.rs");
}
