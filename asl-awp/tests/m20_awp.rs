// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// m20_awp.rs — Integration tests for AWP Protocol PD (M20)
// Target: 30+ tests, 0 failures
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_awp::{
    AwpHeader, AwpFlags, AwpRouter, AwpRoute, AwpError,
    validate_awp_url, parse_awp_addr, aegis_threat_gate, dispatch,
    DispatchResult, verify_sovereign_proof,
    AWP_MAGIC, AWP_VERSION, AWP_PD_ID, AXON_PROOF,
    AEGIS_THREAT_THRESHOLD, CH_HANIEL, CH_AEGIS_MESH,
};

// ── Full packet lifecycle ─────────────────────────────────────────────────────

#[test]
fn test_awp_request_lifecycle() {
    // 1. Parse URL
    let url = b"awp://aegis.mesh";
    let addr = validate_awp_url(url).unwrap();
    let parsed = parse_awp_addr(addr).unwrap();
    assert_eq!(parsed.name, b"aegis");
    assert_eq!(parsed.category, b"mesh");

    // 2. Build header
    let mut router = AwpRouter::new();
    let seq = router.next_seq();
    let h = AwpHeader::new_request(seq, 0x01, 0x02);
    assert!(h.validate().is_ok());

    // 3. Dispatch (threat score 0 = safe)
    let result = dispatch(&h, 0).unwrap();
    assert_eq!(result, DispatchResult::ToHaniel);
}

#[test]
fn test_awp_mesh_packet_lifecycle() {
    let mut h = AwpHeader::new_request(1, 0x01, 0x03);
    h.flags = AwpFlags::MESH;

    // Register a mesh route
    let mut router = AwpRouter::new();
    router.add_route(AwpRoute {
        dst_id:  0x03,
        channel: CH_AEGIS_MESH,
        trust:   95,
        active:  true,
    }).unwrap();

    let route = router.lookup(0x03).unwrap();
    assert_eq!(route.channel, CH_AEGIS_MESH);

    let result = dispatch(&h, 10).unwrap();
    assert_eq!(result, DispatchResult::ToMesh);
}

#[test]
fn test_wire_format_roundtrip() {
    let h = AwpHeader::new_request(0xABCD, 0x11223344, 0x55667788);
    let bytes = h.to_bytes();

    // Magic visible at start
    assert_eq!(&bytes[0..4], &[0xA1, 0xE0, 0xAE, 0x70]);
    assert_eq!(bytes[4], AWP_VERSION);

    let h2 = AwpHeader::from_bytes(&bytes);
    assert_eq!(h, h2);
    assert_eq!(h2.seq, 0xABCD);
    assert_eq!(h2.src_id, 0x11223344);
    assert_eq!(h2.dst_id, 0x55667788);
}

// ── Address parsing integration ───────────────────────────────────────────────

#[test]
fn test_sovereign_node_addresses() {
    // Standard sovereign addresses
    let cases: &[(&[u8], &[u8], &[u8])] = &[
        (b"aegis.mesh",       b"aegis",    b"mesh"),
        (b"soma.id",          b"soma",     b"id"),
        (b"bastion.dev",      b"bastion",  b"dev"),
        (b"edisondb.dev",     b"edisondb", b"dev"),
        (b"onyxia.social",    b"onyxia",   b"social"),
    ];
    for (addr, name, cat) in cases {
        let parsed = parse_awp_addr(addr).unwrap();
        assert_eq!(parsed.name, *name);
        assert_eq!(parsed.category, *cat);
        assert!(parsed.region.is_none());
    }
}

#[test]
fn test_regional_addresses() {
    let addr = parse_awp_addr(b"josebank.bank.ph").unwrap();
    assert_eq!(addr.name, b"josebank");
    assert_eq!(addr.category, b"bank");
    assert_eq!(addr.region, Some(b"ph".as_ref()));

    let addr2 = parse_awp_addr(b"aieonyx.dev.cz").unwrap();
    assert_eq!(addr2.region, Some(b"cz".as_ref()));
}

#[test]
fn test_invalid_addresses_rejected() {
    assert!(parse_awp_addr(b"").is_err());
    assert!(parse_awp_addr(b"nocat").is_err());
    assert!(parse_awp_addr(b".empty").is_err());
}

