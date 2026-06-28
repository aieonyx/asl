// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// asl-multinode — Multi-Node Boot Coordinator
//
// ASL M23: Boots multiple sovereign nodes under seL4.
// Each node is a full AIEONYX sovereign stack instance.
//
// Desktop (aixOs/Phoenix) multi-node topology:
//
//   Node 0 — PRIMARY       : Phoenix desktop OS (GENESIS + all PDs)
//   Node 1 — BASTION       : Sovereign security node (ARPi + TrustGraph)
//   Node 2 — DATASTORE     : EdisonDB PD (DataTier-Enforcer + WAL)
//   Node 3 — RENDERER      : HANIEL PD (1280×720 sovereign render)
//
// Boot sequence:
//   1. GENESIS boots on Node 0 — sovereignty proof verified
//   2. BASTION boots on Node 1 — ARPi mesh link established
//   3. DATASTORE boots on Node 2 — EdisonDB WAL ready
//   4. RENDERER boots on Node 3 — HANIEL surface live
//   5. Inter-node ARPi channels established (0↔1, 0↔2, 0↔3)
//   6. Phoenix desktop declared SOVEREIGN READY
//
// Sovereign proof: axon_main() → 0x4153 (invariant, all nodes)

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;
use alloc::vec::Vec;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum nodes in an ASL sovereign mesh.
pub const MAX_NODES: usize = 16;

/// Phoenix desktop topology — 4 nodes.
pub const PHOENIX_NODE_COUNT: usize = 4;

/// Sovereign proof value — invariant across all nodes.
pub const AXON_PROOF: u64 = 0x4153;

/// Node type identifiers.
pub const NODE_PRIMARY:   u8 = 0x00;
pub const NODE_BASTION:   u8 = 0x01;
pub const NODE_DATASTORE: u8 = 0x02;
pub const NODE_RENDERER:  u8 = 0x03;

/// Inter-node ARPi channel base.
pub const ARPI_CHANNEL_BASE: u8 = 0x10;

/// Boot timeout in ticks (simulated).
pub const BOOT_TIMEOUT_TICKS: u32 = 1000;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MultinodeError {
    /// Node failed sovereignty proof.
    SovereigntyFailed,
    /// Node boot timed out.
    BootTimeout,
    /// Inter-node ARPi channel establishment failed.
    ChannelFailed,
    /// Mesh is full — cannot add more nodes.
    MeshFull,
    /// Node ID already registered.
    DuplicateNode,
    /// Node not found.
    NodeNotFound,
    /// Invalid topology.
    InvalidTopology,
    /// Phoenix desktop not ready — missing required nodes.
    PhoenixNotReady,
}

// ── Node state ────────────────────────────────────────────────────────────────

/// Boot phase of a sovereign node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BootPhase {
    /// Node not yet started.
    Offline    = 0,
    /// Node booting — sovereignty proof in progress.
    Booting    = 1,
    /// Sovereignty proof passed — PDs initialising.
    Proving    = 2,
    /// All PDs live — awaiting mesh link.
    PdsReady   = 3,
    /// ARPi mesh link established — node sovereign ready.
    MeshLinked = 4,
    /// Node fully operational.
    Online     = 5,
}

/// A single sovereign node in the mesh.
#[derive(Debug, Clone, Copy)]
pub struct SovereignNode {
    pub node_id:    u8,
    pub node_type:  u8,
    pub phase:      BootPhase,
    pub proof:      u64,
    pub pd_count:   u8,
    pub arpi_chan:  u8,
    pub tick_start: u32,
}

impl SovereignNode {
    /// Create a new node in Offline state.
    pub fn new(node_id: u8, node_type: u8, arpi_chan: u8) -> Self {
        Self {
            node_id,
            node_type,
            phase:      BootPhase::Offline,
            proof:      0,
            pd_count:   0,
            arpi_chan,
            tick_start: 0,
        }
    }

