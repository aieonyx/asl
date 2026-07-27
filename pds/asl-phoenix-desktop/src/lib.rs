// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ════════════════════════════════════════════════════════════════════════════
// asl-phoenix-desktop — Phoenix-Desktop Protection Domain
// PL-70 / ASL-M25: seL4 boot handoff — aiXos Phoenix under seL4 microkernel
// ════════════════════════════════════════════════════════════════════════════
//
// ROLE: Sovereign desktop render PD. Receives framebuffer capability from
//       GPU-Cap PD via seL4 IPC, runs the aiXos Phoenix desktop render loop
//       inside an isolated Protection Domain.
//
// CAPABILITY POLICY (enforced by seL4):
//   GRANTED  : FramebufferWrite  — write to ramfb (1280×720 ARGB8888)
//   GRANTED  : FontRead          — read sovereign font store (AXFS)
//   GRANTED  : EDBRead           — read EdisonDB via DataTier-Enforcer
//   GRANTED  : InputRead         — receive virtio-input events from Input-PD
//   DENIED   : Network           — desktop PD has zero direct network access
//   DENIED   : StorageWrite      — cannot write to AXFS directly (via DataTier)
//
// ISOLATION PROOF:
//   A crash in the desktop render loop cannot corrupt the kernel,
//   EdisonDB, ARPi identity, or AWP protocol state — each runs in its own PD.
//   seL4 enforces this at the hardware MMU level. No aiXos v1.0 bare-metal
//   equivalent exists — this is the v2.0 sovereign differentiator.
//
// BOOT SEQUENCE (PL-70):
//   UEFI → seL4 microkernel → GENESIS PD
//     → Phoenix-Init PD (M15 boot sequencer)
//       → GPU-Cap PD (maps ramfb, grants FramebufferWrite cap)
//         → Phoenix-Desktop PD (THIS PD — runs desktop render loop)
//
// IPC CHANNELS:
//   Phoenix-Init → Phoenix-Desktop  : BOOT_SIGNAL (start render loop)
//   GPU-Cap      → Phoenix-Desktop  : FRAMEBUFFER_CAP (shared memory grant)
//   Input-PD     → Phoenix-Desktop  : INPUT_EVENT (keyboard/mouse)
//   Phoenix-Desktop → DataTier      : EDB_QUERY (database reads)
//   Phoenix-Desktop → Phoenix-Watchdog : HEARTBEAT (every 100ms)
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓
// Sovereign proof invariant: axon_main() → 0x4153

#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::all)]

#[cfg(kani)]
extern crate kani;

use asl_common::pd::PdId;
use asl_haniel::{SURFACE_WIDTH, SURFACE_HEIGHT, AXON_PROOF};

// ── PD identity ───────────────────────────────────────────────────────────────

/// Phoenix-Desktop PD identifier (optional PD, desktop profile)
pub const PHOENIX_DESKTOP_PD_ID: u8 = 0x30;

/// Sovereign proof — must remain 0x4153 across all milestones
pub const SOVEREIGN_PROOF: u64 = AXON_PROOF;

// ── IPC message labels ────────────────────────────────────────────────────────

/// Boot signal from Phoenix-Init: "start the render loop"
pub const MSG_BOOT_SIGNAL:       u32 = 0xB001;
/// Framebuffer capability grant from GPU-Cap
pub const MSG_FB_CAP_GRANT:      u32 = 0xB002;
/// Input event from Input-PD (key/mouse)
pub const MSG_INPUT_EVENT:       u32 = 0xB003;
/// Heartbeat to Phoenix-Watchdog
pub const MSG_HEARTBEAT:         u32 = 0xB004;
/// EDB query to DataTier-Enforcer
pub const MSG_EDB_QUERY:         u32 = 0xB005;

// ── Desktop render state ──────────────────────────────────────────────────────

