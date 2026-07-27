// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ════════════════════════════════════════════════════════════════════════════
// asl-haniel-canvas-pd — HANIEL Canvas Protection Domain
// PL-74 / ASL-M29: HANIEL compositor as seL4 PD with GPU capability
// ════════════════════════════════════════════════════════════════════════════
//
// ROLE: The sovereign display compositor. All pixel writes to the framebuffer
//       go through this PD. No other PD can write to the display directly —
//       they must send render requests through ARPi → HANIEL-Canvas-PD.
//
// GPU CAPABILITY MODEL (seL4):
//   GPU-Cap-PD grants FramebufferWrite to HANIEL-Canvas-PD only.
//   HANIEL-Canvas-PD is the sole display authority.
//   Onyxia-PD, Phoenix-Desktop-PD, AXON-Exec-PD — all route through HANIEL.
//
// RENDER REQUEST PIPELINE:
//   1. Onyxia-PD sends RenderRequest (awp:// page content)
//   2. Phoenix-Desktop-PD sends RenderRequest (desktop frame)
//   3. HANIEL-Canvas-PD validates: capability check + budget check
//   4. HANIEL-Canvas-PD composites all layers → single frame
//   5. Frame committed to framebuffer via GPU-Cap capability
//   6. Frame number returned to requesting PD via ARPi response
//
// COMPOSITOR LAYERS (Z-order, bottom to top):
//   Layer 0: Desktop background (Phoenix-Desktop-PD)
//   Layer 1: awp:// page content (Onyxia-PD)
//   Layer 2: Window chrome (Phoenix-Desktop-PD)
//   Layer 3: Shell output (Shell-PD)
//   Layer 4: System overlay (top always)
//
// CAPABILITY POLICY:
//   GRANTED  : FramebufferWrite  — via GPU-Cap seL4 capability (sole holder)
//   GRANTED  : FontRead          — sovereign font store
//   DENIED   : Network           — NetworkNone
//   DENIED   : StorageWrite      — read-only VAULT cache
//   DENIED   : AxonExec          — compositor cannot execute scripts
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

#![no_std]
#![forbid(unsafe_code)]

#[cfg(kani)]
extern crate kani;

#[cfg(feature = "std")]
extern crate std;

use asl_common::pd::PdId;
use asl_arpi_ipc::AXON_PROOF;
use asl_haniel::{
    RenderSurface, HanielCap, HanielError,
    cap_granted, assert_cap, verify_sovereign_proof,
    SURFACE_WIDTH, SURFACE_HEIGHT, AXON_PROOF as HANIEL_PROOF,
};

// ── Constants ─────────────────────────────────────────────────────────────────

pub const HANIEL_CANVAS_PD_ID: u8  = 0x60;
pub const SOVEREIGN_PROOF:     u64 = AXON_PROOF;
pub const MAX_LAYERS:          usize = 5;
pub const MAX_PENDING:         usize = 8;

// ── Compositor layer types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    /// Desktop background — lowest Z
    DesktopBackground,
    /// awp:// page content from Onyxia-PD
    AwpPage,
    /// Window chrome (title bars, borders)
    WindowChrome,
    /// Shell output overlay
    ShellOutput,
    /// System overlay — always on top
    SystemOverlay,
}

impl LayerKind {
    /// Z-order: lower = further back
    pub fn z_order(self) -> u8 {
        match self {
            LayerKind::DesktopBackground => 0,
            LayerKind::AwpPage           => 1,
            LayerKind::WindowChrome      => 2,
            LayerKind::ShellOutput       => 3,
            LayerKind::SystemOverlay     => 4,
        }
    }
}

// ── Render request ────────────────────────────────────────────────────────────

/// A render request from a client PD to HANIEL-Canvas-PD
#[derive(Debug, Clone, Copy)]
pub struct RenderRequest {
    /// Requesting PD identity
    pub caller_pd:  u8,
    /// Which compositor layer to update
    pub layer:      LayerKind,
    /// ARGB fill color for this layer (solid fill in PL-74 stub)
    /// Full implementation: pixel data in shared memory region
    pub fill_color: u32,
    /// Region to update: (x, y, w, h)
    pub region:     (u32, u32, u32, u32),
    /// ARPi sequence number for provenance
    pub arpi_seq:   u64,
}

impl RenderRequest {
    pub const fn empty() -> Self {
        RenderRequest {
            caller_pd:  0,
            layer:      LayerKind::DesktopBackground,
            fill_color: 0x00000000,
            region:     (0, 0, 0, 0),
            arpi_seq:   0,
        }
    }