    /// Advance node through boot phases.
    pub fn advance(&mut self, tick: u32) -> Result<BootPhase, MultinodeError> {
        match self.phase {
            BootPhase::Offline => {
                self.phase = BootPhase::Booting;
                self.tick_start = tick;
                Ok(self.phase)
            }
            BootPhase::Booting => {
                // Simulate sovereignty proof
                self.proof = AXON_PROOF;
                if self.proof != AXON_PROOF {
                    return Err(MultinodeError::SovereigntyFailed);
                }
                self.phase = BootPhase::Proving;
                Ok(self.phase)
            }
            BootPhase::Proving => {
                // PD count depends on node type
                self.pd_count = match self.node_type {
                    NODE_PRIMARY   => 6, // all mandatory PDs
                    NODE_BASTION   => 3, // ARPi + TrustGraph + SOMA
                    NODE_DATASTORE => 2, // DataTier + EdisonDB
                    NODE_RENDERER  => 2, // HANIEL + AXON-Bridge
                    _              => 1,
                };
                self.phase = BootPhase::PdsReady;
                Ok(self.phase)
            }
            BootPhase::PdsReady => {
                // Check boot timeout
                if tick - self.tick_start > BOOT_TIMEOUT_TICKS {
                    return Err(MultinodeError::BootTimeout);
                }
                self.phase = BootPhase::MeshLinked;
                Ok(self.phase)
            }
            BootPhase::MeshLinked => {
                self.phase = BootPhase::Online;
                Ok(self.phase)
            }
            BootPhase::Online => Ok(BootPhase::Online),
        }
    }

    /// Returns true if node is fully online.
    pub fn is_online(&self) -> bool {
        self.phase == BootPhase::Online
    }

    /// Returns true if sovereignty proof is valid.
    pub fn proof_valid(&self) -> bool {
        self.proof == AXON_PROOF
    }
}

// ── Inter-node channel ────────────────────────────────────────────────────────

/// An ARPi inter-node channel between two sovereign nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeChannel {
    pub src_id:  u8,
    pub dst_id:  u8,
    pub channel: u8,
    pub active:  bool,
}

impl NodeChannel {
    pub fn new(src_id: u8, dst_id: u8, channel: u8) -> Self {
        Self { src_id, dst_id, channel, active: false }
    }

    pub fn activate(&mut self) -> Result<(), MultinodeError> {
        if self.src_id == self.dst_id {
            return Err(MultinodeError::ChannelFailed);
        }
        self.active = true;
        Ok(())
    }
}

// ── Mesh coordinator ──────────────────────────────────────────────────────────

/// Multi-node boot coordinator — manages sovereign mesh boot.
pub struct MeshCoordinator {
    nodes:    [Option<SovereignNode>; MAX_NODES],
    channels: [Option<NodeChannel>; MAX_NODES],
    count:    usize,
    chan_count: usize,
    tick:     u32,
}

impl MeshCoordinator {
    pub const fn new() -> Self {
        Self {
            nodes:      [None; MAX_NODES],
            channels:   [None; MAX_NODES],
            count:      0,
            chan_count:  0,
            tick:       0,
        }
    }

    /// Register a sovereign node in the mesh.
    pub fn register(&mut self, node: SovereignNode) -> Result<(), MultinodeError> {
        if self.count >= MAX_NODES {
            return Err(MultinodeError::MeshFull);
        }
        for slot in self.nodes.iter() {
            if let Some(n) = slot {
                if n.node_id == node.node_id {
                    return Err(MultinodeError::DuplicateNode);
                }
            }
        }
        for slot in self.nodes.iter_mut() {
            if slot.is_none() {
                *slot = Some(node);
                self.count += 1;
                return Ok(());
            }
        }
        Err(MultinodeError::MeshFull)
    }

    /// Add an inter-node ARPi channel.
    pub fn add_channel(&mut self, ch: NodeChannel) -> Result<(), MultinodeError> {
        if self.chan_count >= MAX_NODES {
            return Err(MultinodeError::MeshFull);
        }
        for slot in self.channels.iter_mut() {
            if slot.is_none() {
                *slot = Some(ch);
                self.chan_count += 1;
                return Ok(());
            }
        }
        Err(MultinodeError::MeshFull)
    }

