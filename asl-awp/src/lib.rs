// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// asl-awp — AWP Protocol Protection Domain
//
// AIEONYX Web Protocol (AWP) — the sovereign nervous system.
// Handles awp:// URL resolution, packet framing, node routing,
// and Aegis mesh integration inside seL4.
//
// Protocol stack (M20 scope):
//   Layer 1 — Frame         : AWP packet framing (magic + version + flags)
//   Layer 2 — Address       : Sovereign node addressing (awp://name.category)
//   Layer 3 — Route         : Mesh routing table (node → channel)
//   Layer 4 — Dispatch      : IPC dispatch to HANIEL PD or local handler
//   Layer 5 — Threat gate   : Aegis threat score check before any dispatch
//
// Sovereign proof: axon_main() → 0x4153 (invariant)
// AWP magic:       0xA1E0AE70 (established M8)
//
// S4+i: AWP is the nervous system — fast, sovereign, no cleartext.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(kani)]
extern crate kani;

extern crate alloc;

use alloc::vec::Vec;

// ── Constants ─────────────────────────────────────────────────────────────────

/// AWP packet magic number (established M8).
pub const AWP_MAGIC: u32 = 0xA1E0AE70;

/// AWP protocol version — M20.
pub const AWP_VERSION: u8 = 0x01;

/// AWP URL scheme prefix.
pub const AWP_SCHEME: &[u8] = b"awp://";

/// AWP scheme length in bytes.
pub const AWP_SCHEME_LEN: usize = 6;

/// Maximum AWP address length (name.category or name.category.region).
pub const AWP_ADDR_MAX: usize = 64;

/// Maximum AWP payload size per packet (bytes).
pub const AWP_PAYLOAD_MAX: usize = 1400;

/// Sovereign proof value — invariant across all milestones.
pub const AXON_PROOF: u64 = 0x4153;

/// AWP PD identifier.
pub const AWP_PD_ID: u8 = 0x30;

/// Aegis threat score threshold — packets above this score are rejected.
pub const AEGIS_THREAT_THRESHOLD: u8 = 80;

/// AWP channel assignments (seL4 IPC channels).
pub const CH_AEGIS_MESH: u8 = 1;
pub const CH_AWP_DISPATCH: u8 = 2;
pub const CH_THREAT_INTEL: u8 = 3;
pub const CH_HANIEL: u8 = 4;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AwpError {
    /// Packet magic number invalid.
    InvalidMagic,
    /// Protocol version mismatch.
    VersionMismatch,
    /// Address format invalid.
    InvalidAddress,
    /// Payload exceeds maximum size.
    PayloadTooLarge,
    /// No route found for destination.
    NoRoute,
    /// Aegis threat score too high — packet rejected.
    ThreatRejected,
    /// URL is not an AWP URL.
    NotAwp,
    /// Routing table full.
    RoutingTableFull,
    /// Invalid input.
    InvalidInput,
}

// ── AWP packet frame ──────────────────────────────────────────────────────────

/// AWP packet flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwpFlags(pub u8);

impl AwpFlags {
    /// Request packet (client → node).
    pub const REQUEST: AwpFlags = AwpFlags(0x01);
    /// Response packet (node → client).
    pub const RESPONSE: AwpFlags = AwpFlags(0x02);
    /// Mesh routing packet (node → node).
    pub const MESH: AwpFlags = AwpFlags(0x04);
    /// Threat intel packet.
    pub const THREAT: AwpFlags = AwpFlags(0x08);

    pub fn contains(&self, other: AwpFlags) -> bool {
        self.0 & other.0 != 0
    }
}

/// AWP packet header — fixed-size, no heap allocation.
///
/// Wire format (16 bytes):
///   [0..4]  magic    u32 BE  — 0xA1E0AE70
///   [4]     version  u8      — 0x01
///   [5]     flags    u8      — AwpFlags
///   [6..8]  seq      u16 BE  — monotonic sequence number
///   [8..12] src_id   u32 BE  — source node ID
///   [12..16] dst_id  u32 BE  — destination node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwpHeader {
    pub magic:   u32,
    pub version: u8,
    pub flags:   AwpFlags,
    pub seq:     u16,
    pub src_id:  u32,
    pub dst_id:  u32,
}