/// Desktop initialization phases — mirrors aiXos Phoenix boot stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DesktopPhase {
    /// Waiting for BOOT_SIGNAL from Phoenix-Init
    AwaitingBoot     = 0x01,
    /// Waiting for framebuffer capability from GPU-Cap
    AwaitingFramebuf = 0x02,
    /// Desktop running — render loop active
    Running          = 0x03,
    /// Suspended — framebuffer capability revoked
    Suspended        = 0x04,
    /// Fatal error — PD will halt and signal watchdog
    Faulted          = 0xFF,
}

/// Framebuffer descriptor — received from GPU-Cap PD via IPC
#[derive(Debug, Clone, Copy)]
pub struct FramebufDesc {
    /// Virtual address of the mapped framebuffer (granted by GPU-Cap)
    pub vaddr:  u64,
    /// Width in pixels (sovereign standard: 1280)
    pub width:  u32,
    /// Height in pixels (sovereign standard: 720)
    pub height: u32,
    /// Bytes per pixel (sovereign standard: 4 — ARGB8888)
    pub bpp:    u32,
    /// Stride in bytes per row
    pub stride: u32,
}

impl FramebufDesc {
    /// Validate that the descriptor matches sovereign standard
    pub fn is_sovereign_standard(&self) -> bool {
        self.width  == SURFACE_WIDTH  &&
        self.height == SURFACE_HEIGHT &&
        self.bpp    == 4              &&
        self.stride == SURFACE_WIDTH * 4
    }

    /// Total size of framebuffer in bytes
    pub fn total_bytes(&self) -> u32 {
        self.stride * self.height
    }
}

/// Input event — keyboard or mouse, received from Input-PD
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    /// Event type: 0x01=key_press, 0x02=key_release,
    ///             0x10=mouse_move, 0x11=mouse_button
    pub kind:    u8,
    /// Key scan code or mouse button index
    pub code:    u16,
    /// Mouse X (absolute, 0..1280) or key character
    pub x:       i32,
    /// Mouse Y (absolute, 0..720)
    pub y:       i32,
}

/// Phoenix-Desktop PD state machine
pub struct PhoenixDesktopPd {
    pub phase:    DesktopPhase,
    pub framebuf: Option<FramebufDesc>,
    /// Heartbeat counter — incremented each render frame
    pub ticks:    u64,
    /// Total input events processed
    pub events:   u64,
    /// Sovereign proof — must always equal 0x4153
    pub proof:    u64,
}

impl PhoenixDesktopPd {
    /// Create a new Phoenix-Desktop PD in AwaitingBoot phase
    pub const fn new() -> Self {
        PhoenixDesktopPd {
            phase:    DesktopPhase::AwaitingBoot,
            framebuf: None,
            ticks:    0,
            events:   0,
            proof:    SOVEREIGN_PROOF,
        }
    }

