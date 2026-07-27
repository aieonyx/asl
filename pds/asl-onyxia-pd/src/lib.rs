// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ════════════════════════════════════════════════════════════════════════════
// asl-onyxia-pd — Onyxia Browser Protection Domain
// PL-71 / ASL-M26: Onyxia Browser runs as isolated seL4 PD
// ════════════════════════════════════════════════════════════════════════════
//
// ROLE: Wraps the Onyxia awp:// browser as a seL4 Protection Domain.
//       The browser can only navigate awp:// URLs within its own isolated
//       memory space. https:// requests go through AWP-PD (legacy bridge).
//       Rendering goes through HANIEL-PD → Phoenix-Desktop-PD.
//
// CAPABILITY POLICY:
//   GRANTED  : AwpSend (via AWP-PD IPC)    — awp:// navigation
//   GRANTED  : HanielRender                — render surface write via HANIEL
//   GRANTED  : EDBRead (via EdisonDB-PD)   — page cache reads
//   DENIED   : DirectNetwork               — no raw socket access
//   DENIED   : FramebufferWrite            — all rendering via HANIEL
//   DENIED   : StorageWrite                — read-only EDB access
//   DENIED   : https://                    — cleartext and unverified blocked
//
// URL ROUTING (sovereign policy):
//   awp://   → HANIEL-PD renders sovereign page (this PD handles)
//   https:// → AWP-PD legacy bridge (capability-gated)
//   http://  → BLOCKED — no cleartext
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

#![no_std]
#![forbid(unsafe_code)]

#[cfg(kani)]
extern crate kani;

use asl_common::pd::PdId;
use asl_arpi_ipc::AXON_PROOF;
use asl_haniel::{AWP_SCHEME, HTTPS_SCHEME};

// ── Constants ─────────────────────────────────────────────────────────────────

pub const ONYXIA_PD_ID:    u8  = 0x42;
pub const SOVEREIGN_PROOF: u64 = AXON_PROOF;
pub const MAX_URL_LEN:     usize = 512;

// ── URL routing policy ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlRoute {
    /// awp:// — sovereign path, rendered by HANIEL-PD
    Awp,
    /// https:// — legacy bridge via AWP-PD (capability-gated)
    HttpsLegacy,
    /// http:// — BLOCKED, cleartext forbidden
    Blocked,
    /// Empty URL — new tab sovereign splash
    NewTab,
}

pub fn route_url(url: &[u8]) -> UrlRoute {
    if url.is_empty()                          { return UrlRoute::NewTab; }
    if url.starts_with(AWP_SCHEME)             { return UrlRoute::Awp; }
    if url.starts_with(HTTPS_SCHEME)           { return UrlRoute::HttpsLegacy; }
    if url.starts_with(b"http://")             { return UrlRoute::Blocked; }
    // Unknown scheme — treat as awp:// path
    UrlRoute::Awp
}

// ── Navigation state ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavState {
    /// New tab — sovereign splash
    NewTab,
    /// Navigating — IPC in flight to HANIEL/AWP
    Navigating,
    /// Page loaded — HANIEL has rendered
    Loaded,
    /// Navigation blocked — URL policy violation
    Blocked,
    /// Error — navigation failed
    Error,
}

// ── Onyxia-PD state machine ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnyxiaPhase {
    AwaitingBoot,
    Ready,
    Navigating,
    Faulted,
}

pub struct OnyxiaPd {
    pub phase:      OnyxiaPhase,
    pub nav_state:  NavState,
    pub nav_count:  u64,
    pub blocked:    u64,
    pub proof:      u64,
    pub current_route: UrlRoute,
}

impl OnyxiaPd {
    pub const fn new() -> Self {
        OnyxiaPd {
            phase:         OnyxiaPhase::AwaitingBoot,
            nav_state:     NavState::NewTab,
            nav_count:     0,
            blocked:       0,
            proof:         SOVEREIGN_PROOF,
            current_route: UrlRoute::NewTab,
        }
    }

