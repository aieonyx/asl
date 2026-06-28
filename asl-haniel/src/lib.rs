// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// asl-haniel — HANIEL Protection Domain
//
// Sovereign render surface PD for the AIEONYX Sovereign Layer.
// Replaces WebKitGTK as the renderer inside the Onyxia Browser PD for AWP URLs.
//
// Capability policy (enforced by seL4):
//   GRANTED  : DisplaySurface — write to framebuffer
//   GRANTED  : FontRead       — read sovereign font store
//   DENIED   : Network        — renderer has zero network access (NetworkNone)
//   DENIED   : Storage write  — renderer is read-only to VAULT cache
//
// Render surface: 1280x720 ARGB8888 (sovereign standard, established M11)
// Sovereign proof: axon_main() → 0x4153 (invariant across all milestones)
//
// URL routing policy:
//   awp://   → HANIEL PD  (sovereign path — this PD)
//   https:// → WebKitGTK  (legacy fallback — HTTPS only)
//   http://  → BLOCKED    (no cleartext)
//
// S4+i: Security first — renderer is capability-isolated, cannot exfiltrate data.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(kani)]
extern crate kani;

extern crate alloc;

use alloc::vec::Vec;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Sovereign framebuffer width (pixels).
pub const SURFACE_WIDTH: u32 = 1280;

/// Sovereign framebuffer height (pixels).
pub const SURFACE_HEIGHT: u32 = 720;

/// Bytes per pixel — ARGB8888.
pub const BYTES_PER_PIXEL: u32 = 4;

/// Total framebuffer size in bytes.
pub const SURFACE_BYTES: usize = (SURFACE_WIDTH * SURFACE_HEIGHT * BYTES_PER_PIXEL) as usize;

/// Sovereign proof value — must remain invariant across all ASL milestones.
pub const AXON_PROOF: u64 = 0x4153;

/// HANIEL PD identifier.
pub const HANIEL_PD_ID: u8 = 0x20;

/// AWP URL scheme prefix.
pub const AWP_SCHEME: &[u8] = b"awp://";

/// HTTPS URL scheme prefix (legacy fallback only).
pub const HTTPS_SCHEME: &[u8] = b"https://";

/// Sovereign site marker byte (✦ encoded as sentinel).
pub const AWP_SOVEREIGN_MARKER: u8 = 0xA1;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HanielError {
    /// URL scheme is not AWP — must be routed to legacy fallback.
    NotAwp,
    /// URL is HTTP cleartext — blocked by sovereign policy.
    CleartextBlocked,
    /// Surface buffer size mismatch.
    SurfaceSizeMismatch,
    /// Render budget exceeded — frame dropped.
    BudgetExceeded,
    /// Capability violation — operation not permitted in this PD.
    CapabilityDenied,
    /// Invalid input.
    InvalidInput,
}

// ── URL routing ───────────────────────────────────────────────────────────────

/// URL route decision from the HANIEL PD router.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RouteDecision {
    /// Render through HANIEL sovereign path.
    Haniel,
    /// Route to WebKitGTK legacy fallback (HTTPS only).
    WebKitLegacy,
    /// Block — cleartext HTTP not permitted.
    Block,
}

/// Classify a URL and return a routing decision.
///
/// AWP URLs → HANIEL PD (this PD).
/// HTTPS URLs → WebKitGTK legacy fallback.
/// HTTP URLs → blocked by sovereign policy.
/// Unknown schemes → blocked.
pub fn route_url(url: &[u8]) -> RouteDecision {
    if url.is_empty() {
        return RouteDecision::Block;
    }
    if url.starts_with(AWP_SCHEME) {
        return RouteDecision::Haniel;
    }
    if url.starts_with(HTTPS_SCHEME) {
        return RouteDecision::WebKitLegacy;
    }
    // http:// and all unknown schemes — block
    RouteDecision::Block
}

// ── Render surface ────────────────────────────────────────────────────────────

/// A sovereign render surface — 1280×720 ARGB8888 framebuffer.
///
/// Allocated once at PD boot. Written by CANVAS, read by display controller.
/// No network access from this PD (NetworkNone capability policy).
pub struct RenderSurface {
    /// Flat ARGB8888 pixel buffer — row-major, top-left origin.
    pixels: Vec<u32>,
    /// Width in pixels.
    width: u32,
    /// Height in pixels.
    height: u32,
    /// Frame counter — incremented on every commit.
    frame_count: u64,
    /// Render budget remaining for current frame (arbitrary units).
    budget_remaining: u32,
}