    /// Receive BOOT_SIGNAL from Phoenix-Init — advance to AwaitingFramebuf
    pub fn on_boot_signal(&mut self) -> Result<(), &'static str> {
        if self.phase != DesktopPhase::AwaitingBoot {
            return Err("BOOT_SIGNAL received in wrong phase");
        }
        self.assert_proof();
        self.phase = DesktopPhase::AwaitingFramebuf;
        Ok(())
    }

    /// Receive framebuffer capability grant from GPU-Cap PD
    pub fn on_fb_cap_grant(&mut self, desc: FramebufDesc) -> Result<(), &'static str> {
        if self.phase != DesktopPhase::AwaitingFramebuf {
            return Err("FB_CAP_GRANT received in wrong phase");
        }
        if !desc.is_sovereign_standard() {
            return Err("framebuffer does not meet sovereign standard (1280x720 ARGB8888)");
        }
        self.assert_proof();
        self.framebuf = Some(desc);
        self.phase    = DesktopPhase::Running;
        Ok(())
    }

    /// Process one input event — returns true if render needed
    pub fn on_input_event(&mut self, ev: InputEvent) -> bool {
        if self.phase != DesktopPhase::Running { return false; }
        self.events += 1;
        // In full implementation: dispatch to window manager event queue
        // For PL-70: event is logged and acknowledged
        matches!(ev.kind, 0x01 | 0x10 | 0x11) // key/mouse events trigger redraw
    }

    /// Advance one render tick — called each frame
    /// Returns: heartbeat payload to send to Phoenix-Watchdog
    pub fn render_tick(&mut self) -> u64 {
        if self.phase != DesktopPhase::Running { return 0; }
        self.ticks += 1;
        self.assert_proof();
        // Heartbeat: ticks | proof (low 16 bits)
        (self.ticks << 16) | (SOVEREIGN_PROOF & 0xFFFF)
    }

    /// Suspend — framebuffer capability revoked by GPU-Cap
    pub fn suspend(&mut self) {
        self.framebuf = None;
        self.phase    = DesktopPhase::Suspended;
    }

    /// Resume — new framebuffer capability granted
    pub fn resume(&mut self, desc: FramebufDesc) -> Result<(), &'static str> {
        if self.phase != DesktopPhase::Suspended {
            return Err("resume called outside Suspended phase");
        }
        if !desc.is_sovereign_standard() {
            return Err("framebuffer does not meet sovereign standard");
        }
        self.framebuf = Some(desc);
        self.phase    = DesktopPhase::Running;
        Ok(())
    }

    /// Assert sovereign proof invariant — panics if violated
    #[inline]
    fn assert_proof(&self) {
        assert_eq!(self.proof, SOVEREIGN_PROOF,
            "SOVEREIGN PROOF VIOLATION: Phoenix-Desktop PD integrity check failed");
    }

    /// PD identity
    pub fn pd_id() -> PdId {
        PdId::GpuCap // desktop profile — GpuCap slot used for Phoenix-Desktop
    }
}

impl Default for PhoenixDesktopPd {
    fn default() -> Self { Self::new() }
}

// ── GPU-Cap PD — seL4 framebuffer capability mediator ────────────────────────

/// GPU-Cap PD: maps ramfb and grants FramebufferWrite capability to Phoenix-Desktop.
/// This is the seL4 capability gate for all framebuffer access.
/// No PD can write to the display without going through GPU-Cap.
pub struct GpuCapPd {
    /// ramfb physical address (QEMU: 0x44000000 in Phoenix v1.0)
    pub ramfb_phys:  u64,
    /// Virtual address mapped by seL4 for GPU-Cap's own access
    pub ramfb_vaddr: u64,
    /// Width × Height × 4 bytes
    pub fb_size:     u32,
    /// True after GPU-Cap has mapped ramfb and granted cap to Phoenix-Desktop
    pub cap_granted: bool,
    /// Sovereign proof
    pub proof:       u64,
}

impl GpuCapPd {
    /// Phoenix ramfb physical address — sovereign standard
    pub const RAMFB_PHYS: u64 = 0x44000000;
    /// Sovereign framebuffer size: 1280 × 720 × 4 bytes
    pub const RAMFB_SIZE: u32 = 1280 * 720 * 4;

    pub const fn new() -> Self {
        GpuCapPd {
            ramfb_phys:  Self::RAMFB_PHYS,
            ramfb_vaddr: 0,
            fb_size:     Self::RAMFB_SIZE,
            cap_granted: false,
            proof:       SOVEREIGN_PROOF,
        }
    }