impl AwpHeader {
    pub const WIRE_LEN: usize = 16;

    /// Create a new AWP request header.
    pub fn new_request(seq: u16, src_id: u32, dst_id: u32) -> Self {
        Self {
            magic:   AWP_MAGIC,
            version: AWP_VERSION,
            flags:   AwpFlags::REQUEST,
            seq,
            src_id,
            dst_id,
        }
    }

    /// Create a new AWP response header.
    pub fn new_response(seq: u16, src_id: u32, dst_id: u32) -> Self {
        Self {
            magic:   AWP_MAGIC,
            version: AWP_VERSION,
            flags:   AwpFlags::RESPONSE,
            seq,
            src_id,
            dst_id,
        }
    }

    /// Validate the header.
    pub fn validate(&self) -> Result<(), AwpError> {
        if self.magic != AWP_MAGIC {
            return Err(AwpError::InvalidMagic);
        }
        if self.version != AWP_VERSION {
            return Err(AwpError::VersionMismatch);
        }
        Ok(())
    }

    /// Serialise to 16-byte wire format.
    pub fn to_bytes(&self) -> [u8; Self::WIRE_LEN] {
        let mut b = [0u8; Self::WIRE_LEN];
        b[0..4].copy_from_slice(&self.magic.to_be_bytes());
        b[4] = self.version;
        b[5] = self.flags.0;
        b[6..8].copy_from_slice(&self.seq.to_be_bytes());
        b[8..12].copy_from_slice(&self.src_id.to_be_bytes());
        b[12..16].copy_from_slice(&self.dst_id.to_be_bytes());
        b
    }

    /// Deserialise from 16-byte wire format.
    pub fn from_bytes(b: &[u8; Self::WIRE_LEN]) -> Self {
        let magic   = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        let version = b[4];
        let flags   = AwpFlags(b[5]);
        let seq     = u16::from_be_bytes([b[6], b[7]]);
        let src_id  = u32::from_be_bytes([b[8], b[9], b[10], b[11]]);
        let dst_id  = u32::from_be_bytes([b[12], b[13], b[14], b[15]]);
        Self { magic, version, flags, seq, src_id, dst_id }
    }
}

// ── AWP address ───────────────────────────────────────────────────────────────

/// Validate an AWP URL — must start with `awp://` and have a non-empty address.
pub fn validate_awp_url(url: &[u8]) -> Result<&[u8], AwpError> {
    if url.len() < AWP_SCHEME_LEN + 1 {
        return Err(AwpError::InvalidAddress);
    }
    if !url.starts_with(AWP_SCHEME) {
        return Err(AwpError::NotAwp);
    }
    let addr = &url[AWP_SCHEME_LEN..];
    if addr.is_empty() || addr.len() > AWP_ADDR_MAX {
        return Err(AwpError::InvalidAddress);
    }
    Ok(addr)
}

/// Parse AWP address into (name, category, optional region).
///
/// Format: `name.category` or `name.category.region`
/// Example: `aegis.mesh` → ("aegis", "mesh", None)
/// Example: `josebank.bank.ph` → ("josebank", "bank", Some("ph"))
pub fn parse_awp_addr(addr: &[u8]) -> Result<AwpAddr, AwpError> {
    let mut parts = addr.splitn(3, |&b| b == b'.');
    let name = parts.next().ok_or(AwpError::InvalidAddress)?;
    let category = parts.next().ok_or(AwpError::InvalidAddress)?;
    let region = parts.next();

    if name.is_empty() || category.is_empty() {
        return Err(AwpError::InvalidAddress);
    }

    Ok(AwpAddr { name, category, region })
}

/// Parsed AWP address components.
#[derive(Debug, PartialEq, Eq)]
pub struct AwpAddr<'a> {
    pub name:     &'a [u8],
    pub category: &'a [u8],
    pub region:   Option<&'a [u8]>,
}

// ── Routing table ─────────────────────────────────────────────────────────────

/// Maximum number of routes in the AWP routing table.
pub const AWP_ROUTE_MAX: usize = 16;

/// A single AWP route entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwpRoute {
    /// Destination node ID.
    pub dst_id:  u32,
    /// seL4 IPC channel to reach this node.
    pub channel: u8,
    /// Aegis trust score for this route (0=untrusted, 100=sovereign).
    pub trust:   u8,
    /// Route is active.
    pub active:  bool,
}