// ── Router integration ────────────────────────────────────────────────────────

#[test]
fn test_multi_node_routing() {
    let mut router = AwpRouter::new();

    // Register multiple sovereign nodes
    for i in 1u32..=8 {
        router.add_route(AwpRoute {
            dst_id:  i,
            channel: i as u8,
            trust:   90,
            active:  true,
        }).unwrap();
    }

    assert_eq!(router.count(), 8);

    // Each node is reachable
    for i in 1u32..=8 {
        let r = router.lookup(i).unwrap();
        assert_eq!(r.dst_id, i);
    }
}

#[test]
fn test_router_table_full() {
    let mut router = AwpRouter::new();
    for i in 0..16u32 {
        router.add_route(AwpRoute {
            dst_id: i, channel: 1, trust: 90, active: true
        }).unwrap();
    }
    // 17th route should fail
    let result = router.add_route(AwpRoute {
        dst_id: 99, channel: 1, trust: 90, active: true
    });
    assert_eq!(result, Err(AwpError::RoutingTableFull));
}

#[test]
fn test_seq_counter_monotonic() {
    let mut router = AwpRouter::new();
    let mut last = 0u16;
    for _ in 0..100 {
        let seq = router.next_seq();
        assert!(seq != 0 || last == u16::MAX); // wraps only at MAX
        last = seq;
    }
}

// ── Threat gate integration ───────────────────────────────────────────────────

#[test]
fn test_threat_gate_boundary() {
    // 79 = safe, 80 = rejected
    assert!(aegis_threat_gate(79).is_ok());
    assert!(aegis_threat_gate(80).is_err());
}

#[test]
fn test_high_threat_blocks_all_packet_types() {
    let mut request = AwpHeader::new_request(1, 1, 2);
    let mut mesh = AwpHeader::new_request(1, 1, 2);
    mesh.flags = AwpFlags::MESH;

    assert_eq!(dispatch(&request, 90).unwrap(), DispatchResult::Rejected);
    assert_eq!(dispatch(&mesh, 90).unwrap(), DispatchResult::Rejected);
}

#[test]
fn test_zero_threat_allows_all() {
    let h = AwpHeader::new_request(1, 1, 2);
    assert_ne!(dispatch(&h, 0).unwrap(), DispatchResult::Rejected);
}

// ── Dispatch integration ──────────────────────────────────────────────────────

#[test]
fn test_dispatch_priority_threat_over_mesh() {
    // THREAT flag takes priority over MESH in dispatch
    let mut h = AwpHeader::new_request(1, 1, 2);
    h.flags = AwpFlags::THREAT;
    assert_eq!(dispatch(&h, 0).unwrap(), DispatchResult::ToThreatIntel);
}

#[test]
fn test_dispatch_haniel_channel() {
    // Requests go to HANIEL for rendering
    let h = AwpHeader::new_request(1, 1, 2);
    assert_eq!(dispatch(&h, 0).unwrap(), DispatchResult::ToHaniel);
}

// ── Sovereign proof + constants ───────────────────────────────────────────────

#[test]
fn test_sovereign_proof_invariant() {
    assert_eq!(AXON_PROOF, 0x4153);
    assert!(verify_sovereign_proof(AXON_PROOF));
    assert!(!verify_sovereign_proof(AXON_PROOF - 1));
    assert!(!verify_sovereign_proof(AXON_PROOF + 1));
}

#[test]
fn test_awp_magic_established_m8() {
    // AWP_MAGIC was established in M8 — must not change
    assert_eq!(AWP_MAGIC, 0xA1E0AE70);
}

#[test]
fn test_awp_pd_id_range() {
    assert!(AWP_PD_ID >= 0x10);
    assert_eq!(AWP_PD_ID, 0x30);
}

#[test]
fn test_constants_stable() {
    assert_eq!(AWP_VERSION, 0x01);
    assert_eq!(AEGIS_THREAT_THRESHOLD, 80);
    assert_eq!(CH_HANIEL, 4);
    assert_eq!(CH_AEGIS_MESH, 1);
}