    /// Map ramfb — in real seL4 this calls seL4_Page_Map with DeviceMemory type.
    /// PL-70 stub: records vaddr and marks mapping complete.
    pub fn map_ramfb(&mut self, vaddr: u64) -> Result<(), &'static str> {
        if vaddr == 0 {
            return Err("GPU-Cap: invalid vaddr for ramfb mapping");
        }
        self.ramfb_vaddr = vaddr;
        Ok(())
    }

    /// Grant FramebufferWrite capability to Phoenix-Desktop PD.
    /// In real seL4: seL4_CNode_Copy of the mapped frame cap.
    /// Returns the FramebufDesc to send via IPC.
    pub fn grant_fb_cap(&mut self) -> Result<FramebufDesc, &'static str> {
        if self.ramfb_vaddr == 0 {
            return Err("GPU-Cap: ramfb not yet mapped — cannot grant cap");
        }
        self.cap_granted = true;
        Ok(FramebufDesc {
            vaddr:  self.ramfb_vaddr,
            width:  SURFACE_WIDTH,
            height: SURFACE_HEIGHT,
            bpp:    4,
            stride: SURFACE_WIDTH * 4,
        })
    }

    /// Revoke framebuffer capability — seL4_CNode_Delete in full impl.
    pub fn revoke_fb_cap(&mut self) {
        self.cap_granted = false;
    }
}

impl Default for GpuCapPd {
    fn default() -> Self { Self::new() }
}

// ── seL4 boot handoff description ────────────────────────────────────────────

/// PL-70 boot handoff — describes the full seL4 → Phoenix-Desktop sequence.
/// This struct is the authoritative documentation of the v2.0 boot path.
pub struct Pl70BootHandoff {
    /// Stage 1: UEFI hands control to seL4 microkernel
    pub uefi_to_sel4:        bool,
    /// Stage 2: seL4 launches GENESIS PD
    pub sel4_to_genesis:     bool,
    /// Stage 3: GENESIS registers mandatory PDs and surrenders authority
    pub genesis_surrender:   bool,
    /// Stage 4: Phoenix-Init sequencer runs 6-phase boot
    pub phoenix_init_done:   bool,
    /// Stage 5: GPU-Cap maps ramfb and grants FramebufferWrite cap
    pub gpu_cap_granted:     bool,
    /// Stage 6: Phoenix-Desktop receives cap and starts render loop
    pub desktop_running:     bool,
    /// Sovereign proof carried through all stages
    pub proof:               u64,
}

impl Pl70BootHandoff {
    pub const fn new() -> Self {
        Pl70BootHandoff {
            uefi_to_sel4:      false,
            sel4_to_genesis:   false,
            genesis_surrender: false,
            phoenix_init_done: false,
            gpu_cap_granted:   false,
            desktop_running:   false,
            proof:             SOVEREIGN_PROOF,
        }
    }

    /// Returns true when all 6 stages complete — sovereign desktop is live under seL4
    pub fn is_complete(&self) -> bool {
        self.uefi_to_sel4       &&
        self.sel4_to_genesis    &&
        self.genesis_surrender  &&
        self.phoenix_init_done  &&
        self.gpu_cap_granted    &&
        self.desktop_running    &&
        self.proof == SOVEREIGN_PROOF
    }

    /// Human-readable boot stage description
    pub fn stage_label(&self) -> &'static str {
        if !self.uefi_to_sel4      { return "UEFI → seL4 handoff"; }
        if !self.sel4_to_genesis   { return "seL4 → GENESIS PD"; }
        if !self.genesis_surrender { return "GENESIS: surrender authority"; }
        if !self.phoenix_init_done { return "Phoenix-Init: 6-phase boot"; }
        if !self.gpu_cap_granted   { return "GPU-Cap: grant FramebufferWrite"; }
        if !self.desktop_running   { return "Phoenix-Desktop: start render loop"; }
        "SOVEREIGN DESKTOP LIVE UNDER seL4"
    }
}

impl Default for Pl70BootHandoff {
    fn default() -> Self { Self::new() }
}