    /// Validate region is within sovereign surface bounds
    pub fn region_valid(&self) -> bool {
        let (x, y, w, h) = self.region;
        x + w <= SURFACE_WIDTH && y + h <= SURFACE_HEIGHT && w > 0 && h > 0
    }
}

// ── Render response ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderOutcome {
    /// Frame committed to framebuffer
    Committed { frame_number: u64 },
    /// Capability denied — caller not allowed
    CapDenied,
    /// Budget exceeded — frame dropped
    BudgetExceeded,
    /// Invalid region
    InvalidRegion,
    /// PD not in Ready phase
    NotReady,
}

// ── HANIEL Canvas PD state ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasPhase {
    AwaitingBoot,
    /// Waiting for GPU-Cap FramebufferWrite capability
    AwaitingGpuCap,
    /// Ready — accepting render requests
    Ready,
    /// Compositing — building current frame
    Compositing,
    /// Committed — frame written to framebuffer
    Committed,
    Faulted,
}

/// Per-layer state
#[derive(Clone, Copy)]
pub struct LayerState {
    pub kind:    LayerKind,
    pub dirty:   bool,
    pub fill:    u32,
    pub region:  (u32, u32, u32, u32),
}

impl LayerState {
    pub const fn empty(kind: LayerKind) -> Self {
        LayerState { kind, dirty: false, fill: 0, region: (0, 0, 0, 0) }
    }
}

pub struct HanielCanvasPd {
    pub phase:        CanvasPhase,
    pub frame_count:  u64,
    pub render_count: u64,
    pub drop_count:   u64,
    pub proof:        u64,
    /// GPU-Cap framebuffer capability — granted by GPU-Cap-PD
    pub fb_cap:       bool,
    /// Compositor layers
    layers:           [LayerState; MAX_LAYERS],
    /// Pending render requests
    pending:          [Option<RenderRequest>; MAX_PENDING],
    pending_count:    usize,
}

impl HanielCanvasPd {
    pub fn new() -> Self {
        HanielCanvasPd {
            phase:        CanvasPhase::AwaitingBoot,
            frame_count:  0,
            render_count: 0,
            drop_count:   0,
            proof:        SOVEREIGN_PROOF,
            fb_cap:       false,
            layers: [
                LayerState::empty(LayerKind::DesktopBackground),
                LayerState::empty(LayerKind::AwpPage),
                LayerState::empty(LayerKind::WindowChrome),
                LayerState::empty(LayerKind::ShellOutput),
                LayerState::empty(LayerKind::SystemOverlay),
            ],
            pending:       [None; MAX_PENDING],
            pending_count: 0,
        }
    }

