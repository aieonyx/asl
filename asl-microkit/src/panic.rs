// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// panic.rs — Sovereign panic handler for seL4 no_std PD
// Excluded during test builds (std provides its own panic handler)

#[cfg(all(not(kani), not(test)))]
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(all(not(kani), not(test)))]
pub static PANICKED: AtomicBool = AtomicBool::new(false);

#[cfg(all(not(kani), not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    PANICKED.store(true, Ordering::SeqCst);
    loop {
        core::hint::spin_loop();
    }
}
