// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Panic handler for Microkit PDs.

use core::sync::atomic::{AtomicBool, Ordering};

static PANICKED: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    PANICKED.store(true, Ordering::SeqCst);
    loop {
        core::hint::spin_loop();
    }
}
