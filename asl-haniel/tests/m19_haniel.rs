// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// m19_haniel.rs — Integration tests for HANIEL PD (M19)
// Target: 30+ tests, 0 failures
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_haniel::{
    route_url, RouteDecision, RenderSurface, HanielError,
    cap_granted, assert_cap, HanielCap, verify_sovereign_proof,
    SURFACE_WIDTH, SURFACE_HEIGHT, SURFACE_BYTES, AXON_PROOF,
    HANIEL_PD_ID, AWP_SCHEME, HTTPS_SCHEME,
};

// ── URL routing integration ───────────────────────────────────────────────────

#[test]
fn test_awp_aegis_routes_sovereign() {
    assert_eq!(route_url(b"awp://aegis"), RouteDecision::Haniel);
}

#[test]
fn test_awp_soma_routes_sovereign() {
    assert_eq!(route_url(b"awp://soma.identity/auth"), RouteDecision::Haniel);
}

#[test]
fn test_awp_bastion_routes_sovereign() {
    assert_eq!(route_url(b"awp://bastion.node/mesh"), RouteDecision::Haniel);
}

#[test]
fn test_https_google_legacy() {
    assert_eq!(route_url(b"https://google.com"), RouteDecision::WebKitLegacy);
}

#[test]
fn test_https_github_legacy() {
    assert_eq!(route_url(b"https://github.com/aieonyx"), RouteDecision::WebKitLegacy);
}

#[test]
fn test_http_blocked_always() {
    assert_eq!(route_url(b"http://insecure.example.com"), RouteDecision::Block);
}

#[test]
fn test_http_localhost_blocked() {
    // Even localhost HTTP is blocked — cleartext policy is absolute.
    assert_eq!(route_url(b"http://localhost:8080"), RouteDecision::Block);
}

#[test]
fn test_awp_is_not_legacy() {
    assert_ne!(route_url(b"awp://sovereign"), RouteDecision::WebKitLegacy);
}

#[test]
fn test_https_is_not_haniel() {
    assert_ne!(route_url(b"https://example.com"), RouteDecision::Haniel);
}

// ── Render surface integration ────────────────────────────────────────────────

#[test]
fn test_surface_is_standard_dimensions() {
    let s = RenderSurface::new();
    assert_eq!(s.width(), 1280);
    assert_eq!(s.height(), 720);
}

#[test]
fn test_surface_pixel_buffer_size() {
    let s = RenderSurface::new();
    assert_eq!(s.pixels().len(), SURFACE_BYTES / 4);
}

#[test]
fn test_render_frame_lifecycle() {
    let mut s = RenderSurface::new();

    // Frame 1: clear to sovereign midnight, draw one pixel, commit
    s.clear(0xFF_0A_0F_1A);
    s.put_pixel(640, 360, 0xFF_00_C8_96); // teal centre pixel
    assert_eq!(s.get_pixel(640, 360), 0xFF_00_C8_96);
    let fc = s.commit();
    assert_eq!(fc, 1);

    // Frame 2: independent budget
    assert_eq!(s.budget_remaining(), 1000);
}

#[test]
fn test_multiple_frames_increment_count() {
    let mut s = RenderSurface::new();
    for i in 1..=10 {
        assert_eq!(s.commit(), i);
    }
}

#[test]
fn test_budget_across_frame_boundary() {
    let mut s = RenderSurface::new();
    s.spend_budget(999).unwrap();
    assert_eq!(s.budget_remaining(), 1);
    s.commit();
    assert_eq!(s.budget_remaining(), 1000); // reset
}

#[test]
fn test_budget_exact_spend() {
    let mut s = RenderSurface::new();
    let r = s.spend_budget(1000).unwrap();
    assert_eq!(r, 0);
}

#[test]
fn test_budget_over_spend_rejected() {
    let mut s = RenderSurface::new();
    assert_eq!(s.spend_budget(1001), Err(HanielError::BudgetExceeded));
}

#[test]
fn test_clear_then_pixel_override() {
    let mut s = RenderSurface::new();
    s.clear(0xFFFFFFFF);
    s.put_pixel(0, 0, 0x00000000);
    assert_eq!(s.get_pixel(0, 0), 0x00000000);
    assert_eq!(s.get_pixel(1, 0), 0xFFFFFFFF);
}

#[test]
fn test_corner_pixels() {
    let mut s = RenderSurface::new();
    s.put_pixel(0, 0, 0x11111111);
    s.put_pixel(1279, 0, 0x22222222);
    s.put_pixel(0, 719, 0x33333333);
    s.put_pixel(1279, 719, 0x44444444);
    assert_eq!(s.get_pixel(0, 0),      0x11111111);
    assert_eq!(s.get_pixel(1279, 0),   0x22222222);
    assert_eq!(s.get_pixel(0, 719),    0x33333333);
    assert_eq!(s.get_pixel(1279, 719), 0x44444444);
}

// ── Capability gate integration ───────────────────────────────────────────────

#[test]
fn test_renderer_has_display_cap() {
    assert!(cap_granted(HanielCap::DisplaySurface));
    assert!(assert_cap(HanielCap::DisplaySurface).is_ok());
}

#[test]
fn test_renderer_has_font_cap() {
    assert!(cap_granted(HanielCap::FontRead));
    assert!(assert_cap(HanielCap::FontRead).is_ok());
}

#[test]
fn test_renderer_network_denied() {
    assert!(!cap_granted(HanielCap::Network));
    assert_eq!(assert_cap(HanielCap::Network), Err(HanielError::CapabilityDenied));
}

#[test]
fn test_renderer_storage_write_denied() {
    assert!(!cap_granted(HanielCap::StorageWrite));
    assert_eq!(assert_cap(HanielCap::StorageWrite), Err(HanielError::CapabilityDenied));
}

// ── Sovereign proof integration ───────────────────────────────────────────────

#[test]
fn test_axon_proof_value() {
    assert_eq!(AXON_PROOF, 0x4153);
    assert!(verify_sovereign_proof(AXON_PROOF));
}

#[test]
fn test_sovereign_proof_off_by_one_fails() {
    assert!(!verify_sovereign_proof(AXON_PROOF - 1));
    assert!(!verify_sovereign_proof(AXON_PROOF + 1));
}

#[test]
fn test_pd_id_is_in_optional_range() {
    // Optional PD IDs are >= 0x10 (established in M1 sovereignty spec)
    assert!(HANIEL_PD_ID >= 0x10);
}

#[test]
fn test_awp_scheme_bytes() {
    assert_eq!(AWP_SCHEME, b"awp://");
    assert_eq!(HTTPS_SCHEME, b"https://");
}

#[test]
fn test_surface_bytes_is_correct() {
    assert_eq!(SURFACE_BYTES, 1280 * 720 * 4);
    assert_eq!(SURFACE_WIDTH * SURFACE_HEIGHT * 4, SURFACE_BYTES as u32);
}