    /// Advance the boot tick — drives all nodes forward one phase.
    pub fn tick(&mut self) -> u32 {
        self.tick = self.tick.wrapping_add(1);
        self.tick
    }

    /// Boot all registered nodes to Online state.
    pub fn boot_all(&mut self) -> Result<usize, MultinodeError> {
        // Five phases: Offline→Booting→Proving→PdsReady→MeshLinked→Online
        for _ in 0..6 {
            let tick = self.tick();
            for slot in self.nodes.iter_mut() {
                if let Some(node) = slot {
                    if node.phase != BootPhase::Online {
                        node.advance(tick)?;
                    }
                }
            }
        }
        // Activate all channels
        for slot in self.channels.iter_mut() {
            if let Some(ch) = slot {
                ch.activate()?;
            }
        }
        Ok(self.online_count())
    }

    /// Count nodes currently online.
    pub fn online_count(&self) -> usize {
        self.nodes.iter()
            .filter(|s| s.map(|n| n.is_online()).unwrap_or(false))
            .count()
    }

    /// Count nodes with valid sovereignty proof.
    pub fn proven_count(&self) -> usize {
        self.nodes.iter()
            .filter(|s| s.map(|n| n.proof_valid()).unwrap_or(false))
            .count()
    }

    /// Total registered node count.
    pub fn count(&self) -> usize { self.count }

    /// Current tick value.
    pub fn current_tick(&self) -> u32 { self.tick }

    /// Look up a node by ID.
    pub fn get_node(&self, node_id: u8) -> Option<&SovereignNode> {
        for slot in self.nodes.iter() {
            if let Some(n) = slot {
                if n.node_id == node_id {
                    return Some(n);
                }
            }
        }
        None
    }

    /// Count active inter-node channels.
    pub fn active_channel_count(&self) -> usize {
        self.channels.iter()
            .filter(|s| s.map(|c| c.active).unwrap_or(false))
            .count()
    }
}

impl Default for MeshCoordinator {
    fn default() -> Self { Self::new() }
}

// ── Phoenix desktop topology ──────────────────────────────────────────────────

/// Boot the Phoenix desktop topology — 4 sovereign nodes.
///
/// Node 0: PRIMARY   — Phoenix OS + all mandatory PDs
/// Node 1: BASTION   — Security node
/// Node 2: DATASTORE — EdisonDB
/// Node 3: RENDERER  — HANIEL 1280×720
pub fn boot_phoenix_desktop() -> Result<MeshCoordinator, MultinodeError> {
    let mut mesh = MeshCoordinator::new();

    // Register nodes
    mesh.register(SovereignNode::new(0, NODE_PRIMARY,   ARPI_CHANNEL_BASE))?;
    mesh.register(SovereignNode::new(1, NODE_BASTION,   ARPI_CHANNEL_BASE + 1))?;
    mesh.register(SovereignNode::new(2, NODE_DATASTORE, ARPI_CHANNEL_BASE + 2))?;
    mesh.register(SovereignNode::new(3, NODE_RENDERER,  ARPI_CHANNEL_BASE + 3))?;

    // Register inter-node ARPi channels (star topology: primary ↔ all)
    mesh.add_channel(NodeChannel::new(0, 1, ARPI_CHANNEL_BASE + 0x10))?; // primary ↔ bastion
    mesh.add_channel(NodeChannel::new(0, 2, ARPI_CHANNEL_BASE + 0x11))?; // primary ↔ datastore
    mesh.add_channel(NodeChannel::new(0, 3, ARPI_CHANNEL_BASE + 0x12))?; // primary ↔ renderer

    // Boot all nodes
    mesh.boot_all()?;

    Ok(mesh)
}

/// Verify Phoenix desktop is sovereign ready.
pub fn verify_phoenix_ready(mesh: &MeshCoordinator) -> Result<(), MultinodeError> {
    if mesh.online_count() < PHOENIX_NODE_COUNT {
        return Err(MultinodeError::PhoenixNotReady);
    }
    if mesh.proven_count() < PHOENIX_NODE_COUNT {
        return Err(MultinodeError::SovereigntyFailed);
    }
    if mesh.active_channel_count() < 3 {
        return Err(MultinodeError::ChannelFailed);
    }
    Ok(())
}