/// AWP routing table — fixed-size, no heap.
pub struct AwpRouter {
    routes: [Option<AwpRoute>; AWP_ROUTE_MAX],
    count:  usize,
    /// Packet sequence counter.
    seq:    u16,
}

impl AwpRouter {
    pub const fn new() -> Self {
        Self {
            routes: [None; AWP_ROUTE_MAX],
            count:  0,
            seq:    0,
        }
    }

    /// Register a route to a destination node.
    pub fn add_route(&mut self, route: AwpRoute) -> Result<(), AwpError> {
        if self.count >= AWP_ROUTE_MAX {
            return Err(AwpError::RoutingTableFull);
        }
        // Check for duplicate dst_id
        for slot in self.routes.iter() {
            if let Some(r) = slot {
                if r.dst_id == route.dst_id {
                    return Err(AwpError::InvalidInput);
                }
            }
        }
        for slot in self.routes.iter_mut() {
            if slot.is_none() {
                *slot = Some(route);
                self.count += 1;
                return Ok(());
            }
        }
        Err(AwpError::RoutingTableFull)
    }

    /// Look up a route by destination node ID.
    pub fn lookup(&self, dst_id: u32) -> Option<&AwpRoute> {
        for slot in self.routes.iter() {
            if let Some(r) = slot {
                if r.dst_id == dst_id && r.active {
                    return Some(r);
                }
            }
        }
        None
    }

    /// Remove a route by destination node ID.
    pub fn remove_route(&mut self, dst_id: u32) -> bool {
        for slot in self.routes.iter_mut() {
            if let Some(r) = slot {
                if r.dst_id == dst_id {
                    *slot = None;
                    self.count -= 1;
                    return true;
                }
            }
        }
        false
    }

    /// Return current route count.
    pub fn count(&self) -> usize { self.count }

    /// Advance sequence counter and return new value.
    pub fn next_seq(&mut self) -> u16 {
        self.seq = self.seq.wrapping_add(1);
        self.seq
    }

    /// Current sequence value.
    pub fn seq(&self) -> u16 { self.seq }
}

impl Default for AwpRouter {
    fn default() -> Self { Self::new() }
}

// ── Aegis threat gate ─────────────────────────────────────────────────────────

/// Check a packet against the Aegis threat threshold.
/// Returns Ok if safe to dispatch, Err(ThreatRejected) if threat score too high.
pub fn aegis_threat_gate(threat_score: u8) -> Result<(), AwpError> {
    if threat_score >= AEGIS_THREAT_THRESHOLD {
        Err(AwpError::ThreatRejected)
    } else {
        Ok(())
    }
}

// ── Packet dispatcher ─────────────────────────────────────────────────────────

/// AWP dispatch result.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DispatchResult {
    /// Dispatched to HANIEL PD for rendering.
    ToHaniel,
    /// Dispatched to local mesh handler.
    ToMesh,
    /// Dispatched to threat intel channel.
    ToThreatIntel,
    /// Packet rejected by threat gate.
    Rejected,
}

/// Dispatch an AWP packet based on its flags and threat score.
pub fn dispatch(header: &AwpHeader, threat_score: u8) -> Result<DispatchResult, AwpError> {
    header.validate()?;

    // Threat gate — check before any dispatch
    if aegis_threat_gate(threat_score).is_err() {
        return Ok(DispatchResult::Rejected);
    }

    if header.flags.contains(AwpFlags::THREAT) {
        return Ok(DispatchResult::ToThreatIntel);
    }
    if header.flags.contains(AwpFlags::MESH) {
        return Ok(DispatchResult::ToMesh);
    }
    // REQUEST or RESPONSE → HANIEL for rendering
    Ok(DispatchResult::ToHaniel)
}

// ── Sovereign proof ───────────────────────────────────────────────────────────