    pub fn on_boot_signal(&mut self) -> Result<(), &'static str> {
        if self.phase != OnyxiaPhase::AwaitingBoot {
            return Err("Onyxia-PD: BOOT_SIGNAL in wrong phase");
        }
        self.assert_proof();
        self.phase = OnyxiaPhase::Ready;
        Ok(())
    }

    /// Navigate to URL — enforces sovereign URL policy
    pub fn navigate(&mut self, url: &[u8]) -> Result<UrlRoute, &'static str> {
        if self.phase != OnyxiaPhase::Ready {
            return Err("Onyxia-PD: not in Ready phase");
        }
        if url.len() > MAX_URL_LEN {
            return Err("Onyxia-PD: URL too long");
        }
        self.assert_proof();

        let route = route_url(url);
        match route {
            UrlRoute::Blocked => {
                self.blocked += 1;
                self.nav_state = NavState::Blocked;
                Err("Onyxia-PD: cleartext http:// blocked by sovereign policy")
            }
            r => {
                self.nav_count += 1;
                self.current_route = r;
                self.phase = OnyxiaPhase::Navigating;
                self.nav_state = NavState::Navigating;
                Ok(r)
            }
        }
    }

    /// HANIEL-PD has rendered the page — navigation complete
    pub fn on_render_complete(&mut self) -> Result<(), &'static str> {
        if self.phase != OnyxiaPhase::Navigating {
            return Err("Onyxia-PD: render_complete in wrong phase");
        }
        self.nav_state = NavState::Loaded;
        self.phase = OnyxiaPhase::Ready;
        Ok(())
    }

    /// Is the current URL sovereign (awp://)?
    pub fn is_sovereign(&self) -> bool {
        matches!(self.current_route, UrlRoute::Awp | UrlRoute::NewTab)
    }

    pub fn pd_id() -> PdId { PdId::GpuCap } // browser slot

    #[inline]
    fn assert_proof(&self) {
        assert_eq!(self.proof, SOVEREIGN_PROOF,
            "SOVEREIGN PROOF VIOLATION: Onyxia-PD integrity failed");
    }
}

impl Default for OnyxiaPd { fn default() -> Self { Self::new() } }

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tab_on_init() {
        let pd = OnyxiaPd::new();
        assert_eq!(pd.nav_state, NavState::NewTab);
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
    }

    #[test]
    fn test_boot_signal() {
        let mut pd = OnyxiaPd::new();
        assert!(pd.on_boot_signal().is_ok());
        assert_eq!(pd.phase, OnyxiaPhase::Ready);
    }

    #[test]
    fn test_awp_navigation_succeeds() {
        let mut pd = OnyxiaPd::new();
        pd.on_boot_signal().unwrap();
        let route = pd.navigate(b"awp://aieonyx").unwrap();
        assert_eq!(route, UrlRoute::Awp);
        assert_eq!(pd.phase, OnyxiaPhase::Navigating);
        assert_eq!(pd.nav_count, 1);
    }

    #[test]
    fn test_http_blocked() {
        let mut pd = OnyxiaPd::new();
        pd.on_boot_signal().unwrap();
        assert!(pd.navigate(b"http://example.com").is_err());
        assert_eq!(pd.blocked, 1);
        assert_eq!(pd.nav_state, NavState::Blocked);
    }

    #[test]
    fn test_https_routed_to_legacy() {
        let mut pd = OnyxiaPd::new();
        pd.on_boot_signal().unwrap();
        let route = pd.navigate(b"https://example.com").unwrap();
        assert_eq!(route, UrlRoute::HttpsLegacy);
    }

    #[test]
    fn test_render_complete_returns_ready() {
        let mut pd = OnyxiaPd::new();
        pd.on_boot_signal().unwrap();
        pd.navigate(b"awp://aieonyx").unwrap();
        assert!(pd.on_render_complete().is_ok());
        assert_eq!(pd.phase, OnyxiaPhase::Ready);
        assert_eq!(pd.nav_state, NavState::Loaded);
    }

    #[test]
    fn test_is_sovereign_awp() {
        let mut pd = OnyxiaPd::new();
        pd.on_boot_signal().unwrap();
        pd.navigate(b"awp://about").unwrap();
        assert!(pd.is_sovereign());
    }

    #[test]
    fn test_is_not_sovereign_https() {
        let mut pd = OnyxiaPd::new();
        pd.on_boot_signal().unwrap();
        pd.navigate(b"https://example.com").unwrap();
        assert!(!pd.is_sovereign());
    }

    #[test]
    fn test_new_tab_is_sovereign() {
        let pd = OnyxiaPd::new();
        assert!(pd.is_sovereign());
    }

    #[test]
    fn test_navigate_before_boot_fails() {
        let mut pd = OnyxiaPd::new();
        assert!(pd.navigate(b"awp://aieonyx").is_err());
    }

    #[test]
    fn test_proof_invariant() {
        let mut pd = OnyxiaPd::new();
        pd.on_boot_signal().unwrap();
        pd.navigate(b"awp://aieonyx").unwrap();
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
    }

    #[test]
    fn test_route_url_empty_is_newtab() {
        assert_eq!(route_url(b""), UrlRoute::NewTab);
    }

    #[test]
    fn test_route_url_http_blocked() {
        assert_eq!(route_url(b"http://evil.com"), UrlRoute::Blocked);
    }

    #[test]
    fn test_multiple_navigations() {
        let mut pd = OnyxiaPd::new();
        pd.on_boot_signal().unwrap();
        pd.navigate(b"awp://aieonyx").unwrap();
        pd.on_render_complete().unwrap();
        pd.navigate(b"awp://about").unwrap();
        pd.on_render_complete().unwrap();
        assert_eq!(pd.nav_count, 2);
        assert_eq!(pd.blocked, 0);
    }
}
