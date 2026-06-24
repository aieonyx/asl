// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Panic handler for GENESIS root task.
// In seL4 bare-metal: panic = system halt.
// No unwinding. No recovery. Halt is the safe state.

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // In a full seL4 deployment this would:
    // 1. Write panic info to GENESIS audit log
    // 2. Signal all PDs to enter safe shutdown
    // 3. Call seL4_TCB_Suspend on all non-GENESIS threads
    // For ASL-M1 QEMU: spin loop (QEMU test harness detects non-exit)
    loop {
        core::hint::spin_loop();
    }
}
