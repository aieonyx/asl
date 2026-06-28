// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// m23_multinode.rs — Integration tests for multi-node boot (M23)
// Target: 20+ tests, 0 failures
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

use asl_multinode::{
    MeshCoordinator, SovereignNode, NodeChannel, BootPhase, BootReport,
    MultinodeError, boot_phoenix_desktop, verify_phoenix_ready,
    verify_sovereign_proof,
    NODE_PRIMARY, NODE_BASTION, NODE_DATASTORE, NODE_RENDERER,
    AXON_PROOF, PHOENIX_NODE_COUNT, ARPI_CHANNEL_BASE, MAX_NODES,
};

// ── Full Phoenix desktop boot ─────────────────────────────────────────────────

#[test]
fn test_phoenix_desktop_full_boot() {
    let mesh = boot_phoenix_desktop().unwrap();

    // All 4 nodes online
    assert_eq!(mesh.online_count(), PHOENIX_NODE_COUNT);

    // All nodes sovereign proof verified
    assert_eq!(mesh.proven_count(), PHOENIX_NODE_COUNT);

    // 3 inter-node channels active (star topology)
    assert_eq!(mesh.active_channel_count(), 3);

    // Phoenix declared sovereign ready
    assert!(verify_phoenix_ready(&mesh).is_ok());
}

#[test]
fn test_phoenix_boot_report() {
    let mesh = boot_phoenix_desktop().unwrap();
    let report = BootReport::from_mesh(&mesh);

    assert_eq!(report.nodes_online, 4);
    assert_eq!(report.nodes_proven, 4);
    assert_eq!(report.channels_active, 3);
    assert!(report.phoenix_ready);
    assert!(report.ticks_elapsed > 0);
}

#[test]
fn test_all_nodes_carry_sovereign_proof() {
    let mesh = boot_phoenix_desktop().unwrap();
    for id in 0..PHOENIX_NODE_COUNT as u8 {
        let node = mesh.get_node(id).unwrap();
        assert_eq!(node.proof, AXON_PROOF);
        assert!(node.proof == 0x4153);
    }
}

#[test]
fn test_phoenix_topology_node_types() {
    let mesh = boot_phoenix_desktop().unwrap();
    assert_eq!(mesh.get_node(0).unwrap().node_type, NODE_PRIMARY);
    assert_eq!(mesh.get_node(1).unwrap().node_type, NODE_BASTION);
    assert_eq!(mesh.get_node(2).unwrap().node_type, NODE_DATASTORE);
    assert_eq!(mesh.get_node(3).unwrap().node_type, NODE_RENDERER);
}

#[test]
fn test_primary_node_has_all_mandatory_pds() {
    let mesh = boot_phoenix_desktop().unwrap();
    // Primary node carries all 6 mandatory PDs
    assert_eq!(mesh.get_node(NODE_PRIMARY).unwrap().pd_count, 6);
}

#[test]
fn test_renderer_node_is_haniel() {
    let mesh = boot_phoenix_desktop().unwrap();
    let renderer = mesh.get_node(NODE_RENDERER).unwrap();
    assert_eq!(renderer.node_type, NODE_RENDERER);
    assert!(renderer.is_online());
    assert_eq!(renderer.pd_count, 2); // HANIEL + AXON-Bridge
}

#[test]
fn test_datastore_node_edisondb() {
    let mesh = boot_phoenix_desktop().unwrap();
    let ds = mesh.get_node(NODE_DATASTORE).unwrap();
    assert_eq!(ds.node_type, NODE_DATASTORE);
    assert_eq!(ds.pd_count, 2); // DataTier + EdisonDB
}

// ── Mesh coordinator ──────────────────────────────────────────────────────────

#[test]
fn test_mesh_capacity() {
    let mut mesh = MeshCoordinator::new();
    for i in 0..MAX_NODES as u8 {
        mesh.register(SovereignNode::new(i, NODE_PRIMARY, ARPI_CHANNEL_BASE)).unwrap();
    }
    assert_eq!(mesh.count(), MAX_NODES);
    // 17th node rejected
    let result = mesh.register(SovereignNode::new(99, NODE_PRIMARY, 0));
    assert_eq!(result, Err(MultinodeError::MeshFull));
}