impl RenderSurface {
    /// Allocate the sovereign render surface at standard dimensions.
    pub fn new() -> Self {
        Self {
            pixels: alloc::vec![0u32; (SURFACE_WIDTH * SURFACE_HEIGHT) as usize],
            width: SURFACE_WIDTH,
            height: SURFACE_HEIGHT,
            frame_count: 0,
            budget_remaining: 1000,
        }
    }

    /// Width in pixels.
    #[inline]
    pub fn width(&self) -> u32 { self.width }

    /// Height in pixels.
    #[inline]
    pub fn height(&self) -> u32 { self.height }

    /// Total pixel count.
    #[inline]
    pub fn pixel_count(&self) -> usize { (self.width * self.height) as usize }

    /// Frame counter value.
    #[inline]
    pub fn frame_count(&self) -> u64 { self.frame_count }

    /// Remaining render budget for this frame.
    #[inline]
    pub fn budget_remaining(&self) -> u32 { self.budget_remaining }

    /// Write a single pixel at (x, y). Out-of-bounds writes are silently dropped.
    pub fn put_pixel(&mut self, x: u32, y: u32, argb: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        self.pixels[idx] = argb;
    }

    /// Read a pixel at (x, y). Returns 0x00000000 for out-of-bounds.
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.pixels[(y * self.width + x) as usize]
    }

    /// Fill the entire surface with a solid ARGB colour.
    pub fn clear(&mut self, argb: u32) {
        for p in self.pixels.iter_mut() {
            *p = argb;
        }
    }

    /// Commit the frame — increments frame counter, resets budget.
    /// Returns the new frame count.
    pub fn commit(&mut self) -> u64 {
        self.frame_count = self.frame_count.saturating_add(1);
        self.budget_remaining = 1000;
        self.frame_count
    }

    /// Consume `cost` budget units. Returns Err if budget exhausted.
    pub fn spend_budget(&mut self, cost: u32) -> Result<u32, HanielError> {
        if cost > self.budget_remaining {
            return Err(HanielError::BudgetExceeded);
        }
        self.budget_remaining -= cost;
        Ok(self.budget_remaining)
    }

    /// Return a read-only slice of the pixel buffer.
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Verify the surface dimensions match the sovereign standard.
    pub fn verify_dimensions(&self) -> bool {
        self.width == SURFACE_WIDTH && self.height == SURFACE_HEIGHT
    }
}

impl Default for RenderSurface {
    fn default() -> Self {
        Self::new()
    }
}

// ── Capability gate ───────────────────────────────────────────────────────────

/// Capability types available to the HANIEL PD.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HanielCap {
    /// Write to display framebuffer.
    DisplaySurface,
    /// Read sovereign font store.
    FontRead,
    /// Network access — NOT granted to HANIEL PD.
    Network,
    /// Storage write — NOT granted to HANIEL PD.
    StorageWrite,
}

/// Check whether a capability is granted to the HANIEL PD.
///
/// Enforced at seL4 level — this function mirrors the policy for testing.
pub fn cap_granted(cap: HanielCap) -> bool {
    match cap {
        HanielCap::DisplaySurface => true,
        HanielCap::FontRead => true,
        HanielCap::Network => false,       // NetworkNone
        HanielCap::StorageWrite => false,  // read-only VAULT cache
    }
}

/// Assert a capability is granted. Returns CapabilityDenied if not.
pub fn assert_cap(cap: HanielCap) -> Result<(), HanielError> {
    if cap_granted(cap) {
        Ok(())
    } else {
        Err(HanielError::CapabilityDenied)
    }
}

// ── Sovereign proof ───────────────────────────────────────────────────────────