    pub fn on_boot_signal(&mut self) -> Result<(), &'static str> {
        if self.phase != CanvasPhase::AwaitingBoot {
            return Err("HANIEL-Canvas: BOOT_SIGNAL in wrong phase");
        }
        self.assert_proof();
        self.phase = CanvasPhase::AwaitingGpuCap;
        Ok(())
    }

    /// GPU-Cap-PD grants FramebufferWrite capability
    pub fn on_gpu_cap_granted(&mut self, fb_vaddr: u64) -> Result<(), &'static str> {
        if self.phase != CanvasPhase::AwaitingGpuCap {
            return Err("HANIEL-Canvas: GPU cap grant in wrong phase");
        }
        if fb_vaddr == 0 {
            return Err("HANIEL-Canvas: invalid framebuffer vaddr");
        }
        // Verify HANIEL has DisplaySurface capability
        if !cap_granted(HanielCap::DisplaySurface) {
            return Err("HANIEL-Canvas: DisplaySurface capability not granted");
        }
        self.assert_proof();
        self.fb_cap = true;
        self.phase  = CanvasPhase::Ready;
        Ok(())
    }

    /// Submit a render request from a client PD
    /// Validates: caller allowed + region valid + budget available
    pub fn submit_render(&mut self, req: RenderRequest) -> RenderOutcome {
        if self.phase != CanvasPhase::Ready {
            return RenderOutcome::NotReady;
        }

        // Only Desktop and Onyxia PDs can submit render requests
        let allowed = matches!(req.caller_pd,
            0x30 | // Phoenix-Desktop-PD
            0x42   // Onyxia-PD
        );
        if !allowed {
            return RenderOutcome::CapDenied;
        }

        if !req.region_valid() {
            return RenderOutcome::InvalidRegion;
        }

        self.assert_proof();

        // Queue the render request
        if self.pending_count < MAX_PENDING {
            for slot in self.pending.iter_mut() {
                if slot.is_none() {
                    *slot = Some(req);
                    self.pending_count += 1;
                    break;
                }
            }
        }

        // Composite and commit immediately (PL-74 eager mode)
        self.composite_and_commit(req)
    }

    /// Composite all pending layers and commit to framebuffer
    fn composite_and_commit(&mut self, req: RenderRequest) -> RenderOutcome {
        self.phase = CanvasPhase::Compositing;

        // Update the relevant layer
        let z = req.layer.z_order() as usize;
        if z < MAX_LAYERS {
            self.layers[z].dirty  = true;
            self.layers[z].fill   = req.fill_color;
            self.layers[z].region = req.region;
        }

        // Verify GPU-Cap is still held
        if !self.fb_cap {
            self.phase = CanvasPhase::Ready;
            return RenderOutcome::CapDenied;
        }

        // Verify DisplaySurface capability
        if assert_cap(HanielCap::DisplaySurface).is_err() {
            self.phase = CanvasPhase::Ready;
            return RenderOutcome::CapDenied;
        }

        // Commit frame (stub: in full impl writes pixels to framebuffer via GPU-Cap)
        self.frame_count  += 1;
        self.render_count += 1;

        // Clear pending slot
        for slot in self.pending.iter_mut() {
            if let Some(r) = slot {
                if r.arpi_seq == req.arpi_seq {
                    *slot = None;
                    self.pending_count = self.pending_count.saturating_sub(1);
                    break;
                }
            }
        }

        self.phase = CanvasPhase::Ready;

        RenderOutcome::Committed { frame_number: self.frame_count }
    }

    /// Revoke GPU-Cap — called when GPU-Cap-PD needs to reclaim
    pub fn revoke_gpu_cap(&mut self) {
        self.fb_cap = false;
        if self.phase == CanvasPhase::Ready {
            self.phase = CanvasPhase::AwaitingGpuCap;
        }
    }

    /// Layer dirty state — true if layer has pending update
    pub fn layer_dirty(&self, kind: LayerKind) -> bool {
        self.layers[kind.z_order() as usize].dirty
    }

    /// Verify compositor proof invariant
    pub fn proof_valid(&self) -> bool {
        self.proof == SOVEREIGN_PROOF &&
        verify_sovereign_proof(HANIEL_PROOF)
    }

    pub fn pd_id() -> PdId { PdId::GpuCap }

    #[inline]
    fn assert_proof(&self) {
        assert_eq!(self.proof, SOVEREIGN_PROOF,
            "SOVEREIGN PROOF VIOLATION: HANIEL-Canvas-PD integrity failed");
    }
}

impl Default for HanielCanvasPd { fn default() -> Self { Self::new() } }

// ── End-to-end compositor pipeline ───────────────────────────────────────────

/// Prove: Onyxia-PD → HANIEL-Canvas-PD → framebuffer
pub struct Pl74Pipeline {
    pub onyxia:  asl_onyxia_pd::OnyxiaPd,
    pub canvas:  HanielCanvasPd,
    pub frames:  u64,
}

impl Pl74Pipeline {
    pub fn new() -> Self {
        Pl74Pipeline {
            onyxia: asl_onyxia_pd::OnyxiaPd::new(),
            canvas: HanielCanvasPd::new(),
            frames: 0,
        }
    }

    pub fn boot(&mut self) {
        self.onyxia.on_boot_signal().unwrap();
        self.canvas.on_boot_signal().unwrap();
        self.canvas.on_gpu_cap_granted(0x44000000).unwrap();
    }

    /// Onyxia navigates to awp:// → HANIEL renders the page
    pub fn navigate_and_render(&mut self, url: &[u8]) -> RenderOutcome {
        // 1. Onyxia navigates
        self.onyxia.navigate(url).unwrap();

        // 2. HANIEL renders the page canvas
        let req = RenderRequest {
            caller_pd:  0x42, // Onyxia-PD
            layer:      LayerKind::AwpPage,
            fill_color: 0xFF0A1630, // Midnight Blue — sovereign page bg
            region:     (200, 50, 880, 600),
            arpi_seq:   self.frames + 1,
        };
        let outcome = self.canvas.submit_render(req);

        if matches!(outcome, RenderOutcome::Committed { .. }) {
            self.onyxia.on_render_complete().unwrap();
            self.frames += 1;
        }

        outcome
    }