#[test]
fn test_mesh_tick_advances() {
    let mut mesh = MeshCoordinator::new();
    assert_eq!(mesh.current_tick(), 0);
    mesh.tick();
    assert_eq!(mesh.current_tick(), 1);
    mesh.tick();
    assert_eq!(mesh.current_tick(), 2);
}

#[test]
fn test_mesh_online_count_before_boot() {
    let mut mesh = MeshCoordinator::new();
    mesh.register(SovereignNode::new(0, NODE_PRIMARY, 0x10)).unwrap();
    assert_eq!(mesh.online_count(), 0); // not booted yet
}

// ── Boot sequence ─────────────────────────────────────────────────────────────

#[test]
fn test_boot_phases_in_order() {
    let mut node = SovereignNode::new(0, NODE_PRIMARY, ARPI_CHANNEL_BASE);
    assert_eq!(node.phase, BootPhase::Offline);
    node.advance(1).unwrap();
    assert_eq!(node.phase, BootPhase::Booting);
    node.advance(2).unwrap();
    assert_eq!(node.phase, BootPhase::Proving);
    node.advance(3).unwrap();
    assert_eq!(node.phase, BootPhase::PdsReady);
    node.advance(4).unwrap();
    assert_eq!(node.phase, BootPhase::MeshLinked);
    node.advance(5).unwrap();
    assert_eq!(node.phase, BootPhase::Online);
}

#[test]
fn test_proof_set_at_proving_phase() {
    let mut node = SovereignNode::new(0, NODE_PRIMARY, ARPI_CHANNEL_BASE);
    assert_eq!(node.proof, 0);
    node.advance(1).unwrap(); // Booting
    node.advance(2).unwrap(); // Proving — proof set here
    assert_eq!(node.proof, AXON_PROOF);
}

// ── Inter-node channels ───────────────────────────────────────────────────────

#[test]
fn test_star_topology_channels() {
    // Phoenix uses star topology: primary connects to all others
    let mut mesh = MeshCoordinator::new();
    for i in 0..4u8 {
        mesh.register(SovereignNode::new(i, i, ARPI_CHANNEL_BASE + i)).unwrap();
    }
    // 3 channels: 0↔1, 0↔2, 0↔3
    mesh.add_channel(NodeChannel::new(0, 1, 0x20)).unwrap();
    mesh.add_channel(NodeChannel::new(0, 2, 0x21)).unwrap();
    mesh.add_channel(NodeChannel::new(0, 3, 0x22)).unwrap();
    mesh.boot_all().unwrap();
    assert_eq!(mesh.active_channel_count(), 3);
}

#[test]
fn test_channel_self_link_rejected() {
    let mut ch = NodeChannel::new(0, 0, 0x10);
    assert_eq!(ch.activate(), Err(MultinodeError::ChannelFailed));
}

// ── Sovereign proof ───────────────────────────────────────────────────────────

#[test]
fn test_axon_proof_value() {
    assert_eq!(AXON_PROOF, 0x4153);
    assert!(verify_sovereign_proof(AXON_PROOF));
    assert!(!verify_sovereign_proof(0));
    assert!(!verify_sovereign_proof(AXON_PROOF - 1));
    assert!(!verify_sovereign_proof(AXON_PROOF + 1));
}

#[test]
fn test_phoenix_not_ready_without_renderer() {
    let mut mesh = MeshCoordinator::new();
    // Boot only 3 nodes (missing renderer)
    mesh.register(SovereignNode::new(0, NODE_PRIMARY,   0x10)).unwrap();
    mesh.register(SovereignNode::new(1, NODE_BASTION,   0x11)).unwrap();
    mesh.register(SovereignNode::new(2, NODE_DATASTORE, 0x12)).unwrap();
    mesh.boot_all().unwrap();
    assert_eq!(verify_phoenix_ready(&mesh), Err(MultinodeError::PhoenixNotReady));
}

#[test]
fn test_phoenix_node_count_constant() {
    assert_eq!(PHOENIX_NODE_COUNT, 4);
}

#[test]
fn test_max_nodes_constant() {
    assert_eq!(MAX_NODES, 16);
}