/// Verify the sovereign proof value is invariant.
/// axon_main() → 0x4153 must hold across all ASL milestones.
#[inline]
pub fn verify_sovereign_proof(proof: u64) -> bool {
    proof == AXON_PROOF
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL routing ──────────────────────────────────────────────────────────

    #[test]
    fn test_awp_routes_to_haniel() {
        assert_eq!(route_url(b"awp://aegis"), RouteDecision::Haniel);
    }

    #[test]
    fn test_awp_any_path_routes_to_haniel() {
        assert_eq!(route_url(b"awp://sovereign.node/page"), RouteDecision::Haniel);
    }

    #[test]
    fn test_https_routes_to_webkit_legacy() {
        assert_eq!(route_url(b"https://example.com"), RouteDecision::WebKitLegacy);
    }

    #[test]
    fn test_http_is_blocked() {
        assert_eq!(route_url(b"http://example.com"), RouteDecision::Block);
    }

    #[test]
    fn test_empty_url_blocked() {
        assert_eq!(route_url(b""), RouteDecision::Block);
    }

    #[test]
    fn test_unknown_scheme_blocked() {
        assert_eq!(route_url(b"ftp://files.example.com"), RouteDecision::Block);
    }

    #[test]
    fn test_bare_string_blocked() {
        assert_eq!(route_url(b"notaurl"), RouteDecision::Block);
    }

    // ── Render surface ───────────────────────────────────────────────────────

    #[test]
    fn test_surface_dimensions() {
        let s = RenderSurface::new();
        assert_eq!(s.width(), SURFACE_WIDTH);
        assert_eq!(s.height(), SURFACE_HEIGHT);
    }

    #[test]
    fn test_surface_pixel_count() {
        let s = RenderSurface::new();
        assert_eq!(s.pixel_count(), (SURFACE_WIDTH * SURFACE_HEIGHT) as usize);
    }

    #[test]
    fn test_surface_verify_dimensions() {
        let s = RenderSurface::new();
        assert!(s.verify_dimensions());
    }

    #[test]
    fn test_surface_initial_pixels_zero() {
        let s = RenderSurface::new();
        assert!(s.pixels().iter().all(|&p| p == 0));
    }

    #[test]
    fn test_put_get_pixel() {
        let mut s = RenderSurface::new();
        s.put_pixel(100, 200, 0xFF_00_C8_96); // AIEONYX teal
        assert_eq!(s.get_pixel(100, 200), 0xFF_00_C8_96);
    }

    #[test]
    fn test_out_of_bounds_write_ignored() {
        let mut s = RenderSurface::new();
        s.put_pixel(9999, 9999, 0xDEADBEEF); // should not panic
        assert_eq!(s.get_pixel(9999, 9999), 0); // OOB read returns 0
    }

    #[test]
    fn test_clear_fills_surface() {
        let mut s = RenderSurface::new();
        s.clear(0xFF_0A_0F_1A); // sovereign midnight
        assert!(s.pixels().iter().all(|&p| p == 0xFF_0A_0F_1A));
    }

    #[test]
    fn test_commit_increments_frame_count() {
        let mut s = RenderSurface::new();
        assert_eq!(s.frame_count(), 0);
        s.commit();
        assert_eq!(s.frame_count(), 1);
        s.commit();
        assert_eq!(s.frame_count(), 2);
    }

    #[test]
    fn test_commit_resets_budget() {
        let mut s = RenderSurface::new();
        s.spend_budget(500).unwrap();
        assert_eq!(s.budget_remaining(), 500);
        s.commit();
        assert_eq!(s.budget_remaining(), 1000);
    }

    #[test]
    fn test_spend_budget_deducts() {
        let mut s = RenderSurface::new();
        let remaining = s.spend_budget(300).unwrap();
        assert_eq!(remaining, 700);
    }

    #[test]
    fn test_budget_exhausted_returns_error() {
        let mut s = RenderSurface::new();
        let result = s.spend_budget(1001);
        assert_eq!(result, Err(HanielError::BudgetExceeded));
    }

    // ── Capability gate ──────────────────────────────────────────────────────

    #[test]
    fn test_display_surface_cap_granted() {
        assert!(cap_granted(HanielCap::DisplaySurface));
    }

    #[test]
    fn test_font_read_cap_granted() {
        assert!(cap_granted(HanielCap::FontRead));
    }

    #[test]
    fn test_network_cap_denied() {
        assert!(!cap_granted(HanielCap::Network));
    }

    #[test]
    fn test_storage_write_cap_denied() {
        assert!(!cap_granted(HanielCap::StorageWrite));
    }

    #[test]
    fn test_assert_cap_granted_ok() {
        assert!(assert_cap(HanielCap::DisplaySurface).is_ok());
    }

    #[test]
    fn test_assert_cap_denied_err() {
        assert_eq!(assert_cap(HanielCap::Network), Err(HanielError::CapabilityDenied));
    }

    // ── Sovereign proof ──────────────────────────────────────────────────────

    #[test]
    fn test_sovereign_proof_invariant() {
        assert!(verify_sovereign_proof(0x4153));
    }

    #[test]
    fn test_wrong_proof_fails() {
        assert!(!verify_sovereign_proof(0x0000));
        assert!(!verify_sovereign_proof(0xDEAD));
        assert!(!verify_sovereign_proof(0x4152)); // off by one
    }

    #[test]
    fn test_axon_proof_constant() {
        assert_eq!(AXON_PROOF, 0x4153);
    }

    // ── Constants ────────────────────────────────────────────────────────────

    #[test]
    fn test_surface_bytes_constant() {
        assert_eq!(SURFACE_BYTES, 1280 * 720 * 4);
    }

    #[test]
    fn test_haniel_pd_id() {
        assert_eq!(HANIEL_PD_ID, 0x20);
    }

    #[test]
    fn test_awp_scheme_prefix() {
        assert_eq!(AWP_SCHEME, b"awp://");
    }
}