#[inline]
pub fn verify_sovereign_proof(proof: u64) -> bool {
    proof == AXON_PROOF
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Header tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_header_magic_constant() {
        assert_eq!(AWP_MAGIC, 0xA1E0AE70);
    }

    #[test]
    fn test_header_new_request() {
        let h = AwpHeader::new_request(1, 0x01, 0x02);
        assert_eq!(h.magic, AWP_MAGIC);
        assert_eq!(h.version, AWP_VERSION);
        assert_eq!(h.seq, 1);
        assert_eq!(h.src_id, 0x01);
        assert_eq!(h.dst_id, 0x02);
        assert!(h.flags.contains(AwpFlags::REQUEST));
    }

    #[test]
    fn test_header_new_response() {
        let h = AwpHeader::new_response(2, 0x02, 0x01);
        assert!(h.flags.contains(AwpFlags::RESPONSE));
    }

    #[test]
    fn test_header_serialise_roundtrip() {
        let h = AwpHeader::new_request(42, 0xDEAD, 0xBEEF);
        let bytes = h.to_bytes();
        let h2 = AwpHeader::from_bytes(&bytes);
        assert_eq!(h, h2);
    }

    #[test]
    fn test_header_magic_in_wire_bytes() {
        let h = AwpHeader::new_request(1, 1, 2);
        let b = h.to_bytes();
        assert_eq!(&b[0..4], &[0xA1, 0xE0, 0xAE, 0x70]);
    }

    #[test]
    fn test_header_validate_ok() {
        let h = AwpHeader::new_request(1, 1, 2);
        assert!(h.validate().is_ok());
    }

    #[test]
    fn test_header_validate_bad_magic() {
        let mut h = AwpHeader::new_request(1, 1, 2);
        h.magic = 0xDEADBEEF;
        assert_eq!(h.validate(), Err(AwpError::InvalidMagic));
    }

    #[test]
    fn test_header_validate_bad_version() {
        let mut h = AwpHeader::new_request(1, 1, 2);
        h.version = 0xFF;
        assert_eq!(h.validate(), Err(AwpError::VersionMismatch));
    }

    #[test]
    fn test_header_wire_len() {
        assert_eq!(AwpHeader::WIRE_LEN, 16);
    }

    // ── URL / address tests ───────────────────────────────────────────────────

    #[test]
    fn test_validate_awp_url_ok() {
        assert!(validate_awp_url(b"awp://aegis.mesh").is_ok());
    }

    #[test]
    fn test_validate_awp_url_returns_addr() {
        let addr = validate_awp_url(b"awp://aegis.mesh").unwrap();
        assert_eq!(addr, b"aegis.mesh");
    }

    #[test]
    fn test_validate_awp_url_not_awp() {
        assert_eq!(validate_awp_url(b"https://example.com"), Err(AwpError::NotAwp));
    }

    #[test]
    fn test_validate_awp_url_too_short() {
        assert_eq!(validate_awp_url(b"awp://"), Err(AwpError::InvalidAddress));
    }

    #[test]
    fn test_parse_addr_two_part() {
        let addr = parse_awp_addr(b"aegis.mesh").unwrap();
        assert_eq!(addr.name, b"aegis");
        assert_eq!(addr.category, b"mesh");
        assert!(addr.region.is_none());
    }

    #[test]
    fn test_parse_addr_three_part() {
        let addr = parse_awp_addr(b"josebank.bank.ph").unwrap();
        assert_eq!(addr.name, b"josebank");
        assert_eq!(addr.category, b"bank");
        assert_eq!(addr.region, Some(b"ph".as_ref()));
    }

    #[test]
    fn test_parse_addr_empty_name_rejected() {
        assert_eq!(parse_awp_addr(b".mesh"), Err(AwpError::InvalidAddress));
    }

    #[test]
    fn test_parse_addr_no_dot_rejected() {
        assert_eq!(parse_awp_addr(b"nodot"), Err(AwpError::InvalidAddress));
    }

    // ── Router tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_router_starts_empty() {
        let r = AwpRouter::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_router_add_and_lookup() {
        let mut r = AwpRouter::new();
        let route = AwpRoute { dst_id: 0x01, channel: CH_AWP_DISPATCH, trust: 90, active: true };
        r.add_route(route).unwrap();
        let found = r.lookup(0x01).unwrap();
        assert_eq!(found.dst_id, 0x01);
        assert_eq!(found.channel, CH_AWP_DISPATCH);
    }

    #[test]
    fn test_router_lookup_missing() {
        let r = AwpRouter::new();
        assert!(r.lookup(0xFF).is_none());
    }

    #[test]
    fn test_router_duplicate_rejected() {
        let mut r = AwpRouter::new();
        let route = AwpRoute { dst_id: 0x01, channel: 1, trust: 90, active: true };
        r.add_route(route).unwrap();
        assert!(r.add_route(route).is_err());
    }

    #[test]
    fn test_router_remove_route() {
        let mut r = AwpRouter::new();
        let route = AwpRoute { dst_id: 0x01, channel: 1, trust: 90, active: true };
        r.add_route(route).unwrap();
        assert!(r.remove_route(0x01));
        assert_eq!(r.count(), 0);
        assert!(r.lookup(0x01).is_none());
    }

    #[test]
    fn test_router_inactive_route_not_found() {
        let mut r = AwpRouter::new();
        let route = AwpRoute { dst_id: 0x01, channel: 1, trust: 90, active: false };
        r.add_route(route).unwrap();
        assert!(r.lookup(0x01).is_none());
    }

    #[test]
    fn test_router_seq_increments() {
        let mut r = AwpRouter::new();
        assert_eq!(r.seq(), 0);
        assert_eq!(r.next_seq(), 1);
        assert_eq!(r.next_seq(), 2);
    }

    #[test]
    fn test_router_seq_wraps() {
        let mut r = AwpRouter::new();
        r.seq = u16::MAX;
        assert_eq!(r.next_seq(), 0);
    }

    // ── Threat gate tests ─────────────────────────────────────────────────────

    #[test]
    fn test_threat_gate_safe() {
        assert!(aegis_threat_gate(0).is_ok());
        assert!(aegis_threat_gate(79).is_ok());
    }

    #[test]
    fn test_threat_gate_threshold() {
        assert_eq!(aegis_threat_gate(80), Err(AwpError::ThreatRejected));
        assert_eq!(aegis_threat_gate(100), Err(AwpError::ThreatRejected));
    }

    #[test]
    fn test_threat_threshold_constant() {
        assert_eq!(AEGIS_THREAT_THRESHOLD, 80);
    }

    // ── Dispatch tests ────────────────────────────────────────────────────────

    #[test]
    fn test_dispatch_request_to_haniel() {
        let h = AwpHeader::new_request(1, 1, 2);
        assert_eq!(dispatch(&h, 0).unwrap(), DispatchResult::ToHaniel);
    }

    #[test]
    fn test_dispatch_response_to_haniel() {
        let h = AwpHeader::new_response(1, 2, 1);
        assert_eq!(dispatch(&h, 0).unwrap(), DispatchResult::ToHaniel);
    }

    #[test]
    fn test_dispatch_mesh_packet() {
        let mut h = AwpHeader::new_request(1, 1, 2);
        h.flags = AwpFlags::MESH;
        assert_eq!(dispatch(&h, 0).unwrap(), DispatchResult::ToMesh);
    }

    #[test]
    fn test_dispatch_threat_packet() {
        let mut h = AwpHeader::new_request(1, 1, 2);
        h.flags = AwpFlags::THREAT;
        assert_eq!(dispatch(&h, 0).unwrap(), DispatchResult::ToThreatIntel);
    }

    #[test]
    fn test_dispatch_high_threat_rejected() {
        let h = AwpHeader::new_request(1, 1, 2);
        assert_eq!(dispatch(&h, 80).unwrap(), DispatchResult::Rejected);
        assert_eq!(dispatch(&h, 100).unwrap(), DispatchResult::Rejected);
    }

    #[test]
    fn test_dispatch_invalid_magic_errors() {
        let mut h = AwpHeader::new_request(1, 1, 2);
        h.magic = 0x00000000;
        assert_eq!(dispatch(&h, 0), Err(AwpError::InvalidMagic));
    }

    // ── Sovereign proof ───────────────────────────────────────────────────────

    #[test]
    fn test_sovereign_proof() {
        assert!(verify_sovereign_proof(0x4153));
        assert!(!verify_sovereign_proof(0x0000));
    }

    #[test]
    fn test_axon_proof_constant() {
        assert_eq!(AXON_PROOF, 0x4153);
    }

    #[test]
    fn test_awp_pd_id_in_optional_range() {
        assert!(AWP_PD_ID >= 0x10);
    }
}