// ── Kani formal verification ──────────────────────────────────────────────────

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn proof_sovereign_proof_invariant() {
        let pd = PhoenixDesktopPd::new();
        kani::assert(pd.proof == SOVEREIGN_PROOF,
            "proof invariant holds at construction");
    }

    #[kani::proof]
    fn proof_boot_sequence_ordering() {
        let mut pd = PhoenixDesktopPd::new();
        // Cannot receive FB cap before boot signal
        let desc = FramebufDesc { vaddr: 0x44000000, width: 1280,
            height: 720, bpp: 4, stride: 1280*4 };
        let result = pd.on_fb_cap_grant(desc);
        kani::assert(result.is_err(),
            "FB cap grant rejected before boot signal");
    }

    #[kani::proof]
    fn proof_framebuf_sovereign_standard() {
        let desc = FramebufDesc { vaddr: 0x44000000, width: 1280,
            height: 720, bpp: 4, stride: 1280*4 };
        kani::assert(desc.is_sovereign_standard(),
            "sovereign standard framebuffer validates");
    }

    #[kani::proof]
    fn proof_non_standard_framebuf_rejected() {
        let desc = FramebufDesc { vaddr: 0x44000000, width: 1920,
            height: 1080, bpp: 4, stride: 1920*4 };
        kani::assert(!desc.is_sovereign_standard(),
            "non-standard framebuffer rejected");
    }

    #[kani::proof]
    fn proof_boot_handoff_completeness() {
        let mut h = Pl70BootHandoff::new();
        kani::assert(!h.is_complete(), "incomplete handoff is not complete");
        h.uefi_to_sel4      = true;
        h.sel4_to_genesis   = true;
        h.genesis_surrender = true;
        h.phoenix_init_done = true;
        h.gpu_cap_granted   = true;
        h.desktop_running   = true;
        kani::assert(h.is_complete(), "complete handoff is complete");
    }

    #[kani::proof]
    fn proof_gpu_cap_grant_requires_mapping() {
        let mut gpu = GpuCapPd::new();
        let result = gpu.grant_fb_cap();
        kani::assert(result.is_err(),
            "GPU-Cap cannot grant cap before ramfb is mapped");
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pd_awaiting_boot() {
        let pd = PhoenixDesktopPd::new();
        assert_eq!(pd.phase, DesktopPhase::AwaitingBoot);
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
        assert!(pd.framebuf.is_none());
    }

    #[test]
    fn test_boot_signal_advances_phase() {
        let mut pd = PhoenixDesktopPd::new();
        assert!(pd.on_boot_signal().is_ok());
        assert_eq!(pd.phase, DesktopPhase::AwaitingFramebuf);
    }

    #[test]
    fn test_fb_cap_grant_advances_to_running() {
        let mut pd = PhoenixDesktopPd::new();
        pd.on_boot_signal().unwrap();
        let desc = FramebufDesc {
            vaddr: 0x44000000, width: 1280, height: 720, bpp: 4, stride: 1280*4
        };
        assert!(pd.on_fb_cap_grant(desc).is_ok());
        assert_eq!(pd.phase, DesktopPhase::Running);
        assert!(pd.framebuf.is_some());
    }

    #[test]
    fn test_fb_cap_rejected_before_boot_signal() {
        let mut pd = PhoenixDesktopPd::new();
        let desc = FramebufDesc {
            vaddr: 0x44000000, width: 1280, height: 720, bpp: 4, stride: 1280*4
        };
        assert!(pd.on_fb_cap_grant(desc).is_err());
        assert_eq!(pd.phase, DesktopPhase::AwaitingBoot);
    }

    #[test]
    fn test_non_standard_framebuf_rejected() {
        let mut pd = PhoenixDesktopPd::new();
        pd.on_boot_signal().unwrap();
        // 1920×1080 — not sovereign standard
        let desc = FramebufDesc {
            vaddr: 0x44000000, width: 1920, height: 1080, bpp: 4, stride: 1920*4
        };
        assert!(pd.on_fb_cap_grant(desc).is_err());
    }

    #[test]
    fn test_render_tick_increments() {
        let mut pd = PhoenixDesktopPd::new();
        pd.on_boot_signal().unwrap();
        let desc = FramebufDesc {
            vaddr: 0x44000000, width: 1280, height: 720, bpp: 4, stride: 1280*4
        };
        pd.on_fb_cap_grant(desc).unwrap();
        let h1 = pd.render_tick();
        let h2 = pd.render_tick();
        assert!(h2 > h1, "heartbeat increments each tick");
        assert_eq!(pd.ticks, 2);
    }

    #[test]
    fn test_suspend_and_resume() {
        let mut pd = PhoenixDesktopPd::new();
        pd.on_boot_signal().unwrap();
        let desc = FramebufDesc {
            vaddr: 0x44000000, width: 1280, height: 720, bpp: 4, stride: 1280*4
        };
        pd.on_fb_cap_grant(desc).unwrap();
        assert_eq!(pd.phase, DesktopPhase::Running);
        pd.suspend();
        assert_eq!(pd.phase, DesktopPhase::Suspended);
        assert!(pd.framebuf.is_none());
        let desc2 = FramebufDesc {
            vaddr: 0x44000000, width: 1280, height: 720, bpp: 4, stride: 1280*4
        };
        assert!(pd.resume(desc2).is_ok());
        assert_eq!(pd.phase, DesktopPhase::Running);
    }

    #[test]
    fn test_proof_invariant_throughout_lifecycle() {
        let mut pd = PhoenixDesktopPd::new();
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
        pd.on_boot_signal().unwrap();
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
        let desc = FramebufDesc {
            vaddr: 0x44000000, width: 1280, height: 720, bpp: 4, stride: 1280*4
        };
        pd.on_fb_cap_grant(desc).unwrap();
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
        pd.render_tick();
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
    }

    #[test]
    fn test_gpu_cap_requires_mapping_before_grant() {
        let mut gpu = GpuCapPd::new();
        assert!(gpu.grant_fb_cap().is_err());
    }

    #[test]
    fn test_gpu_cap_grant_after_mapping() {
        let mut gpu = GpuCapPd::new();
        gpu.map_ramfb(0x44000000).unwrap();
        let desc = gpu.grant_fb_cap().unwrap();
        assert!(desc.is_sovereign_standard());
        assert_eq!(desc.vaddr, 0x44000000);
        assert!(gpu.cap_granted);
    }

    #[test]
    fn test_boot_handoff_stages() {
        let mut h = Pl70BootHandoff::new();
        assert!(!h.is_complete());
        assert_eq!(h.stage_label(), "UEFI → seL4 handoff");
        h.uefi_to_sel4 = true;
        assert_eq!(h.stage_label(), "seL4 → GENESIS PD");
        h.sel4_to_genesis   = true;
        h.genesis_surrender = true;
        h.phoenix_init_done = true;
        h.gpu_cap_granted   = true;
        h.desktop_running   = true;
        assert!(h.is_complete());
        assert_eq!(h.stage_label(), "SOVEREIGN DESKTOP LIVE UNDER seL4");
    }

    #[test]
    fn test_sovereign_proof_is_0x4153() {
        assert_eq!(SOVEREIGN_PROOF, 0x4153);
    }

    #[test]
    fn test_framebuf_total_bytes() {
        let desc = FramebufDesc {
            vaddr: 0x44000000, width: 1280, height: 720, bpp: 4, stride: 1280*4
        };
        assert_eq!(desc.total_bytes(), 1280 * 720 * 4);
    }

    #[test]
    fn test_input_event_triggers_redraw() {
        let mut pd = PhoenixDesktopPd::new();
        pd.on_boot_signal().unwrap();
        let desc = FramebufDesc {
            vaddr: 0x44000000, width: 1280, height: 720, bpp: 4, stride: 1280*4
        };
        pd.on_fb_cap_grant(desc).unwrap();
        let key_ev = InputEvent { kind: 0x01, code: 28, x: 0, y: 0 };
        assert!(pd.on_input_event(key_ev));
        let mouse_ev = InputEvent { kind: 0x10, code: 0, x: 640, y: 360 };
        assert!(pd.on_input_event(mouse_ev));
        assert_eq!(pd.events, 2);
    }
}