/// Sovereign proof check.
pub fn verify_sovereign_proof(proof: u64) -> bool {
    proof == AXON_PROOF
}

// ── Boot report ───────────────────────────────────────────────────────────────

/// Summary of a completed multi-node boot.
#[derive(Debug)]
pub struct BootReport {
    pub nodes_online:   usize,
    pub nodes_proven:   usize,
    pub channels_active: usize,
    pub ticks_elapsed:  u32,
    pub phoenix_ready:  bool,
}

impl BootReport {
    pub fn from_mesh(mesh: &MeshCoordinator) -> Self {
        Self {
            nodes_online:    mesh.online_count(),
            nodes_proven:    mesh.proven_count(),
            channels_active: mesh.active_channel_count(),
            ticks_elapsed:   mesh.current_tick(),
            phoenix_ready:   verify_phoenix_ready(mesh).is_ok(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Node tests ────────────────────────────────────────────────────────────

    #[test]
    fn test_node_starts_offline() {
        let n = SovereignNode::new(0, NODE_PRIMARY, ARPI_CHANNEL_BASE);
        assert_eq!(n.phase, BootPhase::Offline);
        assert!(!n.is_online());
    }

    #[test]
    fn test_node_boot_sequence() {
        let mut n = SovereignNode::new(0, NODE_PRIMARY, ARPI_CHANNEL_BASE);
        n.advance(1).unwrap(); // → Booting
        assert_eq!(n.phase, BootPhase::Booting);
        n.advance(2).unwrap(); // → Proving
        assert_eq!(n.phase, BootPhase::Proving);
        assert!(n.proof_valid());
        n.advance(3).unwrap(); // → PdsReady
        assert_eq!(n.phase, BootPhase::PdsReady);
        n.advance(4).unwrap(); // → MeshLinked
        assert_eq!(n.phase, BootPhase::MeshLinked);
        n.advance(5).unwrap(); // → Online
        assert_eq!(n.phase, BootPhase::Online);
        assert!(n.is_online());
    }

    #[test]
    fn test_node_sovereignty_proof() {
        let mut n = SovereignNode::new(0, NODE_PRIMARY, ARPI_CHANNEL_BASE);
        n.advance(1).unwrap();
        n.advance(2).unwrap(); // Proving sets proof
        assert_eq!(n.proof, AXON_PROOF);
        assert!(n.proof_valid());
    }

    #[test]
    fn test_node_pd_counts() {
        let types = [
            (NODE_PRIMARY,   6u8),
            (NODE_BASTION,   3u8),
            (NODE_DATASTORE, 2u8),
            (NODE_RENDERER,  2u8),
        ];
        for (node_type, expected_pds) in types {
            let mut n = SovereignNode::new(0, node_type, 0);
            n.advance(1).unwrap();
            n.advance(2).unwrap();
            n.advance(3).unwrap(); // PdsReady sets pd_count
            assert_eq!(n.pd_count, expected_pds);
        }
    }

    #[test]
    fn test_node_already_online_stays_online() {
        let mut n = SovereignNode::new(0, NODE_PRIMARY, ARPI_CHANNEL_BASE);
        for i in 1..=6 { n.advance(i).unwrap(); }
        assert_eq!(n.advance(7).unwrap(), BootPhase::Online);
    }

    // ── Channel tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_channel_activates() {
        let mut ch = NodeChannel::new(0, 1, 0x10);
        assert!(!ch.active);
        ch.activate().unwrap();
        assert!(ch.active);
    }

    #[test]
    fn test_channel_self_link_rejected() {
        let mut ch = NodeChannel::new(0, 0, 0x10);
        assert_eq!(ch.activate(), Err(MultinodeError::ChannelFailed));
    }

    // ── Mesh coordinator tests ────────────────────────────────────────────────

    #[test]
    fn test_mesh_register_nodes() {
        let mut mesh = MeshCoordinator::new();
        mesh.register(SovereignNode::new(0, NODE_PRIMARY, 0x10)).unwrap();
        mesh.register(SovereignNode::new(1, NODE_BASTION, 0x11)).unwrap();
        assert_eq!(mesh.count(), 2);
    }

    #[test]
    fn test_mesh_duplicate_rejected() {
        let mut mesh = MeshCoordinator::new();
        mesh.register(SovereignNode::new(0, NODE_PRIMARY, 0x10)).unwrap();
        assert_eq!(
            mesh.register(SovereignNode::new(0, NODE_PRIMARY, 0x11)),
            Err(MultinodeError::DuplicateNode)
        );
    }

    #[test]
    fn test_mesh_lookup_node() {
        let mut mesh = MeshCoordinator::new();
        mesh.register(SovereignNode::new(0, NODE_PRIMARY, 0x10)).unwrap();
        let n = mesh.get_node(0).unwrap();
        assert_eq!(n.node_type, NODE_PRIMARY);
        assert!(mesh.get_node(99).is_none());
    }

    #[test]
    fn test_mesh_boot_all() {
        let mut mesh = MeshCoordinator::new();
        mesh.register(SovereignNode::new(0, NODE_PRIMARY,   0x10)).unwrap();
        mesh.register(SovereignNode::new(1, NODE_BASTION,   0x11)).unwrap();
        mesh.register(SovereignNode::new(2, NODE_DATASTORE, 0x12)).unwrap();
        let online = mesh.boot_all().unwrap();
        assert_eq!(online, 3);
        assert_eq!(mesh.proven_count(), 3);
    }

    // ── Phoenix desktop topology ──────────────────────────────────────────────

    #[test]
    fn test_phoenix_boot() {
        let mesh = boot_phoenix_desktop().unwrap();
        assert_eq!(mesh.online_count(), 4);
        assert_eq!(mesh.proven_count(), 4);
        assert_eq!(mesh.active_channel_count(), 3);
    }

    #[test]
    fn test_phoenix_sovereign_ready() {
        let mesh = boot_phoenix_desktop().unwrap();
        assert!(verify_phoenix_ready(&mesh).is_ok());
    }

    #[test]
    fn test_phoenix_all_nodes_proven() {
        let mesh = boot_phoenix_desktop().unwrap();
        for i in 0..4u8 {
            let n = mesh.get_node(i).unwrap();
            assert!(n.proof_valid());
            assert_eq!(n.proof, AXON_PROOF);
        }
    }

    #[test]
    fn test_phoenix_primary_has_six_pds() {
        let mesh = boot_phoenix_desktop().unwrap();
        let primary = mesh.get_node(NODE_PRIMARY).unwrap();
        assert_eq!(primary.pd_count, 6);
    }

    #[test]
    fn test_phoenix_renderer_node() {
        let mesh = boot_phoenix_desktop().unwrap();
        let renderer = mesh.get_node(NODE_RENDERER).unwrap();
        assert_eq!(renderer.node_type, NODE_RENDERER);
        assert!(renderer.is_online());
    }

    #[test]
    fn test_phoenix_channels_star_topology() {
        let mesh = boot_phoenix_desktop().unwrap();
        // 3 channels: primary↔bastion, primary↔datastore, primary↔renderer
        assert_eq!(mesh.active_channel_count(), 3);
    }

    // ── Boot report ───────────────────────────────────────────────────────────

    #[test]
    fn test_boot_report() {
        let mesh = boot_phoenix_desktop().unwrap();
        let report = BootReport::from_mesh(&mesh);
        assert_eq!(report.nodes_online, 4);
        assert_eq!(report.nodes_proven, 4);
        assert_eq!(report.channels_active, 3);
        assert!(report.phoenix_ready);
    }

    // ── Sovereign proof ───────────────────────────────────────────────────────

    #[test]
    fn test_sovereign_proof_constant() {
        assert_eq!(AXON_PROOF, 0x4153);
        assert!(verify_sovereign_proof(0x4153));
        assert!(!verify_sovereign_proof(0x0000));
    }

    #[test]
    fn test_phoenix_node_count() {
        assert_eq!(PHOENIX_NODE_COUNT, 4);
    }
}