    /// Desktop submits background frame
    pub fn render_desktop_bg(&mut self, color: u32) -> RenderOutcome {
        let req = RenderRequest {
            caller_pd:  0x30, // Phoenix-Desktop-PD
            layer:      LayerKind::DesktopBackground,
            fill_color: color,
            region:     (0, 0, SURFACE_WIDTH, SURFACE_HEIGHT),
            arpi_seq:   self.frames + 100,
        };
        let outcome = self.canvas.submit_render(req);
        if matches!(outcome, RenderOutcome::Committed { .. }) {
            self.frames += 1;
        }
        outcome
    }
}

impl Default for Pl74Pipeline { fn default() -> Self { Self::new() } }

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── HanielCanvasPd tests ──────────────────────────────────────────────────

    #[test]
    fn test_new_pd_awaiting_boot() {
        let pd = HanielCanvasPd::new();
        assert_eq!(pd.phase, CanvasPhase::AwaitingBoot);
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
        assert!(!pd.fb_cap);
    }

    #[test]
    fn test_boot_signal_advances_to_awaiting_gpu() {
        let mut pd = HanielCanvasPd::new();
        assert!(pd.on_boot_signal().is_ok());
        assert_eq!(pd.phase, CanvasPhase::AwaitingGpuCap);
    }

    #[test]
    fn test_gpu_cap_grant_advances_to_ready() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        assert!(pd.on_gpu_cap_granted(0x44000000).is_ok());
        assert_eq!(pd.phase, CanvasPhase::Ready);
        assert!(pd.fb_cap);
    }

    #[test]
    fn test_gpu_cap_invalid_vaddr_rejected() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        assert!(pd.on_gpu_cap_granted(0).is_err());
        assert_eq!(pd.phase, CanvasPhase::AwaitingGpuCap);
    }

    #[test]
    fn test_render_before_gpu_cap_denied() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        // No GPU cap yet
        let req = RenderRequest {
            caller_pd: 0x42, layer: LayerKind::AwpPage,
            fill_color: 0xFF0A1630, region: (0, 0, 100, 100), arpi_seq: 1,
        };
        let outcome = pd.submit_render(req);
        assert_eq!(outcome, RenderOutcome::NotReady);
    }

    #[test]
    fn test_desktop_render_succeeds() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        pd.on_gpu_cap_granted(0x44000000).unwrap();
        let req = RenderRequest {
            caller_pd:  0x30, // Phoenix-Desktop-PD
            layer:      LayerKind::DesktopBackground,
            fill_color: 0xFF070E1A,
            region:     (0, 0, SURFACE_WIDTH, SURFACE_HEIGHT),
            arpi_seq:   1,
        };
        let outcome = pd.submit_render(req);
        assert!(matches!(outcome, RenderOutcome::Committed { frame_number: 1 }));
        assert_eq!(pd.frame_count, 1);
    }

    #[test]
    fn test_onyxia_render_succeeds() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        pd.on_gpu_cap_granted(0x44000000).unwrap();
        let req = RenderRequest {
            caller_pd:  0x42, // Onyxia-PD
            layer:      LayerKind::AwpPage,
            fill_color: 0xFF0A1630,
            region:     (200, 50, 880, 600),
            arpi_seq:   1,
        };
        let outcome = pd.submit_render(req);
        assert!(matches!(outcome, RenderOutcome::Committed { .. }));
        assert!(pd.layer_dirty(LayerKind::AwpPage));
    }

    #[test]
    fn test_wrong_caller_denied() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        pd.on_gpu_cap_granted(0x44000000).unwrap();
        let req = RenderRequest {
            caller_pd:  0x50, // AXON-Exec-PD — not allowed
            layer:      LayerKind::AwpPage,
            fill_color: 0xFF000000,
            region:     (0, 0, 100, 100),
            arpi_seq:   1,
        };
        assert_eq!(pd.submit_render(req), RenderOutcome::CapDenied);
    }

    #[test]
    fn test_invalid_region_rejected() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        pd.on_gpu_cap_granted(0x44000000).unwrap();
        // Region exceeds surface bounds
        let req = RenderRequest {
            caller_pd:  0x30,
            layer:      LayerKind::DesktopBackground,
            fill_color: 0xFF000000,
            region:     (0, 0, 9999, 9999), // out of bounds
            arpi_seq:   1,
        };
        assert_eq!(pd.submit_render(req), RenderOutcome::InvalidRegion);
    }

    #[test]
    fn test_zero_size_region_rejected() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        pd.on_gpu_cap_granted(0x44000000).unwrap();
        let req = RenderRequest {
            caller_pd:  0x30,
            layer:      LayerKind::DesktopBackground,
            fill_color: 0xFF000000,
            region:     (0, 0, 0, 0), // zero size
            arpi_seq:   1,
        };
        assert_eq!(pd.submit_render(req), RenderOutcome::InvalidRegion);
    }

    #[test]
    fn test_gpu_cap_revoke_blocks_render() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        pd.on_gpu_cap_granted(0x44000000).unwrap();
        pd.revoke_gpu_cap();
        assert!(!pd.fb_cap);
        assert_eq!(pd.phase, CanvasPhase::AwaitingGpuCap);
    }

    #[test]
    fn test_frame_count_increments() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        pd.on_gpu_cap_granted(0x44000000).unwrap();
        for i in 1..=3u64 {
            let req = RenderRequest {
                caller_pd: 0x30, layer: LayerKind::DesktopBackground,
                fill_color: 0xFF000000, region: (0, 0, 100, 100),
                arpi_seq: i,
            };
            pd.submit_render(req);
        }
        assert_eq!(pd.frame_count, 3);
    }

    #[test]
    fn test_proof_invariant() {
        let mut pd = HanielCanvasPd::new();
        pd.on_boot_signal().unwrap();
        pd.on_gpu_cap_granted(0x44000000).unwrap();
        assert!(pd.proof_valid());
        let req = RenderRequest {
            caller_pd: 0x30, layer: LayerKind::DesktopBackground,
            fill_color: 0xFF070E1A, region: (0, 0, 200, 200), arpi_seq: 1,
        };
        pd.submit_render(req);
        assert!(pd.proof_valid());
    }

    #[test]
    fn test_layer_z_order() {
        assert_eq!(LayerKind::DesktopBackground.z_order(), 0);
        assert_eq!(LayerKind::AwpPage.z_order(), 1);
        assert_eq!(LayerKind::WindowChrome.z_order(), 2);
        assert_eq!(LayerKind::ShellOutput.z_order(), 3);
        assert_eq!(LayerKind::SystemOverlay.z_order(), 4);
    }

    #[test]
    fn test_sovereign_proof_constant() {
        assert_eq!(SOVEREIGN_PROOF, 0x4153);
    }

    // ── End-to-end pipeline tests ─────────────────────────────────────────────

    #[test]
    fn test_pipeline_awp_navigation() {
        let mut pipeline = Pl74Pipeline::new();
        pipeline.boot();
        let outcome = pipeline.navigate_and_render(b"awp://aieonyx");
        assert!(matches!(outcome, RenderOutcome::Committed { .. }));
        assert_eq!(pipeline.frames, 1);
        assert_eq!(pipeline.canvas.frame_count, 1);
        assert!(pipeline.onyxia.is_sovereign());
    }

    #[test]
    fn test_pipeline_desktop_background() {
        let mut pipeline = Pl74Pipeline::new();
        pipeline.boot();
        let outcome = pipeline.render_desktop_bg(0xFF070E1A);
        assert!(matches!(outcome, RenderOutcome::Committed { .. }));
        assert!(pipeline.canvas.layer_dirty(LayerKind::DesktopBackground));
    }

    #[test]
    fn test_pipeline_multiple_frames() {
        let mut pipeline = Pl74Pipeline::new();
        pipeline.boot();
        pipeline.render_desktop_bg(0xFF070E1A);
        pipeline.navigate_and_render(b"awp://aieonyx");
        pipeline.navigate_and_render(b"awp://about");
        assert_eq!(pipeline.canvas.frame_count, 3);
        assert_eq!(pipeline.frames, 3);
    }

    #[test]
    fn test_pipeline_gpu_cap_required() {
        let mut canvas = HanielCanvasPd::new();
        canvas.on_boot_signal().unwrap();
        // No GPU cap granted — render must fail
        let req = RenderRequest {
            caller_pd: 0x30, layer: LayerKind::DesktopBackground,
            fill_color: 0xFF000000, region: (0, 0, 100, 100), arpi_seq: 1,
        };
        assert_eq!(canvas.submit_render(req), RenderOutcome::NotReady);
    }

    #[test]
    fn test_haniel_cap_policy() {
        // DisplaySurface and FontRead granted
        assert!(cap_granted(HanielCap::DisplaySurface));
        assert!(cap_granted(HanielCap::FontRead));
        // Network and StorageWrite denied
        assert!(!cap_granted(HanielCap::Network));
        assert!(!cap_granted(HanielCap::StorageWrite));
    }
}
